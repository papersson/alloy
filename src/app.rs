use crate::{
    core::Timer,
    input::InputState,
    log,
    math::Vec3,
    renderer::SceneRenderer,
    scene::{Mesh, Node, Scene},
};
use std::cell::RefCell;
use std::rc::Rc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::HasWindowHandle,
    window::{CursorGrabMode, Window, WindowAttributes, WindowId},
};

pub struct App {
    window: Option<Window>,
    renderer: Option<SceneRenderer>,
    scene: Scene,
    timer: Timer,
    frame_count: u32,
    fps_update_timer: f32,
    current_fps: u32,
    input_state: InputState,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            scene: Self::create_scene(),
            timer: Timer::new(),
            frame_count: 0,
            fps_update_timer: 0.0,
            current_fps: 0,
            input_state: InputState::new(),
        }
    }

    fn create_scene() -> Scene {
        let mut scene = Scene::new();

        // Create ground plane
        let ground_node = Rc::new(RefCell::new(Node::with_mesh(
            "Ground".to_string(),
            Mesh::plane(10.0, 10.0),
        )));
        ground_node.borrow_mut().transform.position = Vec3::new(0.0, -1.0, 0.0);
        scene.add_node(ground_node);

        // Create multiple cubes
        let positions = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, -1.0),
        ];

        for (i, &position) in positions.iter().enumerate() {
            let cube_node = Rc::new(RefCell::new(Node::with_mesh(
                format!("Cube{i}"),
                Mesh::cube(),
            )));
            cube_node.borrow_mut().transform.position = position;
            scene.add_node(cube_node);
        }

        scene
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

                    // Capture cursor for mouse look
                    if let Err(e) = window.set_cursor_grab(CursorGrabMode::Locked) {
                        log!("Failed to lock cursor: {}, trying confined mode", e);
                        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                            log!("Failed to confine cursor: {}", e);
                        }
                    }
                    window.set_cursor_visible(false);

                    match window.window_handle() {
                        Ok(handle) => {
                            let size = window.inner_size();
                            match SceneRenderer::new(handle.as_raw(), size.width, size.height) {
                                Ok(renderer) => {
                                    self.renderer = Some(renderer);
                                    log!("Renderer initialized successfully");
                                    window.request_redraw();
                                }
                                Err(e) => {
                                    log!("Failed to create renderer: {}", e);
                                    event_loop.exit();
                                }
                            }
                        }
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
                if let Some(renderer) = &mut self.renderer {
                    if size.width > 0 && size.height > 0 {
                        renderer.update_drawable_size(size.width, size.height);
                        log!("Window resized to {}x{}", size.width, size.height);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => match physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    if state == ElementState::Pressed {
                        log!("Escape pressed, exiting");
                        event_loop.exit();
                    }
                }
                PhysicalKey::Code(KeyCode::Tab) => {
                    if state == ElementState::Pressed {
                        if let Some(window) = &self.window {
                            window.set_cursor_grab(CursorGrabMode::None).ok();
                            window.set_cursor_visible(true);
                        }
                    }
                }
                _ => match state {
                    ElementState::Pressed => self.input_state.key_pressed(physical_key),
                    ElementState::Released => self.input_state.key_released(physical_key),
                },
            },
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

                // Update camera based on input
                if let Some(renderer) = &mut self.renderer {
                    let camera = renderer.camera_mut();

                    // Handle movement
                    let movement_speed = self.input_state.movement_speed() * delta;

                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyW))
                    {
                        camera.move_forward(movement_speed);
                    }
                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyS))
                    {
                        camera.move_forward(-movement_speed);
                    }
                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyA))
                    {
                        camera.move_right(-movement_speed);
                    }
                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyD))
                    {
                        camera.move_right(movement_speed);
                    }

                    // Handle rotation
                    let (dx, dy) = self.input_state.mouse_delta();
                    if dx.abs() > 0.0 || dy.abs() > 0.0 {
                        let sensitivity = self.input_state.mouse_sensitivity();
                        camera.rotate(-dx * sensitivity, -dy * sensitivity);
                        self.input_state.reset_mouse_delta();
                    }

                    if let Err(e) = renderer.render(&self.scene) {
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

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.input_state.set_mouse_delta(dx as f32, dy as f32);
            }
            _ => {}
        }
    }
}
