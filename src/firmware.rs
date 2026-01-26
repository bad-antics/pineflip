//! Firmware management for Flipper Zero

use anyhow::Result;
use std::path::Path;

/// Firmware update channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FirmwareChannel {
    #[default]
    Release,
    ReleaseCandidate,
    Dev,
    Custom,
}

impl FirmwareChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::ReleaseCandidate => "rc",
            Self::Dev => "dev",
            Self::Custom => "custom",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Release => "Release (Stable)",
            Self::ReleaseCandidate => "Release Candidate",
            Self::Dev => "Development",
            Self::Custom => "Custom Firmware",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            Self::Release => "Recommended for most users. Stable and well-tested.",
            Self::ReleaseCandidate => "Preview of upcoming release. May have minor bugs.",
            Self::Dev => "Latest features but may be unstable.",
            Self::Custom => "Third-party firmware distributions.",
        }
    }
    
    pub fn update_url(&self) -> &'static str {
        match self {
            Self::Release => "https://update.flipperzero.one/firmware/release",
            Self::ReleaseCandidate => "https://update.flipperzero.one/firmware/rc",
            Self::Dev => "https://update.flipperzero.one/firmware/dev",
            Self::Custom => "",
        }
    }
}

/// Firmware version information
#[derive(Debug, Clone)]
pub struct FirmwareVersion {
    pub version: String,
    pub channel: FirmwareChannel,
    pub commit: String,
    pub build_date: String,
    pub target: String,
}

impl FirmwareVersion {
    pub fn parse(version_str: &str) -> Option<Self> {
        // Parse version string like "0.99.1-rc" or "0.98.3"
        let parts: Vec<&str> = version_str.split('-').collect();
        
        let version = parts.first()?.to_string();
        let channel = if version_str.contains("-rc") {
            FirmwareChannel::ReleaseCandidate
        } else if version_str.contains("-dev") {
            FirmwareChannel::Dev
        } else {
            FirmwareChannel::Release
        };
        
        Some(Self {
            version,
            channel,
            commit: String::new(),
            build_date: String::new(),
            target: "f7".to_string(),
        })
    }
    
    pub fn display(&self) -> String {
        if self.commit.is_empty() {
            self.version.clone()
        } else {
            format!("{} ({})", self.version, &self.commit[..7.min(self.commit.len())])
        }
    }
}

/// Firmware update status
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Idle,
    CheckingForUpdates,
    UpdateAvailable {
        current: FirmwareVersion,
        latest: FirmwareVersion,
    },
    UpToDate {
        version: FirmwareVersion,
    },
    Downloading {
        progress: f32,
        total_bytes: u64,
        downloaded_bytes: u64,
    },
    Extracting,
    Flashing {
        progress: f32,
    },
    Verifying,
    Complete,
    Error(String),
}

impl UpdateStatus {
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            Self::CheckingForUpdates
                | Self::Downloading { .. }
                | Self::Extracting
                | Self::Flashing { .. }
                | Self::Verifying
        )
    }
    
    pub fn message(&self) -> String {
        match self {
            Self::Idle => "Ready".to_string(),
            Self::CheckingForUpdates => "Checking for updates...".to_string(),
            Self::UpdateAvailable { latest, .. } => {
                format!("Update available: {}", latest.display())
            }
            Self::UpToDate { version } => {
                format!("Up to date ({})", version.display())
            }
            Self::Downloading { progress, .. } => {
                format!("Downloading... {:.0}%", progress * 100.0)
            }
            Self::Extracting => "Extracting firmware...".to_string(),
            Self::Flashing { progress } => {
                format!("Flashing... {:.0}%", progress * 100.0)
            }
            Self::Verifying => "Verifying installation...".to_string(),
            Self::Complete => "Update complete! Device will restart.".to_string(),
            Self::Error(msg) => format!("Error: {}", msg),
        }
    }
}

/// Firmware update manifest
#[derive(Debug, Clone)]
pub struct UpdateManifest {
    pub version: FirmwareVersion,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub changelog: Vec<String>,
}

/// Firmware updater
pub struct FirmwareUpdater {
    channel: FirmwareChannel,
    custom_url: Option<String>,
}

impl FirmwareUpdater {
    pub fn new() -> Self {
        Self {
            channel: FirmwareChannel::Release,
            custom_url: None,
        }
    }
    
    pub fn with_channel(channel: FirmwareChannel) -> Self {
        Self {
            channel,
            custom_url: None,
        }
    }
    
    pub fn channel(&self) -> FirmwareChannel {
        self.channel
    }
    
    pub fn set_channel(&mut self, channel: FirmwareChannel) {
        self.channel = channel;
    }
    
    pub fn set_custom_url(&mut self, url: String) {
        self.custom_url = Some(url);
        self.channel = FirmwareChannel::Custom;
    }
    
    /// Check for available updates
    pub async fn check_for_updates(&self, current_version: &str) -> Result<UpdateStatus> {
        // In a real implementation, this would fetch from the update server
        let current = FirmwareVersion::parse(current_version)
            .unwrap_or_else(|| FirmwareVersion {
                version: current_version.to_string(),
                channel: self.channel,
                commit: String::new(),
                build_date: String::new(),
                target: "f7".to_string(),
            });
        
        // Simulated check - in reality would query API
        Ok(UpdateStatus::UpToDate { version: current })
    }
    
    /// Download firmware update
    pub async fn download(&self, manifest: &UpdateManifest, _on_progress: impl Fn(f32)) -> Result<Vec<u8>> {
        // Download firmware bundle
        // In real implementation, use reqwest to download
        anyhow::bail!("Firmware download not yet implemented")
    }
    
    /// Flash firmware to device
    pub async fn flash(&self, _firmware_path: &Path, _on_progress: impl Fn(f32)) -> Result<()> {
        // Flash firmware using DFU mode
        anyhow::bail!("Firmware flashing not yet implemented")
    }
}

impl Default for FirmwareUpdater {
    fn default() -> Self {
        Self::new()
    }
}

/// Popular custom firmware distributions
#[derive(Debug, Clone)]
pub struct CustomFirmware {
    pub name: &'static str,
    pub description: &'static str,
    pub url: &'static str,
    pub github: &'static str,
}

pub const CUSTOM_FIRMWARES: &[CustomFirmware] = &[
    CustomFirmware {
        name: "Momentum",
        description: "Feature-rich custom firmware with many improvements",
        url: "https://momentum-fw.dev/",
        github: "Next-Flip/Momentum-Firmware",
    },
    CustomFirmware {
        name: "Xtreme",
        description: "Extended features and customizations",
        url: "https://flipper-xtre.me/",
        github: "Flipper-XFW/Xtreme-Firmware",
    },
    CustomFirmware {
        name: "Unleashed",
        description: "Unlocked frequencies and additional features",
        url: "https://github.com/DarkFlippers/unleashed-firmware",
        github: "DarkFlippers/unleashed-firmware",
    },
    CustomFirmware {
        name: "RogueMaster",
        description: "Combined features from multiple firmware sources",
        url: "https://github.com/RogueMaster/flipperzero-firmware-wPlugins",
        github: "RogueMaster/flipperzero-firmware-wPlugins",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parse() {
        let v = FirmwareVersion::parse("0.99.1").unwrap();
        assert_eq!(v.version, "0.99.1");
        assert_eq!(v.channel, FirmwareChannel::Release);
        
        let v = FirmwareVersion::parse("0.99.1-rc").unwrap();
        assert_eq!(v.channel, FirmwareChannel::ReleaseCandidate);
        
        let v = FirmwareVersion::parse("0.99.1-dev").unwrap();
        assert_eq!(v.channel, FirmwareChannel::Dev);
    }
}
