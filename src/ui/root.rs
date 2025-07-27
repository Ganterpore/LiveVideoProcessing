use eframe::{egui, Frame};
use egui::Context;
use crate::camera::list_devices::VideoDevice;

pub fn build_ui(devices: Vec<VideoDevice>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    let first_device = devices.get(0).unwrap_or(&VideoDevice{ id: 0, name: "".to_owned() }).id;
    eframe::run_native(
        "Live Video Processor",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MyApp {
                available_devices: devices,
                delay_sec: 30,
                selected_device: first_device
            }))
        }),
    )
}

pub struct MyApp {
    available_devices: Vec<VideoDevice>,
    delay_sec: u32,
    selected_device: u32
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            available_devices: Vec::new(),
            delay_sec: 30,
            selected_device: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Video Processor");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Delay: ");
                ui.add(egui::DragValue::new(&mut self.delay_sec).range(0..=120));
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Available Devices:");
                egui::ComboBox::from_id_salt("Available Devices:")
                    .selected_text(&self.available_devices
                        .iter().find(|device| {device.id == self.selected_device})
                        .unwrap_or(&VideoDevice{ id: 0, name: "".to_owned() }).name)
                    .show_ui(ui, |ui| {
                        for device in &self.available_devices {
                            let id = device.id;
                            let name = &device.name;
                            ui.selectable_value(&mut self.selected_device, id, name);
                        }
                    });
            })
        });
    }
}