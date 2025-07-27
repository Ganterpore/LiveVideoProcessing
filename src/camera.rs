use std::io::{BufReader, Read};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

pub struct FFmpegCamera {
    width: u32,
    height: u32,
    fps: f64,
    device_id: u32,
}

impl FFmpegCamera {
    pub fn new(device_id: u32, width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            device_id,
        }
    }

    pub fn list_devices() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new("ffmpeg")
            .args(&["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let devices: Vec<String> = stderr
            .lines()
            .filter(|line| line.contains("AVFoundation video devices:") || line.contains("["))
            .map(|line| line.to_string())
            .collect();

        Ok(devices)
    }

    pub fn capture_continuous<F>(&self, mut callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(CameraFrame) + Send + 'static,
    {
        let mut child = Command::new("ffmpeg")
            .args(&[
                "-f", "avfoundation",
                "-i", &self.device_id.to_string(),
                "-s", &format!("{}x{}", self.width, self.height),
                "-r", &format!("{:.1}", &self.fps),
                "-pix_fmt", "rgb24",
                "-f", "rawvideo",
                "-"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        let frame_size = (self.width * self.height * 3) as usize; // RGB24 = 3 bytes per pixel
        let mut buffer = vec![0u8; frame_size];

        loop {
            // Read one frame worth of data
            match reader.read_exact(&mut buffer) {
                Ok(()) => {
                    let frame = CameraFrame {
                        width: self.width,
                        height: self.height,
                        data: buffer.clone(),
                        timestamp: std::time::SystemTime::now(),
                    };
                    callback(frame);
                }
                Err(e) => {
                    eprintln!("Error reading frame: {}", e);
                    break;
                }
            }
        }

        let _ = child.wait();
        Ok(())
    }

    pub fn capture_single_frame(&self) -> Result<CameraFrame, Box<dyn std::error::Error>> {
        let output = Command::new("ffmpeg")
            .args(&[
                "-f", "avfoundation",
                "-r", &format!("{:.2}", &self.fps),
                "-i", &self.device_id.to_string(),
                "-s", &format!("{}x{}", self.width, self.height),
                "-frames:v", "1",
                "-pix_fmt", "rgb24",
                "-f", "rawvideo",
                "-"
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(CameraFrame {
            width: self.width,
            height: self.height,
            data: output.stdout,
            timestamp: std::time::SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = FFmpegCamera::new(0, 640, 480, 30.0);
        assert_eq!(camera.width, 640);
        assert_eq!(camera.height, 480);
        assert_eq!(camera.fps, 30.0);
        assert_eq!(format!("{:.1}", camera.fps), "30.0");
    }
}