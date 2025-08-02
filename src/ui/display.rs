use egui::{Context, Ui};
use eframe::emath::Vec2;
use crate::ui::state::AppState;

impl AppState {
    pub fn show_stream(&mut self, ctx: &Context, ui: &mut Ui) {
        // Video display UI
        ui.horizontal(|ui| {
            ui.heading(&format!("Video Display - Device {}", self.selected_device));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Back to Settings").clicked() {
                    self.show_video = false;
                    self.stop_streaming();
                }
            });
        });

        ui.label(&format!("Delay: {} seconds", self.delay_sec));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Stop Stream").clicked() && self.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                self.stop_streaming();
                self.stop_streaming();
            }

            ui.label(if self.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                "Status: Streaming"
            } else {
                "Status: Stopped"
            });
        });

        ui.separator();

        // Display the current frame
        if let Ok(frame_guard) = self.current_frame.lock() {
            if let Some(ref frame) = *frame_guard {
                let color_image = Self::frame_to_color_image(frame);

                // Create or update texture
                let texture = self.texture_handle.get_or_insert_with(|| {
                    ctx.load_texture("video_frame", color_image.clone(), Default::default())
                });

                // Update texture with new frame data
                texture.set(color_image, Default::default());

                // Calculate display size while maintaining aspect ratio
                let available_size = ui.available_size();
                let image_aspect = frame.width as f32 / frame.height as f32;
                let display_size = if available_size.x / available_size.y > image_aspect {
                    // Window is wider than image aspect ratio
                    Vec2::new(available_size.y * image_aspect, available_size.y * 0.8)
                } else {
                    // Window is taller than image aspect ratio
                    Vec2::new(available_size.x * 0.8, available_size.x / image_aspect)
                };

                // Display the image
                ui.add(egui::Image::from_texture(&*texture).fit_to_exact_size(display_size));

                // Show frame info
                ui.label(&format!(
                    "Frame: {}x{} | Time: {:?}",
                    frame.width,
                    frame.height,
                    frame.timestamp.elapsed().unwrap_or_default()
                ));
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Waiting for video frames...");
                    if !self.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                        ui.label("Stream stopped.");
                    }
                });
            }
        }
    }
}