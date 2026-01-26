//! File operations for Flipper Zero storage

use anyhow::{Context, Result};
use std::path::Path;
use std::fs;
use std::io::{Read, Write};

use crate::device::FlipperDevice;

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

impl FileInfo {
    pub fn new(name: &str, path: &str, is_dir: bool, size: u64) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            is_dir,
            size,
        }
    }
    
    /// Get human-readable size
    pub fn size_string(&self) -> String {
        if self.is_dir {
            return "-".to_string();
        }
        
        format_size(self.size)
    }
    
    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        if self.is_dir {
            return None;
        }
        
        self.name.rsplit('.').next()
    }
    
    /// Get icon name for this file type
    pub fn icon_name(&self) -> &'static str {
        if self.is_dir {
            return "folder-symbolic";
        }
        
        match self.extension() {
            Some("sub") => "audio-radio-symbolic",
            Some("ir") | Some("irdb") => "video-display-symbolic",
            Some("nfc") => "nfc-symbolic",
            Some("rfid") => "auth-sim-symbolic",
            Some("ibutton") => "key-symbolic",
            Some("txt") | Some("log") => "text-x-generic-symbolic",
            Some("js") | Some("fap") => "application-x-executable-symbolic",
            Some("bak") => "document-save-symbolic",
            _ => "text-x-generic-symbolic",
        }
    }
}

/// Storage information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub label: String,
}

impl StorageInfo {
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }
    
    pub fn used_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        
        (self.used_bytes() as f64 / self.total_bytes as f64) * 100.0
    }
}

/// Format byte size to human-readable string
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// File manager for Flipper Zero storage
pub struct FileManager {
    device: FlipperDevice,
    current_path: String,
}

impl FileManager {
    pub fn new(device: FlipperDevice) -> Self {
        Self {
            device,
            current_path: "/ext".to_string(),
        }
    }
    
    /// Get current directory
    pub fn current_path(&self) -> &str {
        &self.current_path
    }
    
    /// Change directory
    pub async fn cd(&mut self, path: &str) -> Result<()> {
        let new_path = if path.starts_with('/') {
            path.to_string()
        } else if path == ".." {
            let parent = Path::new(&self.current_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            parent
        } else {
            format!("{}/{}", self.current_path, path)
        };
        
        // Verify directory exists
        let _entries = self.device.list_directory(&new_path).await
            .context("Failed to access directory")?;
        
        self.current_path = new_path;
        Ok(())
    }
    
    /// List current directory
    pub async fn list(&mut self) -> Result<Vec<FileInfo>> {
        let device_files = self.device.list_directory(&self.current_path).await?;
        Ok(device_files.into_iter().map(|f| FileInfo::new(
            &f.name,
            &format!("{}/{}", self.current_path, f.name),
            f.is_directory,
            f.size,
        )).collect())
    }
    
    /// Read file contents
    pub async fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path);
        self.device.read_file(&full_path).await
    }
    
    /// Write file
    pub async fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.resolve_path(path);
        self.device.write_file(&full_path, data).await
    }
    
    /// Delete file or directory
    pub async fn delete(&mut self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path);
        self.device.delete_path(&full_path).await
    }
    
    /// Create directory
    pub async fn mkdir(&mut self, name: &str) -> Result<()> {
        let full_path = format!("{}/{}", self.current_path, name);
        self.device.mkdir(&full_path).await
    }
    
    /// Rename file or directory
    pub async fn rename(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        let old_path = self.resolve_path(old_name);
        let new_path = format!("{}/{}", self.current_path, new_name);
        self.device.rename(&old_path, &new_path).await
    }
    
    /// Download file from Flipper to local filesystem
    pub async fn download(&mut self, remote_path: &str, local_path: &str) -> Result<()> {
        let full_remote = self.resolve_path(remote_path);
        let data = self.device.read_file(&full_remote).await?;
        
        let mut file = fs::File::create(local_path)?;
        file.write_all(&data)?;
        
        Ok(())
    }
    
    /// Upload file from local filesystem to Flipper
    pub async fn upload(&mut self, local_path: &str, remote_name: &str) -> Result<()> {
        let mut file = fs::File::open(local_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        
        let full_remote = format!("{}/{}", self.current_path, remote_name);
        self.device.write_file(&full_remote, &data).await
    }
    
    /// Get storage info
    pub async fn storage_info(&mut self, path: &str) -> Result<StorageInfo> {
        let info = self.device.get_storage_info(path).await?;
        Ok(StorageInfo {
            total_bytes: info.total,
            free_bytes: info.free,
            label: path.to_string(),
        })
    }
    
    /// Resolve path relative to current directory
    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.current_path, path)
        }
    }
}

/// CLI file operations
pub mod cli {
    use super::*;
    
    pub async fn list_files(device: &FlipperDevice, path: &str) -> Result<()> {
        let entries = device.list_directory(path).await?;
        
        println!("📁 {} ({} items)", path, entries.len());
        println!("────────────────────────────────────────");
        
        // Directories first
        let mut dirs: Vec<_> = entries.iter().filter(|e| e.is_directory).collect();
        let mut files: Vec<_> = entries.iter().filter(|e| !e.is_directory).collect();
        
        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        for dir in dirs {
            println!("  📁 {}/", dir.name);
        }
        
        for file in files {
            println!("  📄 {} ({})", file.name, format_size(file.size));
        }
        
        Ok(())
    }
    
    pub async fn download_file(device: &FlipperDevice, remote: &str, local: &str) -> Result<()> {
        println!("⬇️  Downloading {} -> {}", remote, local);
        
        let data = device.read_file(remote).await?;
        
        let mut file = fs::File::create(local)?;
        file.write_all(&data)?;
        
        println!("✅ Downloaded {} bytes", data.len());
        
        Ok(())
    }
    
    pub async fn upload_file(device: &FlipperDevice, local: &str, remote: &str) -> Result<()> {
        println!("⬆️  Uploading {} -> {}", local, remote);
        
        let mut file = fs::File::open(local)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        
        device.write_file(remote, &data).await?;
        
        println!("✅ Uploaded {} bytes", data.len());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
    
    #[test]
    fn test_file_info_icon() {
        let sub = FileInfo::new("test.sub", "/ext/test.sub", false, 100);
        assert_eq!(sub.icon_name(), "audio-radio-symbolic");
        
        let dir = FileInfo::new("folder", "/ext/folder", true, 0);
        assert_eq!(dir.icon_name(), "folder-symbolic");
    }
}
