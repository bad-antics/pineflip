//! Firmware Management View

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

/// Firmware management view
#[derive(Clone)]
pub struct FirmwareView {
    pub container: gtk4::Box,
    current_version: gtk4::Label,
    channel_dropdown: gtk4::DropDown,
    update_button: gtk4::Button,
    progress_bar: gtk4::ProgressBar,
}

impl FirmwareView {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);
        container.set_margin_start(24);
        container.set_margin_end(24);
        
        // Header
        let title = gtk4::Label::new(Some("Firmware Management"));
        title.add_css_class("title-1");
        title.set_halign(gtk4::Align::Start);
        container.append(&title);
        
        // Current firmware info card
        let info_group = adw::PreferencesGroup::new();
        info_group.set_title("Current Firmware");
        
        let version_row = adw::ActionRow::builder()
            .title("Version")
            .subtitle("Loading...")
            .build();
        version_row.add_prefix(&gtk4::Image::from_icon_name("emblem-system-symbolic"));
        let current_version = gtk4::Label::new(Some("0.0.0"));
        current_version.add_css_class("accent");
        version_row.add_suffix(&current_version);
        info_group.add(&version_row);
        
        let branch_row = adw::ActionRow::builder()
            .title("Branch")
            .subtitle("Official firmware channel")
            .build();
        branch_row.add_prefix(&gtk4::Image::from_icon_name("system-software-update-symbolic"));
        let branch_label = gtk4::Label::new(Some("release"));
        branch_row.add_suffix(&branch_label);
        info_group.add(&branch_row);
        
        let target_row = adw::ActionRow::builder()
            .title("Target")
            .subtitle("Hardware revision")
            .build();
        target_row.add_prefix(&gtk4::Image::from_icon_name("computer-symbolic"));
        let target_label = gtk4::Label::new(Some("f7"));
        target_row.add_suffix(&target_label);
        info_group.add(&target_row);
        
        container.append(&info_group);
        
        // Update options card
        let update_group = adw::PreferencesGroup::new();
        update_group.set_title("Update Options");
        
        // Firmware channel selection
        let channel_row = adw::ActionRow::builder()
            .title("Update Channel")
            .subtitle("Select firmware source")
            .build();
        channel_row.add_prefix(&gtk4::Image::from_icon_name("network-server-symbolic"));
        
        let channels = ["Official Release", "Official Dev", "Unleashed", "RogueMaster", "Custom DFU"];
        let channel_dropdown = gtk4::DropDown::from_strings(&channels);
        channel_dropdown.set_valign(gtk4::Align::Center);
        channel_row.add_suffix(&channel_dropdown);
        update_group.add(&channel_row);
        
        // Custom URL entry (for custom channel)
        let custom_row = adw::EntryRow::builder()
            .title("Custom URL")
            .build();
        custom_row.set_sensitive(false);
        update_group.add(&custom_row);
        
        container.append(&update_group);
        
        // Update actions
        let actions_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        actions_box.set_halign(gtk4::Align::Center);
        
        // Progress bar (hidden by default)
        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.set_visible(false);
        progress_bar.set_show_text(true);
        progress_bar.set_width_request(400);
        actions_box.append(&progress_bar);
        
        // Action buttons
        let buttons_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        buttons_box.set_halign(gtk4::Align::Center);
        
        let check_button = gtk4::Button::builder()
            .label("Check for Updates")
            .build();
        buttons_box.append(&check_button);
        
        let update_button = gtk4::Button::builder()
            .label("Update Firmware")
            .css_classes(vec!["suggested-action"])
            .build();
        buttons_box.append(&update_button);
        
        actions_box.append(&buttons_box);
        
        // Advanced options
        let advanced_buttons = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        advanced_buttons.set_halign(gtk4::Align::Center);
        advanced_buttons.set_margin_top(12);
        
        let install_local = gtk4::Button::builder()
            .label("Install Local File")
            .build();
        advanced_buttons.append(&install_local);
        
        let repair_button = gtk4::Button::builder()
            .label("Repair / Recovery")
            .css_classes(vec!["destructive-action"])
            .build();
        advanced_buttons.append(&repair_button);
        
        actions_box.append(&advanced_buttons);
        container.append(&actions_box);
        
        // Firmware history card
        let history_group = adw::PreferencesGroup::new();
        history_group.set_title("Recent Updates");
        
        let history_items = [
            ("0.98.3", "2026-01-15", "success"),
            ("0.98.2", "2026-01-10", "success"),
            ("0.98.1", "2025-12-28", "success"),
        ];
        
        for (version, date, _status) in history_items {
            let row = adw::ActionRow::builder()
                .title(version)
                .subtitle(date)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
            history_group.add(&row);
        }
        
        container.append(&history_group);
        
        // Danger zone
        let danger_group = adw::PreferencesGroup::new();
        danger_group.set_title("Danger Zone");
        
        let dfu_row = adw::ActionRow::builder()
            .title("Enter DFU Mode")
            .subtitle("Reboot into Device Firmware Upgrade mode")
            .build();
        dfu_row.add_prefix(&gtk4::Image::from_icon_name("dialog-warning-symbolic"));
        let dfu_button = gtk4::Button::builder()
            .label("Reboot to DFU")
            .css_classes(vec!["destructive-action"])
            .valign(gtk4::Align::Center)
            .build();
        dfu_row.add_suffix(&dfu_button);
        danger_group.add(&dfu_row);
        
        let reboot_row = adw::ActionRow::builder()
            .title("Reboot Device")
            .subtitle("Restart the Flipper Zero")
            .build();
        reboot_row.add_prefix(&gtk4::Image::from_icon_name("system-reboot-symbolic"));
        let reboot_button = gtk4::Button::builder()
            .label("Reboot")
            .valign(gtk4::Align::Center)
            .build();
        reboot_row.add_suffix(&reboot_button);
        danger_group.add(&reboot_row);
        
        container.append(&danger_group);
        
        Self {
            container,
            current_version,
            channel_dropdown,
            update_button,
            progress_bar,
        }
    }
    
    /// Update current firmware version display
    pub fn set_firmware_version(&self, version: &str) {
        self.current_version.set_text(version);
    }
    
    /// Show update progress
    pub fn show_progress(&self, fraction: f64, text: &str) {
        self.progress_bar.set_visible(true);
        self.progress_bar.set_fraction(fraction);
        self.progress_bar.set_text(Some(text));
    }
    
    /// Hide progress bar
    pub fn hide_progress(&self) {
        self.progress_bar.set_visible(false);
    }
}
