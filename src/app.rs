//! Main Application Window

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::device::FlipperDevice;
use crate::ui;

/// Main application window
pub struct PineFlipWindow {
    pub window: adw::ApplicationWindow,
    pub device: Arc<RwLock<Option<FlipperDevice>>>,
    pub stack: gtk4::Stack,
    pub status_page: adw::StatusPage,
    pub main_view: gtk4::Box,
    pub screen_view: ui::ScreenView,
    pub files_view: ui::FilesView,
    pub firmware_view: ui::FirmwareView,
    pub settings_view: ui::SettingsView,
}

impl PineFlipWindow {
    pub fn new(app: &adw::Application) -> Rc<Self> {
        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("PineFlip")
            .default_width(1200)
            .default_height(800)
            .build();
        
        // Apply dark theme
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::PreferDark);
        
        // Create header bar
        let header = adw::HeaderBar::new();
        
        // Menu button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .build();
        
        let menu = gio::Menu::new();
        menu.append(Some("About"), Some("app.about"));
        menu.append(Some("Quit"), Some("app.quit"));
        menu_button.set_menu_model(Some(&menu));
        header.pack_end(&menu_button);
        
        // Connection status indicator
        let connection_status = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let status_icon = gtk4::Image::from_icon_name("network-offline-symbolic");
        let status_label = gtk4::Label::new(Some("Not Connected"));
        status_label.add_css_class("dim-label");
        connection_status.append(&status_icon);
        connection_status.append(&status_label);
        header.pack_start(&connection_status);
        
        // Main content stack
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);
        
        // Status page (shown when no device connected)
        let status_page = adw::StatusPage::builder()
            .icon_name("flipper-zero-symbolic")
            .title("Connect Your Flipper Zero")
            .description("Plug in your Flipper Zero via USB to get started")
            .build();
        
        let connect_button = gtk4::Button::builder()
            .label("Scan for Devices")
            .halign(gtk4::Align::Center)
            .css_classes(vec!["suggested-action", "pill"])
            .build();
        status_page.set_child(Some(&connect_button));
        
        // Create views
        let screen_view = ui::ScreenView::new();
        let files_view = ui::FilesView::new();
        let firmware_view = ui::FirmwareView::new();
        let settings_view = ui::SettingsView::new();
        
        // Main connected view with navigation
        let main_view = Self::create_main_view(
            &screen_view,
            &files_view,
            &firmware_view,
            &settings_view,
        );
        
        stack.add_named(&status_page, Some("disconnected"));
        stack.add_named(&main_view, Some("connected"));
        stack.set_visible_child_name("disconnected");
        
        // Main layout
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.append(&header);
        main_box.append(&stack);
        
        window.set_content(Some(&main_box));
        
        let device = Arc::new(RwLock::new(None));
        
        let this = Rc::new(Self {
            window,
            device,
            stack,
            status_page,
            main_view,
            screen_view,
            files_view,
            firmware_view,
            settings_view,
        });
        
        // Set up actions
        Self::setup_actions(&this, app);
        
        // Set up device detection
        Self::setup_device_detection(Rc::clone(&this));
        
        // Connect button handler
        let this_clone = Rc::clone(&this);
        connect_button.connect_clicked(move |_| {
            this_clone.scan_for_devices();
        });
        
        this
    }
    
    fn create_main_view(
        screen_view: &ui::ScreenView,
        files_view: &ui::FilesView,
        firmware_view: &ui::FirmwareView,
        settings_view: &ui::SettingsView,
    ) -> gtk4::Box {
        let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        
        // Sidebar navigation
        let sidebar = gtk4::ListBox::new();
        sidebar.set_selection_mode(gtk4::SelectionMode::Single);
        sidebar.add_css_class("navigation-sidebar");
        sidebar.set_size_request(220, -1);
        
        // Navigation items
        let nav_items = [
            ("video-display-symbolic", "Screen Mirror", "screen"),
            ("folder-symbolic", "File Manager", "files"),
            ("emblem-system-symbolic", "Firmware", "firmware"),
            ("applications-system-symbolic", "Settings", "settings"),
        ];
        
        for (icon, label, _) in &nav_items {
            let row = adw::ActionRow::builder()
                .title(*label)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(*icon));
            sidebar.append(&row);
        }
        
        // Content stack
        let content_stack = gtk4::Stack::new();
        content_stack.set_hexpand(true);
        content_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        
        content_stack.add_named(&screen_view.container, Some("screen"));
        content_stack.add_named(&files_view.container, Some("files"));
        content_stack.add_named(&firmware_view.container, Some("firmware"));
        content_stack.add_named(&settings_view.container, Some("settings"));
        content_stack.set_visible_child_name("screen");
        
        // Sidebar selection handler
        let content_stack_clone = content_stack.clone();
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let idx = row.index();
                let name = match idx {
                    0 => "screen",
                    1 => "files",
                    2 => "firmware",
                    3 => "settings",
                    _ => "screen",
                };
                content_stack_clone.set_visible_child_name(name);
            }
        });
        
        // Select first row
        if let Some(first_row) = sidebar.row_at_index(0) {
            sidebar.select_row(Some(&first_row));
        }
        
        // Sidebar container with separator
        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let sidebar_scroll = gtk4::ScrolledWindow::builder()
            .child(&sidebar)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        sidebar_box.append(&sidebar_scroll);
        sidebar_box.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
        
        main_box.append(&sidebar_box);
        main_box.append(&content_stack);
        
        main_box
    }
    
    fn setup_actions(this: &Rc<Self>, app: &adw::Application) {
        use gio::prelude::*;
        
        // About action
        let about_action = gio::SimpleAction::new("about", None);
        let window = this.window.clone();
        about_action.connect_activate(move |_, _| {
            let about = adw::AboutDialog::builder()
                .application_name("PineFlip")
                .application_icon("flipper-zero-symbolic")
                .developer_name("bad-antics")
                .version(env!("CARGO_PKG_VERSION"))
                .website("https://github.com/bad-antics/pineflip")
                .issue_url("https://github.com/bad-antics/pineflip/issues")
                .license_type(gtk4::License::Gpl30)
                .comments("Professional Flipper Zero companion application")
                .build();
            about.present(Some(&window));
        });
        app.add_action(&about_action);
        
        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        let app_clone = app.clone();
        quit_action.connect_activate(move |_, _| {
            app_clone.quit();
        });
        app.add_action(&quit_action);
    }
    
    fn setup_device_detection(this: Rc<Self>) {
        // Poll for device connection every 2 seconds
        let this_clone = Rc::clone(&this);
        glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
            this_clone.check_device_connection();
            glib::ControlFlow::Continue
        });
    }
    
    pub fn scan_for_devices(&self) {
        tracing::info!("Scanning for Flipper Zero devices...");
        
        // Spawn async detection
        let device = Arc::clone(&self.device);
        let stack = self.stack.clone();
        let screen_view = self.screen_view.clone();
        
        glib::spawn_future_local(async move {
            match FlipperDevice::auto_detect().await {
                Ok(flipper) => {
                    tracing::info!("Found device: {}", flipper.name());
                    
                    // Store device
                    *device.write().await = Some(flipper);
                    
                    // Switch to connected view
                    stack.set_visible_child_name("connected");
                    
                    // Start screen mirror
                    screen_view.start_mirror(Arc::clone(&device));
                }
                Err(e) => {
                    tracing::warn!("No device found: {}", e);
                }
            }
        });
    }
    
    fn check_device_connection(&self) {
        // Check if device is still connected
        let device = Arc::clone(&self.device);
        let stack = self.stack.clone();
        
        glib::spawn_future_local(async move {
            let device_guard = device.read().await;
            if let Some(ref dev) = *device_guard {
                if !dev.is_connected().await {
                    drop(device_guard);
                    *device.write().await = None;
                    stack.set_visible_child_name("disconnected");
                }
            }
        });
    }
    
    pub fn start_mirror(&self) {
        self.stack.set_visible_child_name("connected");
        self.screen_view.start_mirror(Arc::clone(&self.device));
    }
    
    pub fn present(&self) {
        self.window.present();
    }
}

use gio;
