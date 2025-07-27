use crate::camera::CameraFrame;

pub fn save_frame_as_image(frame: &CameraFrame, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgb};

    let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        frame.width,
        frame.height,
        frame.data.clone(),
    ).ok_or("Failed to create image from raw data")?;

    img.save(filename)?;
    Ok(())
}