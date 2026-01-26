//! Configuration management for PineFlip

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Connection settings
    #[serde(default)]
    pub connection: ConnectionConfig,
    
    /// Screen capture settings
    #[serde(default)]
    pub screen: ScreenConfig,
    
    /// File manager settings
    #[serde(default)]
    pub files: FilesConfig,
    
    /// Appearance settings
    #[serde(default)]
    pub appearance: AppearanceConfig,
    
    /// Keyboard shortcuts
    #[serde(default)]
    pub shortcuts: ShortcutsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            screen: ScreenConfig::default(),
            files: FilesConfig::default(),
            appearance: AppearanceConfig::default(),
            shortcuts: ShortcutsConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        
        Ok(())
    }
    
    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pineflip")
            .join("config.toml")
    }
    
    /// Get data directory path
    pub fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pineflip")
    }
    
    /// Get cache directory path
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pineflip")
    }
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Auto-connect on startup
    #[serde(default = "default_true")]
    pub auto_connect: bool,
    
    /// Preferred port (empty = auto-detect)
    #[serde(default)]
    pub preferred_port: String,
    
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    
    /// Auto-reconnect on disconnect
    #[serde(default = "default_true")]
    pub auto_reconnect: bool,
    
    /// Reconnect delay in seconds
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay_secs: u32,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            auto_connect: true,
            preferred_port: String::new(),
            timeout_secs: 5,
            auto_reconnect: true,
            reconnect_delay_secs: 2,
        }
    }
}

/// Screen capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenConfig {
    /// Frame rate for screen mirroring
    #[serde(default = "default_frame_rate")]
    pub frame_rate: u32,
    
    /// Display scale factor
    #[serde(default = "default_scale")]
    pub scale: u32,
    
    /// Invert colors
    #[serde(default)]
    pub invert_colors: bool,
    
    /// Show grid overlay
    #[serde(default)]
    pub show_grid: bool,
    
    /// Auto-save screenshots
    #[serde(default)]
    pub auto_save: bool,
    
    /// Screenshot save directory
    #[serde(default = "default_screenshot_dir")]
    pub save_directory: String,
    
    /// Screenshot format
    #[serde(default)]
    pub save_format: ScreenshotFormat,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self {
            frame_rate: 10,
            scale: 4,
            invert_colors: false,
            show_grid: false,
            auto_save: false,
            save_directory: default_screenshot_dir(),
            save_format: ScreenshotFormat::Png,
        }
    }
}

/// Screenshot format
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    #[default]
    Png,
    Gif,
    Bmp,
}

impl ScreenshotFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
        }
    }
}

/// File manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    /// Show hidden files
    #[serde(default)]
    pub show_hidden: bool,
    
    /// Sort by (name, size, date)
    #[serde(default)]
    pub sort_by: SortBy,
    
    /// Sort ascending
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
    
    /// Confirm before delete
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    
    /// Default download directory
    #[serde(default = "default_download_dir")]
    pub download_directory: String,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_by: SortBy::Name,
            sort_ascending: true,
            confirm_delete: true,
            download_directory: default_download_dir(),
        }
    }
}

/// File sort order
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    #[default]
    Name,
    Size,
    Type,
    Modified,
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Use system color scheme
    #[serde(default = "default_true")]
    pub follow_system_theme: bool,
    
    /// Dark mode (when not following system)
    #[serde(default)]
    pub dark_mode: bool,
    
    /// Compact mode
    #[serde(default)]
    pub compact_mode: bool,
    
    /// Show status bar
    #[serde(default = "default_true")]
    pub show_status_bar: bool,
    
    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: i32,
    
    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: i32,
    
    /// Remember window size
    #[serde(default = "default_true")]
    pub remember_window_size: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            follow_system_theme: true,
            dark_mode: false,
            compact_mode: false,
            show_status_bar: true,
            window_width: 1000,
            window_height: 700,
            remember_window_size: true,
        }
    }
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    pub connect: String,
    pub disconnect: String,
    pub screenshot: String,
    pub record: String,
    pub refresh: String,
    pub upload: String,
    pub download: String,
    pub delete: String,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            connect: "<Ctrl>k".to_string(),
            disconnect: "<Ctrl><Shift>k".to_string(),
            screenshot: "<Ctrl>s".to_string(),
            record: "<Ctrl>r".to_string(),
            refresh: "F5".to_string(),
            upload: "<Ctrl>u".to_string(),
            download: "<Ctrl>d".to_string(),
            delete: "Delete".to_string(),
        }
    }
}

// Default value helpers
fn default_true() -> bool {
    true
}

fn default_timeout() -> u32 {
    5
}

fn default_reconnect_delay() -> u32 {
    2
}

fn default_frame_rate() -> u32 {
    10
}

fn default_scale() -> u32 {
    4
}

fn default_screenshot_dir() -> String {
    dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PineFlip")
        .to_string_lossy()
        .to_string()
}

fn default_download_dir() -> String {
    dirs::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .to_string_lossy()
        .to_string()
}

fn default_window_width() -> i32 {
    1000
}

fn default_window_height() -> i32 {
    700
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.connection.auto_connect);
        assert_eq!(config.screen.frame_rate, 10);
        assert_eq!(config.appearance.window_width, 1000);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.screen.frame_rate, config.screen.frame_rate);
    }
}
