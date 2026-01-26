//! Screen capture and mirroring functionality

use anyhow::Result;
use std::io::Write;
use std::time::Duration;

use crate::device::FlipperDevice;

/// Flipper Zero screen dimensions
pub const SCREEN_WIDTH: u32 = 128;
pub const SCREEN_HEIGHT: u32 = 64;

/// Bytes per row (1-bit depth, 8 pixels per byte)
pub const BYTES_PER_ROW: usize = SCREEN_WIDTH as usize / 8;

/// Total frame size in bytes
pub const FRAME_SIZE: usize = BYTES_PER_ROW * SCREEN_HEIGHT as usize;

/// Screen frame data
#[derive(Debug, Clone)]
pub struct ScreenFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl ScreenFrame {
    /// Create a new empty frame
    pub fn new() -> Self {
        Self {
            data: vec![0; FRAME_SIZE],
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        }
    }
    
    /// Create from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < FRAME_SIZE {
            anyhow::bail!("Invalid frame size: {} (expected {})", data.len(), FRAME_SIZE);
        }
        
        Ok(Self {
            data: data[..FRAME_SIZE].to_vec(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        })
    }
    
    /// Get pixel value at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let byte_index = (y as usize * BYTES_PER_ROW) + (x as usize / 8);
        let bit_index = 7 - (x % 8);
        
        if byte_index < self.data.len() {
            (self.data[byte_index] >> bit_index) & 1 == 1
        } else {
            false
        }
    }
    
    /// Set pixel value at (x, y)
    pub fn set_pixel(&mut self, x: u32, y: u32, value: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let byte_index = (y as usize * BYTES_PER_ROW) + (x as usize / 8);
        let bit_index = 7 - (x % 8);
        
        if byte_index < self.data.len() {
            if value {
                self.data[byte_index] |= 1 << bit_index;
            } else {
                self.data[byte_index] &= !(1 << bit_index);
            }
        }
    }
    
    /// Convert to PNG bytes using image crate
    pub fn to_png(&self, scale: u32, invert: bool) -> Result<Vec<u8>> {
        use image::{GrayImage, Luma, ImageEncoder};
        
        let width = self.width * scale;
        let height = self.height * scale;
        
        let mut img = GrayImage::new(width, height);
        
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_pixel(x, y);
                let color = if invert {
                    if pixel { 0u8 } else { 255u8 }
                } else {
                    if pixel { 255u8 } else { 0u8 }
                };
                
                // Scale pixel
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = x * scale + sx;
                        let py = y * scale + sy;
                        img.put_pixel(px, py, Luma([color]));
                    }
                }
            }
        }
        
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder.write_image(
            img.as_raw(),
            width,
            height,
            image::ExtendedColorType::L8,
        )?;
        
        Ok(buffer)
    }
    
    /// Render to ASCII art
    pub fn to_ascii(&self) -> String {
        let mut output = String::new();
        
        // Top border
        output.push('┌');
        for _ in 0..self.width {
            output.push('─');
        }
        output.push('┐');
        output.push('\n');
        
        // Pixels (2 rows per character using block elements)
        for y in (0..self.height).step_by(2) {
            output.push('│');
            for x in 0..self.width {
                let top = self.get_pixel(x, y);
                let bottom = if y + 1 < self.height {
                    self.get_pixel(x, y + 1)
                } else {
                    false
                };
                
                let ch = match (top, bottom) {
                    (false, false) => ' ',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (true, true) => '█',
                };
                output.push(ch);
            }
            output.push('│');
            output.push('\n');
        }
        
        // Bottom border
        output.push('└');
        for _ in 0..self.width {
            output.push('─');
        }
        output.push('┘');
        
        output
    }
}

impl Default for ScreenFrame {
    fn default() -> Self {
        Self::new()
    }
}

/// CLI screen mirroring entry point
pub async fn cli_mirror(device: &FlipperDevice, frame_rate: u32) -> Result<()> {
    println!("🖥️  PineFlip Screen Mirror");
    println!("══════════════════════════════════════════");
    println!();
    println!("✅ Connected to: {}", device.name());
    println!();
    println!("Controls:");
    println!("  Ctrl+C - Exit");
    println!();
    
    let delay = Duration::from_millis(1000 / frame_rate as u64);
    
    // Hide cursor
    print!("\x1b[?25l");
    
    // Clear screen
    print!("\x1b[2J");
    
    loop {
        match device.get_screen_frame().await {
            Ok(frame) => {
                // Move cursor to top
                print!("\x1b[H");
                
                // Convert to our ScreenFrame type and display
                let screen_frame = ScreenFrame::from_bytes(&frame.data)?;
                println!("{}", screen_frame.to_ascii());
                std::io::stdout().flush()?;
            }
            Err(e) => {
                eprintln!("\nCapture error: {}", e);
                break;
            }
        }
        
        tokio::time::sleep(delay).await;
    }
    
    // Show cursor
    print!("\x1b[?25h");
    
    Ok(())
}

/// Save frame to file
pub fn save_frame(frame: &ScreenFrame, path: &str, scale: u32, invert: bool) -> Result<()> {
    let png_data = frame.to_png(scale, invert)?;
    std::fs::write(path, png_data)?;
    println!("Screenshot saved to: {}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pixel_access() {
        let mut frame = ScreenFrame::new();
        
        assert!(!frame.get_pixel(0, 0));
        frame.set_pixel(0, 0, true);
        assert!(frame.get_pixel(0, 0));
        
        frame.set_pixel(127, 63, true);
        assert!(frame.get_pixel(127, 63));
    }
    
    #[test]
    fn test_frame_size() {
        assert_eq!(FRAME_SIZE, 1024); // 128 * 64 / 8
    }
}
