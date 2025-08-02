use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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

    pub fn capture_continuous<F>(&self, mut callback: F, is_streaming: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(CameraFrame) + Send + 'static,
    {
        println!("Starting FFmpeg with args: -f avfoundation -i {} -s {}x{} -r {} -pix_fmt rgb24 -f rawvideo -",
                 self.device_id, self.width, self.height, self.fps);

        let mut child = Command::new("ffmpeg")
            .args(&[
                "-f", "avfoundation", // TODO use different "f" for windows/linux
                "-r", &format!("{:.2}", &self.fps),
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
        let stderr = child.stderr.take().unwrap();

        // Spawn a thread to read and print stderr
        let stderr_handle = std::thread::spawn(move || {
            let mut stderr_reader = BufReader::new(stderr);
            let mut line = String::new();
            use std::io::BufRead;
            while let Ok(bytes_read) = stderr_reader.read_line(&mut line) {
                if bytes_read == 0 {
                    break;
                }
                print!("FFmpeg stderr: {}", line);
                line.clear();
            }
        });

        let mut reader = BufReader::new(stdout);

        let frame_size = (self.width * self.height * 3) as usize; // RGB24 = 3 bytes per pixel
        let mut buffer = vec![0u8; frame_size];
        let mut frame_count = 0;

        while is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
            // Read one frame worth of data
            match reader.read_exact(&mut buffer) {
                Ok(()) => {
                    frame_count += 1;
                    if frame_count <= 5 {
                        println!("Successfully read frame {}: {} bytes", frame_count, buffer.len());
                    }

                    let frame = CameraFrame {
                        width: self.width,
                        height: self.height,
                        data: buffer.clone(),
                        timestamp: std::time::SystemTime::now(),
                    };
                    callback(frame);
                }
                Err(e) => {
                    eprintln!("Error reading frame {}: {}", frame_count + 1, e);

                    // Try to read whatever data is available
                    let mut partial_buffer = Vec::new();
                    match reader.read_to_end(&mut partial_buffer) {
                        Ok(bytes_read) => {
                            println!("Read {} remaining bytes before error", bytes_read);
                        }
                        Err(read_err) => {
                            println!("Could not read remaining data: {}", read_err);
                        }
                    }
                    break;
                }
            }
        }

        // Wait for stderr thread to finish
        let _ = stderr_handle.join();

        // Check if the process is still running and kill it if necessary
        match child.try_wait() {
            Ok(Some(status)) => {
                println!("FFmpeg process exited with status: {}", status);
            }
            Ok(None) => {
                println!("FFmpeg process still running, killing it");
                let _ = child.kill();
                let _ = child.wait();
            }
            Err(e) => {
                println!("Error checking FFmpeg process status: {}", e);
            }
        }

        Ok(())
    }

    pub fn capture_single_frame(&self) -> Result<CameraFrame, Box<dyn std::error::Error>> {
        println!("Capturing single frame from device {}", self.device_id);

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
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            eprintln!("FFmpeg stderr: {}", stderr_str);
            return Err(format!("FFmpeg failed with status {}: {}", output.status, stderr_str).into());
        }

        let expected_size = (self.width * self.height * 3) as usize;
        println!("Single frame capture: expected {} bytes, got {} bytes",
                 expected_size, output.stdout.len());

        if output.stdout.len() != expected_size {
            println!("Warning: Frame size mismatch. Expected {}, got {}",
                     expected_size, output.stdout.len());
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