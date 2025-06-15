use crate::{core::Timer, log, renderer::Renderer};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
    raw_window_handle::HasWindowHandle,
};

pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    timer: Timer,
    frame_count: u32,
    fps_update_timer: f32,
    current_fps: u32,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            timer: Timer::new(),
            frame_count: 0,
            fps_update_timer: 0.0,
            current_fps: 0,
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App::new();
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Rust 3D Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    log!("Window created successfully");

                    match window.window_handle() {
                        Ok(handle) => match Renderer::new(handle.as_raw()) {
                            Ok(renderer) => {
                                let size = window.inner_size();
                                if size.width > 0 && size.height > 0 {
                                    renderer.update_drawable_size(size.width, size.height);
                                }
                                self.renderer = Some(renderer);
                                log!("Renderer initialized successfully");
                            }
                            Err(e) => {
                                log!("Failed to create renderer: {}", e);
                                event_loop.exit();
                            }
                        },
                        Err(e) => {
                            log!("Failed to get window handle: {}", e);
                            event_loop.exit();
                        }
                    }

                    self.window = Some(window);
                }
                Err(e) => {
                    log!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &self.renderer {
                    if size.width > 0 && size.height > 0 {
                        renderer.update_drawable_size(size.width, size.height);
                        log!("Window resized to {}x{}", size.width, size.height);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                log!("Escape pressed, exiting");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let delta = self.timer.delta();
                self.frame_count += 1;
                self.fps_update_timer += delta;

                if self.fps_update_timer >= 1.0 {
                    self.current_fps = self.frame_count;
                    self.frame_count = 0;
                    self.fps_update_timer = 0.0;
                    log!("FPS: {}", self.current_fps);
                }

                if let Some(renderer) = &self.renderer {
                    if let Err(e) = renderer.render() {
                        log!("Render error: {}", e);
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
