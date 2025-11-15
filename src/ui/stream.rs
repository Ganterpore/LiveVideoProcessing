use egui::Context;
use std::sync::{Arc};
use eframe::epaint::ColorImage;
use crate::camera::camera::{CameraFrame, FFmpegCamera};
use crate::ui::state::AppState;

impl AppState {
    pub fn start_streaming(&mut self, ctx: &Context) {
        // Check if already streaming using the handle
        if let Some(handle) = &self.stream_handle {
            if handle.is_active() {
                return;
            }
        }
        println!("Starting video stream for device {}", self.selected_device);

        // First capture a single frame to display
        let camera = FFmpegCamera::new(self.selected_device, 640, 480, 30.00);
        match camera.capture_single_frame() {
            Ok(frame) => {
                println!("Successfully captured test frame: {}x{}, {} bytes",
                         frame.width, frame.height, frame.data.len());

                // Display the test frame immediately
                if let Ok(mut current_frame_guard) = self.current_frame.lock() {
                    *current_frame_guard = Some(frame);
                }
                ctx.request_repaint();
            }
            Err(e) => {
                eprintln!("Failed to capture test frame: {}", e);
                return;
            }
        }

        let frame_data = Arc::clone(&self.current_frame);
        let ctx_clone = ctx.clone();
        let delay_sec = self.delay_sec;

        let mut frame_buffer = std::collections::VecDeque::new();
        let target_buffer_size = if delay_sec == 0 { 0 } else { (delay_sec as f64 * 30.0) as usize };

        match camera.capture_continuous(move |frame| {

            if delay_sec == 0 {
                // No delay - display immediately
                if let Ok(mut current_frame_guard) = frame_data.lock() {
                    *current_frame_guard = Some(frame);
                }
                ctx_clone.request_repaint();
            } else {
                // Add frame to buffer
                frame_buffer.push_back(frame);

                // If we have enough frames for the delay, pop the oldest one to display
                if frame_buffer.len() > target_buffer_size {
                    if let Some(delayed_frame) = frame_buffer.pop_front() {
                        if let Ok(mut current_frame_guard) = frame_data.lock() {
                            *current_frame_guard = Some(delayed_frame);
                        }
                        ctx_clone.request_repaint();
                    }
                }
            }
        }) {
            Ok(handle) => {
                self.stream_handle = Some(handle);
                println!("Stream started successfully");
            }
            Err(e) => {
                eprintln!("Failed to start camera capture: {}", e);
            }
        }
    }

    pub fn stop_streaming(&mut self) {
        println!("Stopping video stream");
        if let Some(handle) = self.stream_handle.take() {
            handle.stop();
            println!("Stream stop signal sent");
        }
    }
    
    pub fn restart_stream(&mut self, ctx: &Context) {
        self.stop_streaming();
        self.start_streaming(ctx);
    }

    pub fn frame_to_color_image(frame: &CameraFrame) -> ColorImage {
        let mut pixels = Vec::with_capacity((frame.width * frame.height) as usize);

        // Convert RGB24 data to RGBA (egui expects RGBA)
        for chunk in frame.data.chunks(3) {
            if chunk.len() == 3 {
                pixels.push(egui::Color32::from_rgb(chunk[0], chunk[1], chunk[2]));
            }
        }

        ColorImage {
            size: [frame.width as usize, frame.height as usize],
            source_size: Default::default(),
            pixels,
        }
    }
}