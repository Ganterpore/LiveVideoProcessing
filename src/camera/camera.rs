use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

pub struct StreamHandle {
    is_streaming: Arc<AtomicBool>,
}

impl StreamHandle {
    pub fn stop(&self) {
        self.is_streaming.store(false, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.is_streaming.load(Ordering::Relaxed)
    }
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

    pub fn capture_continuous<F>(&self, callback: F) -> Result<StreamHandle, Box<dyn std::error::Error>>
    where
        F: FnMut(CameraFrame) + Send + 'static,
    {
        let is_streaming = Arc::new(AtomicBool::new(true));
        let is_streaming_clone = Arc::clone(&is_streaming);

        let width = self.width;
        let height = self.height;
        let fps = self.fps;
        let device_id = self.device_id;

        // Spawn the capture thread
        std::thread::spawn(move || {
            if let Err(e) = Self::capture_loop(device_id, width, height, fps, callback, is_streaming_clone) {
                eprintln!("Capture loop error: {}", e);
            }
        });

        Ok(StreamHandle { is_streaming })
    }

    fn capture_loop<F>(
        device_id: u32,
        width: u32,
        height: u32,
        fps: f64,
        mut callback: F,
        is_streaming: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(CameraFrame),
    {
        println!("Starting FFmpeg with args: -f avfoundation -i {} -s {}x{} -r {} -pix_fmt rgb24 -f rawvideo -",
                 device_id, width, height, fps);

        let mut child = Command::new("ffmpeg")
            .args(&[
                "-f", "avfoundation", // TODO use different "f" for windows/linux
                "-r", &format!("{:.2}", fps),
                "-i", &device_id.to_string(),
                "-s", &format!("{}x{}", width, height),
                "-r", &format!("{:.1}", fps),
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

        let frame_size = (width * height * 3) as usize; // RGB24 = 3 bytes per pixel
        let mut buffer = vec![0u8; frame_size];
        let mut frame_count = 0;

        while is_streaming.load(Ordering::Relaxed) {
            // Read one frame worth of data
            match reader.read_exact(&mut buffer) {
                Ok(()) => {
                    frame_count += 1;
                    if frame_count <= 5 {
                        println!("Successfully read frame {}: {} bytes", frame_count, buffer.len());
                    }

                    let frame = CameraFrame {
                        width,
                        height,
                        data: buffer.clone(),
                        timestamp: std::time::SystemTime::now(),
                    };
                    callback(frame);
                }
                Err(e) => {
                    eprintln!("Error reading frame {}: {}", frame_count + 1, e);
                    break;
                }
            }
        }

        // Kill the FFmpeg process
        println!("Stopping stream, killing FFmpeg process");
        let _ = child.kill();
        let _ = child.wait();

        // Wait for stderr thread to finish
        let _ = stderr_handle.join();

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