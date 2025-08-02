use eframe::{egui, Frame};
use egui::Context;
use std::sync::{Arc, Mutex};
use crate::camera::list_devices::VideoDevice;
use crate::ui::state::AppState;

pub fn build_ui(devices: Vec<VideoDevice>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let first_device = devices.get(0).unwrap_or(&VideoDevice{ id: 0, name: "".to_owned() }).id;
    eframe::run_native(
        "Live Video Processor",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(AppState {
                available_devices: devices,
                delay_sec: 5,
                selected_device: first_device,
                current_frame: Arc::new(Mutex::new(None)),
                texture_handle: None,
                is_streaming: false,
                stream_handle: None,
                show_video: false,
            }))
        }),
    )
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.show_video {
                self.show_settings(ctx, ui);
            } else {
                self.show_stream(ctx, ui);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_streaming();
    }
}