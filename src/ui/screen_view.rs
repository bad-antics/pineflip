//! Screen Mirroring View

use gtk4::prelude::*;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::RwLock;

use crate::device::{FlipperDevice, ScreenFrame};

/// Scale factor for the screen display
const SCREEN_SCALE: u32 = 4;

/// Screen mirroring view
#[derive(Clone)]
pub struct ScreenView {
    pub container: gtk4::Box,
    drawing_area: gtk4::DrawingArea,
    frame_buffer: Rc<RefCell<Option<ScreenFrame>>>,
    controls_enabled: Rc<RefCell<bool>>,
    recording: Rc<RefCell<bool>>,
}

impl ScreenView {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        container.set_margin_top(24);
        container.set_margin_bottom(24);
        container.set_margin_start(24);
        container.set_margin_end(24);
        
        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let title = gtk4::Label::new(Some("Screen Mirror"));
        title.add_css_class("title-1");
        header.append(&title);
        
        // Spacer
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);
        
        // Recording button
        let record_button = gtk4::ToggleButton::builder()
            .icon_name("media-record-symbolic")
            .tooltip_text("Record Screen")
            .build();
        header.append(&record_button);
        
        // Screenshot button
        let screenshot_button = gtk4::Button::builder()
            .icon_name("camera-photo-symbolic")
            .tooltip_text("Take Screenshot")
            .build();
        header.append(&screenshot_button);
        
        // Fullscreen button
        let fullscreen_button = gtk4::Button::builder()
            .icon_name("view-fullscreen-symbolic")
            .tooltip_text("Fullscreen")
            .build();
        header.append(&fullscreen_button);
        
        container.append(&header);
        
        // Screen display area
        let screen_frame = gtk4::Frame::new(None);
        screen_frame.add_css_class("view");
        screen_frame.set_halign(gtk4::Align::Center);
        
        let drawing_area = gtk4::DrawingArea::new();
        let scaled_width = (ScreenFrame::WIDTH * SCREEN_SCALE) as i32;
        let scaled_height = (ScreenFrame::HEIGHT * SCREEN_SCALE) as i32;
        drawing_area.set_size_request(scaled_width, scaled_height);
        drawing_area.set_content_width(scaled_width);
        drawing_area.set_content_height(scaled_height);
        
        // Frame buffer for rendering
        let frame_buffer: Rc<RefCell<Option<ScreenFrame>>> = Rc::new(RefCell::new(None));
        
        // Set up drawing
        let frame_buffer_clone = Rc::clone(&frame_buffer);
        drawing_area.set_draw_func(move |_, cr, width, height| {
            // Black background
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = cr.fill();
            
            // Draw frame if available
            if let Some(ref frame) = *frame_buffer_clone.borrow() {
                let scale_x = width as f64 / frame.width as f64;
                let scale_y = height as f64 / frame.height as f64;
                
                // Orange for "on" pixels (Flipper screen color)
                cr.set_source_rgb(1.0, 0.5, 0.0);
                
                for y in 0..frame.height {
                    for x in 0..frame.width {
                        if frame.get_pixel(x, y) {
                            cr.rectangle(
                                x as f64 * scale_x,
                                y as f64 * scale_y,
                                scale_x,
                                scale_y,
                            );
                        }
                    }
                }
                let _ = cr.fill();
            } else {
                // Draw placeholder text
                cr.set_source_rgb(0.4, 0.4, 0.4);
                cr.set_font_size(20.0);
                cr.move_to(width as f64 / 2.0 - 80.0, height as f64 / 2.0);
                let _ = cr.show_text("No Signal");
            }
        });
        
        screen_frame.set_child(Some(&drawing_area));
        container.append(&screen_frame);
        
        // Controls section
        let controls_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        controls_box.set_halign(gtk4::Align::Center);
        controls_box.set_margin_top(24);
        
        let controls_label = gtk4::Label::new(Some("Controls"));
        controls_label.add_css_class("title-4");
        controls_box.append(&controls_label);
        
        // D-pad style controls
        let dpad = gtk4::Grid::new();
        dpad.set_row_spacing(6);
        dpad.set_column_spacing(6);
        dpad.set_halign(gtk4::Align::Center);
        
        let btn_up = Self::create_control_button("go-up-symbolic", "Up");
        let btn_down = Self::create_control_button("go-down-symbolic", "Down");
        let btn_left = Self::create_control_button("go-previous-symbolic", "Left");
        let btn_right = Self::create_control_button("go-next-symbolic", "Right");
        let btn_ok = Self::create_control_button("emblem-ok-symbolic", "OK");
        let btn_back = Self::create_control_button("go-previous-symbolic", "Back");
        btn_back.add_css_class("destructive-action");
        
        dpad.attach(&btn_up, 1, 0, 1, 1);
        dpad.attach(&btn_left, 0, 1, 1, 1);
        dpad.attach(&btn_ok, 1, 1, 1, 1);
        dpad.attach(&btn_right, 2, 1, 1, 1);
        dpad.attach(&btn_down, 1, 2, 1, 1);
        dpad.attach(&btn_back, 3, 1, 1, 1);
        
        controls_box.append(&dpad);
        
        // Keyboard hint
        let hint = gtk4::Label::new(Some("Use arrow keys, Enter, and Escape to control"));
        hint.add_css_class("dim-label");
        controls_box.append(&hint);
        
        container.append(&controls_box);
        
        let controls_enabled = Rc::new(RefCell::new(true));
        let recording = Rc::new(RefCell::new(false));
        
        Self {
            container,
            drawing_area,
            frame_buffer,
            controls_enabled,
            recording,
        }
    }
    
    fn create_control_button(icon: &str, tooltip: &str) -> gtk4::Button {
        gtk4::Button::builder()
            .icon_name(icon)
            .tooltip_text(tooltip)
            .width_request(60)
            .height_request(60)
            .build()
    }
    
    /// Start screen mirroring
    pub fn start_mirror(&self, device: Arc<RwLock<Option<FlipperDevice>>>) {
        let drawing_area = self.drawing_area.clone();
        let frame_buffer = Rc::clone(&self.frame_buffer);
        
        // Poll for frames
        glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
            let device = Arc::clone(&device);
            let frame_buffer = Rc::clone(&frame_buffer);
            let drawing_area = drawing_area.clone();
            
            glib::spawn_future_local(async move {
                let device_guard = device.read().await;
                if let Some(ref dev) = *device_guard {
                    if let Ok(frame) = dev.get_screen_frame().await {
                        *frame_buffer.borrow_mut() = Some(frame);
                        drawing_area.queue_draw();
                    }
                }
            });
            
            glib::ControlFlow::Continue
        });
    }
    
    /// Update display with new frame
    pub fn update_frame(&self, frame: ScreenFrame) {
        *self.frame_buffer.borrow_mut() = Some(frame);
        self.drawing_area.queue_draw();
    }
    
    /// Take screenshot
    pub fn take_screenshot(&self) -> Option<Vec<u8>> {
        let frame = self.frame_buffer.borrow();
        frame.as_ref().map(|f| f.to_rgba(SCREEN_SCALE))
    }
}
