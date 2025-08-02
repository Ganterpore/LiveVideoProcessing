use std::sync::{Arc, Mutex, atomic::{AtomicBool}};
use eframe::epaint::TextureHandle;
use std::thread;
use crate::camera::camera::CameraFrame;
use crate::camera::list_devices::VideoDevice;

pub struct AppState {
    pub available_devices: Vec<VideoDevice>,
    pub delay_sec: u32,
    pub selected_device: u32,
    pub current_frame: Arc<Mutex<Option<CameraFrame>>>,
    pub texture_handle: Option<TextureHandle>,
    pub is_streaming: Arc<AtomicBool>,
    pub stream_handle: Option<thread::JoinHandle<()>>,
    pub show_video: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            available_devices: Vec::new(),
            delay_sec: 30,
            selected_device: 0,
            current_frame: Arc::new(Mutex::new(None)),
            texture_handle: None,
            is_streaming: Arc::new(AtomicBool::new(false)),
            stream_handle: None,
            show_video: false,
        }
    }
}