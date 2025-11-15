use std::time::Duration;
use egui::{Context, Ui};
use eframe::emath::Vec2;
use crate::camera::list_devices::VideoDevice;
use crate::ui::state::AppState;

impl AppState {
    pub fn show_stream(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Avalaible devices
            ui.label("Selected Camera:");
            egui::ComboBox::from_id_salt("Available Devices:")
                .selected_text(&self.available_devices
                    .iter().find(|device| { device.id == self.selected_device })
                    .unwrap_or(&VideoDevice { id: 0, name: "".to_owned() }).name)
                .show_ui(ui, |ui| {
                    let mut should_stop = false;
                    for device in &self.available_devices {
                        let id = device.id;
                        let name = &device.name;
                        if ui.selectable_value(&mut self.selected_device, id, name).changed() {
                            should_stop = true;
                        }
                    }
                    if should_stop {
                        self.restart_stream(ctx);
                    }
                });
            // Stream delay
            ui.label("Delay: ");
            if ui.add(egui::DragValue::new(&mut self.delay_sec).range(0..=120)).changed() {
                self.restart_stream(ctx);
            };
            ui.label("seconds");
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
                    "Frame: {}x{} | Time: {:?}s ago",
                    frame.width,
                    frame.height,
                    frame.timestamp.elapsed().unwrap_or_default().as_secs()
                ));
            }
        }
        ctx.request_repaint_after(Duration::from_millis(100))
    }
}