use eframe::{egui, Frame};
use egui::Context;

pub fn build_ui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
    ..Default::default()
    };

    eframe::run_native(
        "Live Video Processor",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}
pub struct MyApp {
    delay_sec: u32
}
impl Default for MyApp {
    fn default() -> Self {
        Self {
            delay_sec: 30
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
        });
    }
}
