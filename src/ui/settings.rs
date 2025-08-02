use egui::{Context, Ui};
use crate::camera::list_devices::VideoDevice;
use crate::ui::state::AppState;

impl AppState {
    pub fn show_settings(&mut self, ctx: &Context, ui: &mut Ui) {
        // Settings UI
        ui.heading("Live Video Processor");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Delay: ");
            ui.add(egui::DragValue::new(&mut self.delay_sec).range(0..=120));
            ui.label("seconds");
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Available Devices:");
            egui::ComboBox::from_id_salt("Available Devices:")
                .selected_text(&self.available_devices
                    .iter().find(|device| { device.id == self.selected_device })
                    .unwrap_or(&VideoDevice { id: 0, name: "".to_owned() }).name)
                .show_ui(ui, |ui| {
                    for device in &self.available_devices {
                        let id = device.id;
                        let name = &device.name;
                        ui.selectable_value(&mut self.selected_device, id, name);
                    }
                });
        });

        ui.separator();

        // Control buttons
        ui.horizontal(|ui| {
            if ui.button("Start Video Display").clicked() {
                self.show_video = true;
                self.start_streaming(ctx);
            }

            ui.label(&format!("Selected Device: {}", self.selected_device));
        });

        // Show some helpful information
        ui.separator();
        ui.label("Instructions:");
        ui.label("1. Select your camera device from the dropdown");
        ui.label("2. Set the desired delay in seconds");
        ui.label("3. Click 'Start Video Display' to begin streaming");

        if self.available_devices.is_empty() {
            ui.colored_label(egui::Color32::RED, "⚠ No camera devices found!");
        }
    }
}