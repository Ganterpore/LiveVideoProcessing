mod ui;
mod camera;

fn main() -> Result<(), eframe::Error> {
    println!("FFmpeg Camera Demo");
    
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


    // Create camera instance
    // TODO id, width, height, fps??
    // let camera = FFmpegCamera::new(0, 640, 480, 30.00);
    // 
    // // Test single frame capture
    // println!("\nCapturing single fram...");
    // match camera.capture_single_frame() {
    //     Ok(frame) => {
    //         println!("Captured frame: {}x{}, {} bytes",
    //                  frame.width, frame.height, frame.data.len());
    // 
    //         // Save frame as image file
    //         if let Err(e) = save_frame_as_image(&frame, "test_frame.png") {
    //             println!("Error saving frame: {}", e);
    //         } else {
    //             println!("Frame saved as test_frame.png");
    //         }
    //     }
    //     Err(e) => {
    //         println!("Error capturing frame: {}", e);
    //     }
    // }

    // Test continuous capture
    // println!("\nStarting continuous capture... Press Ctrl+C to stop");
    // let mut frame_count = 0;
    // 
    // camera.capture_continuous(move |frame| {
    //     frame_count += 1;
    //     if frame_count % 30 == 0 {  // Print every 30 frames (~1 second at 30fps)
    //         println!("Frame {}: {}x{} {} bytes",
    //                  frame_count, frame.width, frame.height, frame.data.len());
    //     }
    // })?;

    // Ok(())
}