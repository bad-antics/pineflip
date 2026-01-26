//! File Manager View

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::RwLock;

use crate::device::FlipperDevice;

/// File manager view
#[derive(Clone)]
pub struct FilesView {
    pub container: gtk4::Box,
    file_list: gtk4::ListBox,
    path_bar: gtk4::Entry,
    current_path: Rc<RefCell<String>>,
    storage_bar: gtk4::ProgressBar,
}

impl FilesView {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        // Toolbar
        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        toolbar.set_margin_top(12);
        toolbar.set_margin_bottom(12);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);
        
        // Navigation buttons
        let back_btn = gtk4::Button::from_icon_name("go-previous-symbolic");
        back_btn.set_tooltip_text(Some("Back"));
        toolbar.append(&back_btn);
        
        let home_btn = gtk4::Button::from_icon_name("go-home-symbolic");
        home_btn.set_tooltip_text(Some("Home"));
        toolbar.append(&home_btn);
        
        let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Refresh"));
        toolbar.append(&refresh_btn);
        
        // Path bar
        let path_bar = gtk4::Entry::builder()
            .text("/ext")
            .hexpand(true)
            .build();
        toolbar.append(&path_bar);
        
        // Storage selector
        let storage_dropdown = gtk4::DropDown::from_strings(&["Internal (/int)", "SD Card (/ext)"]);
        storage_dropdown.set_selected(1); // Default to SD
        toolbar.append(&storage_dropdown);
        
        container.append(&toolbar);
        container.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
        
        // Split view: folder tree + file list
        let paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        paned.set_position(200);
        
        // Folder tree (left side)
        let folder_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        let folder_tree = gtk4::TreeView::new();
        folder_scroll.set_child(Some(&folder_tree));
        paned.set_start_child(Some(&folder_scroll));
        
        // File list (right side)
        let file_scroll = gtk4::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        
        let file_list = gtk4::ListBox::new();
        file_list.set_selection_mode(gtk4::SelectionMode::Multiple);
        file_list.add_css_class("boxed-list");
        file_scroll.set_child(Some(&file_list));
        
        // Add some placeholder items
        Self::add_placeholder_files(&file_list);
        
        paned.set_end_child(Some(&file_scroll));
        container.append(&paned);
        
        // Status bar
        let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        status_bar.set_margin_top(6);
        status_bar.set_margin_bottom(6);
        status_bar.set_margin_start(12);
        status_bar.set_margin_end(12);
        
        let status_label = gtk4::Label::new(Some("0 items"));
        status_label.add_css_class("dim-label");
        status_bar.append(&status_label);
        
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        status_bar.append(&spacer);
        
        // Storage usage bar
        let storage_label = gtk4::Label::new(Some("Storage:"));
        storage_label.add_css_class("dim-label");
        status_bar.append(&storage_label);
        
        let storage_bar = gtk4::ProgressBar::new();
        storage_bar.set_fraction(0.65);
        storage_bar.set_text(Some("32 GB / 50 GB"));
        storage_bar.set_show_text(true);
        storage_bar.set_width_request(200);
        status_bar.append(&storage_bar);
        
        container.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
        container.append(&status_bar);
        
        let current_path = Rc::new(RefCell::new("/ext".to_string()));
        
        Self {
            container,
            file_list,
            path_bar,
            current_path,
            storage_bar,
        }
    }
    
    fn add_placeholder_files(list: &gtk4::ListBox) {
        let files = [
            ("folder-symbolic", "apps", "Directory", true),
            ("folder-symbolic", "badusb", "Directory", true),
            ("folder-symbolic", "infrared", "Directory", true),
            ("folder-symbolic", "nfc", "Directory", true),
            ("folder-symbolic", "subghz", "Directory", true),
            ("folder-symbolic", "music_player", "Directory", true),
            ("text-x-generic-symbolic", "README.txt", "1.2 KB", false),
        ];
        
        for (icon, name, info, _is_dir) in files {
            let row = adw::ActionRow::builder()
                .title(name)
                .subtitle(info)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));
            
            // Add context menu
            let menu = gio::Menu::new();
            menu.append(Some("Open"), None);
            menu.append(Some("Download"), None);
            menu.append(Some("Rename"), None);
            menu.append(Some("Delete"), None);
            
            list.append(&row);
        }
    }
    
    /// Load directory contents
    pub fn load_directory(&self, device: Arc<RwLock<Option<FlipperDevice>>>, path: &str) {
        let file_list = self.file_list.clone();
        let path_bar = self.path_bar.clone();
        let current_path = Rc::clone(&self.current_path);
        let path = path.to_string();
        
        glib::spawn_future_local(async move {
            let device_guard = device.read().await;
            if let Some(ref dev) = *device_guard {
                match dev.list_directory(&path).await {
                    Ok(files) => {
                        // Clear existing items
                        while let Some(row) = file_list.row_at_index(0) {
                            file_list.remove(&row);
                        }
                        
                        // Add new items
                        for file in files {
                            let icon = if file.is_directory {
                                "folder-symbolic"
                            } else {
                                "text-x-generic-symbolic"
                            };
                            
                            let size_str = if file.is_directory {
                                "Directory".to_string()
                            } else {
                                format_size(file.size)
                            };
                            
                            let row = adw::ActionRow::builder()
                                .title(&file.name)
                                .subtitle(&size_str)
                                .build();
                            row.add_prefix(&gtk4::Image::from_icon_name(icon));
                            file_list.append(&row);
                        }
                        
                        // Update path bar
                        path_bar.set_text(&path);
                        *current_path.borrow_mut() = path;
                    }
                    Err(e) => {
                        tracing::error!("Failed to list directory: {}", e);
                    }
                }
            }
        });
    }
    
    /// Navigate to parent directory
    pub fn navigate_up(&self, device: Arc<RwLock<Option<FlipperDevice>>>) {
        let current = self.current_path.borrow().clone();
        if let Some(parent) = std::path::Path::new(&current).parent() {
            let parent_str = parent.to_string_lossy().to_string();
            if !parent_str.is_empty() {
                self.load_directory(device, &parent_str);
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

use gio;
