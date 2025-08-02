use eframe::{egui, Frame};
use egui::{Context, ColorImage, TextureHandle, Vec2};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::camera::list_devices::VideoDevice;
use crate::camera::camera::{FFmpegCamera, CameraFrame};

pub fn build_ui(devices: Vec<VideoDevice>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let first_device = devices.get(0).unwrap_or(&VideoDevice{ id: 0, name: "".to_owned() }).id;
    eframe::run_native(
        "Live Video Processor",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MyApp {
                available_devices: devices,
                delay_sec: 30,
                selected_device: first_device,
                current_frame: Arc::new(Mutex::new(None)),
                texture_handle: None,
                is_streaming: false,
                stream_handle: None,
                show_video: false,
            }))
        }),
    )
}

pub struct MyApp {
    available_devices: Vec<VideoDevice>,
    delay_sec: u32,
    selected_device: u32,
    current_frame: Arc<Mutex<Option<CameraFrame>>>,
    texture_handle: Option<TextureHandle>,
    is_streaming: bool,
    stream_handle: Option<thread::JoinHandle<()>>,
    show_video: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            available_devices: Vec::new(),
            delay_sec: 30,
            selected_device: 0,
            current_frame: Arc::new(Mutex::new(None)),
            texture_handle: None,
            is_streaming: false,
            stream_handle: None,
            show_video: false,
        }
    }
}

impl MyApp {
    fn start_streaming(&mut self, ctx: &Context) {
        if self.is_streaming {
            return;
        }

        println!("Starting video stream for device {}", self.selected_device);

        // First, let's try to capture a single frame to test camera access
        let test_camera = FFmpegCamera::new(self.selected_device, 640, 480, 30.00);
        match test_camera.capture_single_frame() {
            Ok(frame) => {
                println!("Successfully captured test frame: {}x{}, {} bytes",
                         frame.width, frame.height, frame.data.len());

                // Display the test frame immediately
                if let Ok(mut current_frame_guard) = self.current_frame.lock() {
                    *current_frame_guard = Some(frame);
                }
                ctx.request_repaint();
            }
            Err(e) => {
                eprintln!("Failed to capture test frame: {}", e);
                return;
            }
        }

        self.is_streaming = true;
        let frame_data = Arc::clone(&self.current_frame);
        let camera = FFmpegCamera::new(self.selected_device, 640, 480, 30.00);
        let ctx_clone = ctx.clone();
        let delay_sec = self.delay_sec;

        let handle = thread::spawn(move || {
            let mut frame_buffer = std::collections::VecDeque::new();
            let target_buffer_size = if delay_sec == 0 { 0 } else { (delay_sec as f64 * 30.0) as usize };

            println!("Starting camera capture with buffer size: {}", target_buffer_size);

            let result = camera.capture_continuous(move |frame| {
                println!("Received frame: {}x{}, {} bytes", frame.width, frame.height, frame.data.len());

                if delay_sec == 0 {
                    // No delay - display immediately
                    if let Ok(mut current_frame_guard) = frame_data.lock() {
                        *current_frame_guard = Some(frame);
                    }
                    ctx_clone.request_repaint();
                } else {
                    // Add frame to buffer
                    frame_buffer.push_back(frame);

                    // If we have enough frames for the delay, pop the oldest one to display
                    if frame_buffer.len() > target_buffer_size {
                        if let Some(delayed_frame) = frame_buffer.pop_front() {
                            if let Ok(mut current_frame_guard) = frame_data.lock() {
                                *current_frame_guard = Some(delayed_frame);
                            }
                            ctx_clone.request_repaint();
                        }
                    }
                }
            });

            if let Err(e) = result {
                eprintln!("Camera capture error: {}", e);
            }
            println!("Camera capture thread ended");
        });

        self.stream_handle = Some(handle);
    }

    fn stop_streaming(&mut self) {
        println!("Stopping video stream");
        self.is_streaming = false;
        // Note: In a production app, you'd want to implement proper thread cleanup
    }

    fn frame_to_color_image(frame: &CameraFrame) -> ColorImage {
        let mut pixels = Vec::with_capacity((frame.width * frame.height) as usize);

        // Convert RGB24 data to RGBA (egui expects RGBA)
        for chunk in frame.data.chunks(3) {
            if chunk.len() == 3 {
                pixels.push(egui::Color32::from_rgb(chunk[0], chunk[1], chunk[2]));
            }
        }

        ColorImage {
            size: [frame.width as usize, frame.height as usize],
            source_size: Default::default(),
            pixels,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.show_video {
                // Settings UI
                ui.heading("Live Video Processor");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Delay: ");
                    ui.add(egui::DragValue::new(&mut self.delay_sec).range(0..=120));
                    ui.label("seconds");
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Available Devices:");
                    egui::ComboBox::from_id_salt("Available Devices:")
                        .selected_text(&self.available_devices
                            .iter().find(|device| {device.id == self.selected_device})
                            .unwrap_or(&VideoDevice{ id: 0, name: "".to_owned() }).name)
                        .show_ui(ui, |ui| {
                            for device in &self.available_devices {
                                let id = device.id;
                                let name = &device.name;
                                ui.selectable_value(&mut self.selected_device, id, name);
                            }
                        });
                });

                ui.separator();

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.button("Start Video Display").clicked() {
                        self.show_video = true;
                        self.start_streaming(ctx);
                    }

                    ui.label(&format!("Selected Device: {}", self.selected_device));
                });

                // Show some helpful information
                ui.separator();
                ui.label("Instructions:");
                ui.label("1. Select your camera device from the dropdown");
                ui.label("2. Set the desired delay in seconds");
                ui.label("3. Click 'Start Video Display' to begin streaming");

                if self.available_devices.is_empty() {
                    ui.colored_label(egui::Color32::RED, "⚠ No camera devices found!");
                }
            } else {
                // Video display UI
                ui.horizontal(|ui| {
                    ui.heading(&format!("Video Display - Device {}", self.selected_device));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Back to Settings").clicked() {
                            self.show_video = false;
                            self.stop_streaming();
                        }
                    });
                });

                ui.label(&format!("Delay: {} seconds", self.delay_sec));
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Stop Stream").clicked() && self.is_streaming {
                        self.stop_streaming();
                    }

                    ui.label(if self.is_streaming {
                        "Status: Streaming"
                    } else {
                        "Status: Stopped"
                    });
                });

                ui.separator();

                // Display the current frame
                if let Ok(frame_guard) = self.current_frame.lock() {
                    if let Some(ref frame) = *frame_guard {
                        let color_image = Self::frame_to_color_image(frame);

                        // Create or update texture
                        let texture = self.texture_handle.get_or_insert_with(|| {
                            ctx.load_texture("video_frame", color_image.clone(), Default::default())
                        });

                        // Update texture with new frame data
                        texture.set(color_image, Default::default());

                        // Calculate display size while maintaining aspect ratio
                        let available_size = ui.available_size();
                        let image_aspect = frame.width as f32 / frame.height as f32;
                        let display_size = if available_size.x / available_size.y > image_aspect {
                            // Window is wider than image aspect ratio
                            Vec2::new(available_size.y * image_aspect, available_size.y * 0.8)
                        } else {
                            // Window is taller than image aspect ratio
                            Vec2::new(available_size.x * 0.8, available_size.x / image_aspect)
                        };

                        // Display the image
                        ui.add(egui::Image::from_texture(&*texture).fit_to_exact_size(display_size));

                        // Show frame info
                        ui.label(&format!(
                            "Frame: {}x{} | Time: {:?}",
                            frame.width,
                            frame.height,
                            frame.timestamp.elapsed().unwrap_or_default()
                        ));
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Waiting for video frames...");
                            if !self.is_streaming {
                                ui.label("Stream stopped.");
                            }
                        });
                    }
                }
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_streaming();
    }
}