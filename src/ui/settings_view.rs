//! Settings View

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

/// Settings view
#[derive(Clone)]
pub struct SettingsView {
    pub container: gtk4::Box,
}

impl SettingsView {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);
        container.set_margin_start(24);
        container.set_margin_end(24);
        
        // Header
        let title = gtk4::Label::new(Some("Settings"));
        title.add_css_class("title-1");
        title.set_halign(gtk4::Align::Start);
        container.append(&title);
        
        // Appearance settings
        let appearance_group = adw::PreferencesGroup::new();
        appearance_group.set_title("Appearance");
        
        // Theme selection
        let theme_row = adw::ActionRow::builder()
            .title("Color Scheme")
            .subtitle("Application theme")
            .build();
        theme_row.add_prefix(&gtk4::Image::from_icon_name("preferences-desktop-appearance-symbolic"));
        let theme_dropdown = gtk4::DropDown::from_strings(&["System", "Light", "Dark"]);
        theme_dropdown.set_selected(2); // Default dark
        theme_dropdown.set_valign(gtk4::Align::Center);
        theme_row.add_suffix(&theme_dropdown);
        appearance_group.add(&theme_row);
        
        // Screen scale
        let scale_row = adw::ActionRow::builder()
            .title("Screen Scale")
            .subtitle("Mirror display scale factor")
            .build();
        scale_row.add_prefix(&gtk4::Image::from_icon_name("zoom-in-symbolic"));
        let scale_dropdown = gtk4::DropDown::from_strings(&["2x", "3x", "4x", "5x", "6x"]);
        scale_dropdown.set_selected(2); // Default 4x
        scale_dropdown.set_valign(gtk4::Align::Center);
        scale_row.add_suffix(&scale_dropdown);
        appearance_group.add(&scale_row);
        
        // Screen color
        let color_row = adw::ActionRow::builder()
            .title("Screen Color")
            .subtitle("Flipper display emulation color")
            .build();
        color_row.add_prefix(&gtk4::Image::from_icon_name("preferences-color-symbolic"));
        let color_dropdown = gtk4::DropDown::from_strings(&["Orange (Default)", "Green", "Blue", "White"]);
        color_dropdown.set_valign(gtk4::Align::Center);
        color_row.add_suffix(&color_dropdown);
        appearance_group.add(&color_row);
        
        container.append(&appearance_group);
        
        // Connection settings
        let connection_group = adw::PreferencesGroup::new();
        connection_group.set_title("Connection");
        
        // Auto-connect
        let autoconnect_row = adw::SwitchRow::builder()
            .title("Auto-connect")
            .subtitle("Automatically connect when device is plugged in")
            .active(true)
            .build();
        autoconnect_row.add_prefix(&gtk4::Image::from_icon_name("network-wired-symbolic"));
        connection_group.add(&autoconnect_row);
        
        // Connection timeout
        let timeout_row = adw::SpinRow::with_range(1.0, 30.0, 1.0);
        timeout_row.set_title("Connection Timeout");
        timeout_row.set_subtitle("Seconds to wait for device response");
        timeout_row.set_value(5.0);
        timeout_row.add_prefix(&gtk4::Image::from_icon_name("preferences-system-time-symbolic"));
        connection_group.add(&timeout_row);
        
        // USB rules
        let rules_row = adw::ActionRow::builder()
            .title("USB Rules")
            .subtitle("Install udev rules for non-root access")
            .build();
        rules_row.add_prefix(&gtk4::Image::from_icon_name("dialog-password-symbolic"));
        let rules_button = gtk4::Button::builder()
            .label("Install")
            .valign(gtk4::Align::Center)
            .build();
        rules_row.add_suffix(&rules_button);
        connection_group.add(&rules_row);
        
        container.append(&connection_group);
        
        // Screen capture settings
        let capture_group = adw::PreferencesGroup::new();
        capture_group.set_title("Screen Capture");
        
        // Frame rate
        let fps_row = adw::ActionRow::builder()
            .title("Refresh Rate")
            .subtitle("Screen mirror update frequency")
            .build();
        fps_row.add_prefix(&gtk4::Image::from_icon_name("video-display-symbolic"));
        let fps_dropdown = gtk4::DropDown::from_strings(&["15 FPS", "30 FPS", "60 FPS"]);
        fps_dropdown.set_selected(1); // Default 30
        fps_dropdown.set_valign(gtk4::Align::Center);
        fps_row.add_suffix(&fps_dropdown);
        capture_group.add(&fps_row);
        
        // Screenshot format
        let format_row = adw::ActionRow::builder()
            .title("Screenshot Format")
            .subtitle("Image format for screenshots")
            .build();
        format_row.add_prefix(&gtk4::Image::from_icon_name("image-x-generic-symbolic"));
        let format_dropdown = gtk4::DropDown::from_strings(&["PNG", "JPEG", "WebP", "BMP"]);
        format_dropdown.set_valign(gtk4::Align::Center);
        format_row.add_suffix(&format_dropdown);
        capture_group.add(&format_row);
        
        // Recording settings
        let recording_row = adw::ActionRow::builder()
            .title("Recording Format")
            .subtitle("Video format for screen recordings")
            .build();
        recording_row.add_prefix(&gtk4::Image::from_icon_name("media-record-symbolic"));
        let recording_dropdown = gtk4::DropDown::from_strings(&["GIF", "MP4", "WebM"]);
        recording_dropdown.set_valign(gtk4::Align::Center);
        recording_row.add_suffix(&recording_dropdown);
        capture_group.add(&recording_row);
        
        // Screenshot directory
        let dir_row = adw::ActionRow::builder()
            .title("Save Location")
            .subtitle("~/Pictures/PineFlip")
            .build();
        dir_row.add_prefix(&gtk4::Image::from_icon_name("folder-pictures-symbolic"));
        let dir_button = gtk4::Button::builder()
            .label("Choose")
            .valign(gtk4::Align::Center)
            .build();
        dir_row.add_suffix(&dir_button);
        capture_group.add(&dir_row);
        
        container.append(&capture_group);
        
        // File manager settings
        let files_group = adw::PreferencesGroup::new();
        files_group.set_title("File Manager");
        
        // Default storage
        let storage_row = adw::ActionRow::builder()
            .title("Default Storage")
            .subtitle("Storage to browse on connect")
            .build();
        storage_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));
        let storage_dropdown = gtk4::DropDown::from_strings(&["SD Card (/ext)", "Internal (/int)"]);
        storage_dropdown.set_valign(gtk4::Align::Center);
        storage_row.add_suffix(&storage_dropdown);
        files_group.add(&storage_row);
        
        // Show hidden files
        let hidden_row = adw::SwitchRow::builder()
            .title("Show Hidden Files")
            .subtitle("Display files starting with a dot")
            .active(false)
            .build();
        hidden_row.add_prefix(&gtk4::Image::from_icon_name("view-reveal-symbolic"));
        files_group.add(&hidden_row);
        
        // Confirm delete
        let confirm_row = adw::SwitchRow::builder()
            .title("Confirm Delete")
            .subtitle("Ask before deleting files")
            .active(true)
            .build();
        confirm_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        files_group.add(&confirm_row);
        
        container.append(&files_group);
        
        // Keyboard shortcuts
        let shortcuts_group = adw::PreferencesGroup::new();
        shortcuts_group.set_title("Keyboard Shortcuts");
        
        let shortcuts = [
            ("Arrow Keys", "D-pad navigation"),
            ("Enter", "OK button"),
            ("Escape", "Back button"),
            ("Ctrl+S", "Take screenshot"),
            ("Ctrl+R", "Toggle recording"),
            ("F11", "Toggle fullscreen"),
        ];
        
        for (key, action) in shortcuts {
            let row = adw::ActionRow::builder()
                .title(action)
                .build();
            let key_label = gtk4::Label::new(Some(key));
            key_label.add_css_class("dim-label");
            row.add_suffix(&key_label);
            shortcuts_group.add(&row);
        }
        
        container.append(&shortcuts_group);
        
        // Data & Privacy
        let data_group = adw::PreferencesGroup::new();
        data_group.set_title("Data & Privacy");
        
        let logs_row = adw::ActionRow::builder()
            .title("Debug Logging")
            .subtitle("Enable verbose logging for troubleshooting")
            .build();
        logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        let logs_switch = gtk4::Switch::new();
        logs_switch.set_valign(gtk4::Align::Center);
        logs_row.add_suffix(&logs_switch);
        data_group.add(&logs_row);
        
        let clear_row = adw::ActionRow::builder()
            .title("Clear Cache")
            .subtitle("Remove cached firmware and temporary files")
            .build();
        clear_row.add_prefix(&gtk4::Image::from_icon_name("edit-clear-all-symbolic"));
        let clear_button = gtk4::Button::builder()
            .label("Clear")
            .css_classes(vec!["destructive-action"])
            .valign(gtk4::Align::Center)
            .build();
        clear_row.add_suffix(&clear_button);
        data_group.add(&clear_row);
        
        container.append(&data_group);
        
        // Wrap in scrolled window
        let scroll = gtk4::ScrolledWindow::builder()
            .child(&container)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        
        let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        outer.append(&scroll);
        
        Self {
            container: outer,
        }
    }
}
