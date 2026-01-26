//! Flipper Zero Protocol Implementation

use anyhow::Result;

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    // System commands
    Ping,
    DeviceInfo,
    Reboot,
    RebootDfu,
    
    // Screen commands
    ScreenFrame,
    ScreenStream,
    
    // Button commands
    ButtonPress,
    ButtonRelease,
    
    // Storage commands
    StorageInfo,
    StorageList,
    StorageRead,
    StorageWrite,
    StorageDelete,
    StorageMkdir,
    StorageRename,
    
    // Application commands
    AppStart,
    AppStop,
    AppList,
    
    // GPIO commands
    GpioSetMode,
    GpioRead,
    GpioWrite,
    
    // Property commands
    PropertyGet,
    PropertySet,
}

/// Response status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStatus {
    Ok,
    Error,
    NotImplemented,
    Busy,
    InvalidArgument,
    StorageFull,
    FileNotFound,
    PermissionDenied,
}

impl ResponseStatus {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Ok,
            1 => Self::Error,
            2 => Self::NotImplemented,
            3 => Self::Busy,
            4 => Self::InvalidArgument,
            5 => Self::StorageFull,
            6 => Self::FileNotFound,
            7 => Self::PermissionDenied,
            _ => Self::Error,
        }
    }
    
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// Flipper CLI command builder
pub struct CommandBuilder {
    parts: Vec<String>,
}

impl CommandBuilder {
    pub fn new(cmd: &str) -> Self {
        Self {
            parts: vec![cmd.to_string()],
        }
    }
    
    pub fn arg(mut self, arg: &str) -> Self {
        self.parts.push(arg.to_string());
        self
    }
    
    pub fn arg_quoted(mut self, arg: &str) -> Self {
        self.parts.push(format!("\"{}\"", arg));
        self
    }
    
    pub fn build(self) -> String {
        self.parts.join(" ")
    }
}

/// Parse storage info response
pub fn parse_storage_info(response: &str) -> Result<(u64, u64)> {
    let mut total = 0u64;
    let mut free = 0u64;
    
    for line in response.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Total:") {
            total = parse_size(rest.trim());
        } else if let Some(rest) = line.strip_prefix("Free:") {
            free = parse_size(rest.trim());
        }
    }
    
    Ok((total, free))
}

/// Parse size string (e.g., "32.5 GB" -> bytes)
fn parse_size(s: &str) -> u64 {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 2 {
        return s.parse().unwrap_or(0);
    }
    
    let num: f64 = parts[0].parse().unwrap_or(0.0);
    let unit = parts[1].to_uppercase();
    
    let multiplier = match unit.as_str() {
        "B" => 1u64,
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        "TB" | "T" => 1024 * 1024 * 1024 * 1024,
        _ => 1,
    };
    
    (num * multiplier as f64) as u64
}

/// Parse device info response
pub fn parse_device_info(response: &str) -> DeviceInfo {
    let mut info = DeviceInfo::default();
    
    for line in response.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();
            
            match key.as_str() {
                "hardware_model" | "hardware_name" => info.hardware_model = value.to_string(),
                "hardware_version" => info.hardware_version = value.to_string(),
                "hardware_target" => info.hardware_target = value.to_string(),
                "firmware_version" => info.firmware_version = value.to_string(),
                "firmware_branch" => info.firmware_branch = value.to_string(),
                "firmware_build_date" => info.firmware_build_date = value.to_string(),
                "protobuf_version_major" => info.protobuf_version.0 = value.parse().unwrap_or(0),
                "protobuf_version_minor" => info.protobuf_version.1 = value.parse().unwrap_or(0),
                "device_name" => info.device_name = value.to_string(),
                _ => {}
            }
        }
    }
    
    info
}

/// Device information
#[derive(Debug, Clone, Default)]
pub struct DeviceInfo {
    pub hardware_model: String,
    pub hardware_version: String,
    pub hardware_target: String,
    pub firmware_version: String,
    pub firmware_branch: String,
    pub firmware_build_date: String,
    pub protobuf_version: (u32, u32),
    pub device_name: String,
}

impl DeviceInfo {
    pub fn display_name(&self) -> String {
        if !self.device_name.is_empty() {
            self.device_name.clone()
        } else {
            format!("Flipper {}", self.hardware_target)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024"), 1024);
        assert_eq!(parse_size("1 KB"), 1024);
        assert_eq!(parse_size("1 MB"), 1024 * 1024);
        assert_eq!(parse_size("1.5 GB"), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
    }
    
    #[test]
    fn test_command_builder() {
        let cmd = CommandBuilder::new("storage")
            .arg("read")
            .arg_quoted("/ext/test.txt")
            .build();
        assert_eq!(cmd, "storage read \"/ext/test.txt\"");
    }
}
