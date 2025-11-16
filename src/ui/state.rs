use std::sync::{Arc, Mutex};
use eframe::epaint::TextureHandle;
use crate::camera::camera::{CameraFrame, StreamHandle};
use crate::camera::list_devices::VideoDevice;

pub struct AppState {
    pub available_devices: Vec<VideoDevice>,
    pub delay_sec: u32,
    pub selected_device: u32,
    pub width: u32,
    pub height: u32,
    pub current_frame: Arc<Mutex<Option<CameraFrame>>>,
    pub texture_handle: Option<TextureHandle>,
    pub stream_handle: Option<StreamHandle>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            available_devices: Vec::new(),
            delay_sec: 30,
            selected_device: 0,
            width: 640,
            height: 480,
            current_frame: Arc::new(Mutex::new(None)),
            texture_handle: None,
            stream_handle: None
        }
    }
}