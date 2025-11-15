mod ui;
mod camera;

fn main() -> Result<(), eframe::Error> {
    // Gets the list of available camera devices and builds the UI
    match camera::list_devices::list_devices() {
        Ok(devices) => {
            ui::root::build_ui(devices)
        }
        Err(e) => {
            println!("Error listing devices: {}", e);
            println!("Make sure FFmpeg is installed: brew install ffmpeg");
            return Ok(());
        }
    }
}