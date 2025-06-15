use crate::{
    core::Timer,
    input::InputState,
    log,
    math::Vec3,
    renderer::SceneRenderer,
    scene::{Mesh, Node, Scene},
    ui::{FPSCounter, UIRenderer},
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
    ui_renderer: Option<UIRenderer>,
    scene: Scene,
    timer: Timer,
    frame_count: u32,
    fps_counter: FPSCounter,
    input_state: InputState,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            ui_renderer: None,
            scene: Self::create_scene(),
            timer: Timer::new(),
            frame_count: 0,
            fps_counter: FPSCounter::new(),
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

    fn format_fps(&self) -> String {
        // Using String::with_capacity to avoid multiple allocations
        // This is still more efficient than format! which allocates multiple times
        let fps = self.fps_counter.fps() as u32;
        let mut result = String::with_capacity(16);
        result.push_str("FPS: ");
        result.push_str(&fps.to_string());
        result
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
                                    // Create UI renderer using the same device
                                    match UIRenderer::new(renderer.device()) {
                                        Ok(ui_renderer) => {
                                            ui_renderer.update_projection(
                                                size.width as f32,
                                                size.height as f32,
                                            );

                                            self.renderer = Some(renderer);
                                            self.ui_renderer = Some(ui_renderer);
                                            log!("Renderer initialized successfully");
                                        }
                                        Err(e) => {
                                            log!("Failed to initialize UI renderer: {}", e);
                                        }
                                    }
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
                if size.width > 0 && size.height > 0 {
                    if let Some(renderer) = &mut self.renderer {
                        renderer.update_drawable_size(size.width, size.height);
                    }
                    if let Some(ui_renderer) = &self.ui_renderer {
                        ui_renderer.update_projection(size.width as f32, size.height as f32);
                    }
                    log!("Window resized to {}x{}", size.width, size.height);
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

                // Update FPS counter
                self.fps_counter.update(delta);

                // Format FPS text before mutable borrows
                let fps_text = self.format_fps();

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

                    // Prepare UI rendering
                    if let Some(ui_renderer) = &mut self.ui_renderer {
                        ui_renderer.begin_frame();

                        // Draw FPS counter
                        ui_renderer.draw_text(
                            &fps_text,
                            crate::math::Vec2::new(10.0, 10.0),
                            [1.0, 1.0, 1.0, 1.0],
                        );

                        ui_renderer.end_frame();
                    }

                    // Render scene and UI together
                    if let Err(e) = renderer.render(&self.scene, self.ui_renderer.as_ref()) {
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
