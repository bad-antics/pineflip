//! Flipper Zero Device Communication

use anyhow::{anyhow, Result};
use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Flipper Zero USB Vendor/Product IDs
const FLIPPER_VID: u16 = 0x0483;
const FLIPPER_PID: u16 = 0x5740;

/// Flipper Zero device representation
pub struct FlipperDevice {
    name: String,
    port_name: String,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    firmware_version: Option<String>,
    hardware_version: Option<String>,
}

impl FlipperDevice {
    /// Connect to a specific serial port
    pub async fn connect(port_name: &str) -> Result<Self> {
        tracing::info!("Connecting to {}...", port_name);
        
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(1000))
            .open()?;
        
        let mut device = Self {
            name: "Flipper Zero".to_string(),
            port_name: port_name.to_string(),
            port: Arc::new(Mutex::new(port)),
            firmware_version: None,
            hardware_version: None,
        };
        
        // Query device info
        device.query_device_info().await?;
        
        Ok(device)
    }
    
    /// Auto-detect and connect to Flipper Zero
    pub async fn auto_detect() -> Result<Self> {
        let ports = serialport::available_ports()?;
        
        for port_info in ports {
            if Self::is_flipper_port(&port_info) {
                tracing::info!("Found Flipper Zero at {}", port_info.port_name);
                return Self::connect(&port_info.port_name).await;
            }
        }
        
        Err(anyhow!("No Flipper Zero device found"))
    }
    
    /// Check if a serial port is a Flipper Zero
    fn is_flipper_port(port_info: &SerialPortInfo) -> bool {
        match &port_info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                usb_info.vid == FLIPPER_VID && usb_info.pid == FLIPPER_PID
            }
            _ => false,
        }
    }
    
    /// List all available Flipper Zero devices
    pub fn list_devices() -> Result<Vec<SerialPortInfo>> {
        let ports = serialport::available_ports()?;
        let flippers: Vec<_> = ports
            .into_iter()
            .filter(Self::is_flipper_port)
            .collect();
        Ok(flippers)
    }
    
    /// Get device name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get port name
    pub fn port_name(&self) -> &str {
        &self.port_name
    }
    
    /// Get firmware version
    pub fn firmware_version(&self) -> Option<&str> {
        self.firmware_version.as_deref()
    }
    
    /// Get hardware version
    pub fn hardware_version(&self) -> Option<&str> {
        self.hardware_version.as_deref()
    }
    
    /// Check if device is still connected
    pub async fn is_connected(&self) -> bool {
        // Try to send a ping
        match self.send_command("ping").await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    /// Query device information
    async fn query_device_info(&mut self) -> Result<()> {
        // Send device info request
        let response = self.send_command("device_info").await?;
        
        // Parse response
        for line in response.lines() {
            if let Some(version) = line.strip_prefix("firmware_version:") {
                self.firmware_version = Some(version.trim().to_string());
            } else if let Some(version) = line.strip_prefix("hardware_version:") {
                self.hardware_version = Some(version.trim().to_string());
            } else if let Some(name) = line.strip_prefix("hardware_name:") {
                self.name = name.trim().to_string();
            }
        }
        
        Ok(())
    }
    
    /// Send a CLI command to the device
    pub async fn send_command(&self, command: &str) -> Result<String> {
        let mut port = self.port.lock().await;
        
        // Send command
        writeln!(port, "{}", command)?;
        port.flush()?;
        
        // Read response
        let mut response = String::new();
        let mut buf = [0u8; 1024];
        
        loop {
            match port.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if response.contains(">:") || response.contains("\r\n\r\n") {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => return Err(e.into()),
            }
        }
        
        Ok(response)
    }
    
    /// Start screen capture stream
    pub async fn start_screen_stream(&self) -> Result<ScreenStream> {
        // Enter screen stream mode
        self.send_command("screen_stream").await?;
        
        Ok(ScreenStream {
            port: Arc::clone(&self.port),
            running: true,
        })
    }
    
    /// Get screen frame (128x64 1-bit display)
    pub async fn get_screen_frame(&self) -> Result<ScreenFrame> {
        let response = self.send_command("screen_frame").await?;
        ScreenFrame::from_bytes(response.as_bytes())
    }
    
    /// Send button press
    pub async fn press_button(&self, button: Button) -> Result<()> {
        let cmd = match button {
            Button::Up => "button up",
            Button::Down => "button down",
            Button::Left => "button left",
            Button::Right => "button right",
            Button::Ok => "button ok",
            Button::Back => "button back",
        };
        self.send_command(cmd).await?;
        Ok(())
    }
    
    /// Release button
    pub async fn release_button(&self, button: Button) -> Result<()> {
        let cmd = match button {
            Button::Up => "button up release",
            Button::Down => "button down release",
            Button::Left => "button left release",
            Button::Right => "button right release",
            Button::Ok => "button ok release",
            Button::Back => "button back release",
        };
        self.send_command(cmd).await?;
        Ok(())
    }
    
    /// List files in directory
    pub async fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>> {
        let response = self.send_command(&format!("storage list {}", path)).await?;
        
        let mut files = Vec::new();
        for line in response.lines() {
            if let Some(file_info) = FileInfo::parse(line) {
                files.push(file_info);
            }
        }
        
        Ok(files)
    }
    
    /// Read file from device
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let response = self.send_command(&format!("storage read_chunks {}", path)).await?;
        Ok(response.into_bytes())
    }
    
    /// Write file to device
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        // Start write
        self.send_command(&format!("storage write_start {}", path)).await?;
        
        // Write chunks
        for chunk in data.chunks(512) {
            let mut port = self.port.lock().await;
            port.write_all(chunk)?;
            port.flush()?;
        }
        
        // End write
        self.send_command("storage write_end").await?;
        
        Ok(())
    }
    
    /// Delete file from device
    pub async fn delete_file(&self, path: &str) -> Result<()> {
        self.send_command(&format!("storage remove {}", path)).await?;
        Ok(())
    }
    
    /// Delete file or directory (alias for delete_file)
    pub async fn delete_path(&self, path: &str) -> Result<()> {
        self.delete_file(path).await
    }
    
    /// Create directory
    pub async fn mkdir(&self, path: &str) -> Result<()> {
        self.send_command(&format!("storage mkdir {}", path)).await?;
        Ok(())
    }
    
    /// Rename/move file
    pub async fn rename(&self, old_path: &str, new_path: &str) -> Result<()> {
        self.send_command(&format!("storage rename {} {}", old_path, new_path)).await?;
        Ok(())
    }

    /// Get device storage info
    pub async fn get_storage_info(&self, storage: &str) -> Result<StorageInfo> {
        let response = self.send_command(&format!("storage info {}", storage)).await?;
        StorageInfo::parse(&response)
    }
    
    /// Reboot device
    pub async fn reboot(&self) -> Result<()> {
        self.send_command("power reboot").await?;
        Ok(())
    }
    
    /// Reboot to DFU mode
    pub async fn reboot_dfu(&self) -> Result<()> {
        self.send_command("power reboot dfu").await?;
        Ok(())
    }
}

/// Screen capture stream
pub struct ScreenStream {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    running: bool,
}

impl ScreenStream {
    /// Get next frame
    pub async fn next_frame(&mut self) -> Result<Option<ScreenFrame>> {
        if !self.running {
            return Ok(None);
        }
        
        let mut port = self.port.lock().await;
        let mut buf = [0u8; 1024];
        
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                ScreenFrame::from_bytes(&buf[..n]).map(Some)
            }
            Ok(_) => Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Stop stream
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// Screen frame (128x64 1-bit display)
#[derive(Clone)]
pub struct ScreenFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ScreenFrame {
    pub const WIDTH: u32 = 128;
    pub const HEIGHT: u32 = 64;
    
    /// Create from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Flipper screen is 128x64 = 8192 pixels = 1024 bytes (1-bit per pixel)
        let expected_size = (Self::WIDTH * Self::HEIGHT / 8) as usize;
        
        if data.len() < expected_size {
            return Err(anyhow!("Insufficient screen data: {} < {}", data.len(), expected_size));
        }
        
        Ok(Self {
            width: Self::WIDTH,
            height: Self::HEIGHT,
            data: data[..expected_size].to_vec(),
        })
    }
    
    /// Get pixel at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let byte_idx = ((y * self.width + x) / 8) as usize;
        let bit_idx = (x % 8) as u8;
        
        if byte_idx < self.data.len() {
            (self.data[byte_idx] >> bit_idx) & 1 == 1
        } else {
            false
        }
    }
    
    /// Convert to RGBA image data
    pub fn to_rgba(&self, scale: u32) -> Vec<u8> {
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
        let mut rgba = vec![0u8; (scaled_width * scaled_height * 4) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_pixel(x, y);
                let color = if pixel { [0xFF, 0x80, 0x00, 0xFF] } else { [0x00, 0x00, 0x00, 0xFF] };
                
                // Scale pixel
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = x * scale + sx;
                        let py = y * scale + sy;
                        let idx = ((py * scaled_width + px) * 4) as usize;
                        rgba[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
        }
        
        rgba
    }
}

/// Button on Flipper Zero
#[derive(Debug, Clone, Copy)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    Ok,
    Back,
}

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
}

impl FileInfo {
    fn parse(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }
        
        let is_directory = parts[0] == "[D]";
        let name = parts[1].to_string();
        let size = if parts.len() > 2 {
            parts[2].parse().unwrap_or(0)
        } else {
            0
        };
        
        Some(Self {
            name,
            is_directory,
            size,
        })
    }
}

/// Storage information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub total: u64,
    pub free: u64,
}

impl StorageInfo {
    fn parse(response: &str) -> Result<Self> {
        let mut total = 0u64;
        let mut free = 0u64;
        
        for line in response.lines() {
            if let Some(value) = line.strip_prefix("total:") {
                total = value.trim().parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("free:") {
                free = value.trim().parse().unwrap_or(0);
            }
        }
        
        Ok(Self { total, free })
    }
}
