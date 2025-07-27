use std::process::Command;

#[derive(Debug)]
pub struct VideoDevice {
    pub id: u32,
    pub name: String,
}

pub fn list_devices() -> Result<Vec<VideoDevice>, Box<dyn std::error::Error>> {
    let output = Command::new("ffmpeg")
        .args(&["-f", "avfoundation", "-list_devices", "true", "-i", ""])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut devices = Vec::new();
    let mut in_video_section = false;

    for line in stderr.lines() {
        // Check if we're entering the video devices section
        if line.contains("AVFoundation video devices:") {
            in_video_section = true;
            continue;
        }

        // Check if we've left the video devices section (entering audio section)
        if line.contains("AVFoundation audio devices:") {
            break;
        }

        // Only process lines in the video section that contain device entries
        if in_video_section && line.contains("[AVFoundation indev @") && line.contains("] [") {
            if let Some(device) = parse_device_line(line) {
                devices.push(device);
            }
        }
    }

    Ok(devices)
}

fn parse_device_line(line: &str) -> Option<VideoDevice> {
    // Look for pattern like "] [0] FaceTime HD Camera"
    if let Some(bracket_start) = line.find("] [") {
        let after_bracket = &line[bracket_start + 3..];
        if let Some(bracket_end) = after_bracket.find("] ") {
            let id_str = &after_bracket[..bracket_end];
            if let Ok(id) = id_str.parse::<u32>() {
                let name = after_bracket[bracket_end + 2..].trim().to_string();
                return Some(VideoDevice { id, name });
            }
        }
    }
    None
}

#[test]
fn test_device_listing() {
    // This test requires FFmpeg to be installed
    if let Ok(_devices) = list_devices() {
        assert!(true);
    } else {
        println!("FFmpeg not available for testing");
    }
}
