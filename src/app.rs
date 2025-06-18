use crate::{
    core::{GrassSystem, GravitySystem, RoadSystem, Skybox, SphericalWorld, Timer, TreeSystem},
    input::InputState,
    log,
    math::Vec3,
    renderer::SceneRenderer,
    scene::{Node, Scene},
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
    gravity_system: GravitySystem,
    planet_radius: f32,
    skybox: Skybox,
    grass_system: Option<GrassSystem>,
    road_system: Option<RoadSystem>,
    tree_system: Option<TreeSystem>,
}

impl App {
    pub fn new() -> Self {
        let planet_radius = 25.0; // Reduced from 50.0 for a smaller planet
        Self {
            window: None,
            renderer: None,
            ui_renderer: None,
            scene: Self::create_spherical_scene(planet_radius),
            timer: Timer::new(),
            frame_count: 0,
            fps_counter: FPSCounter::new(),
            input_state: InputState::new(),
            gravity_system: GravitySystem::new(Vec3::zero(), 9.8),
            planet_radius,
            skybox: Skybox::new(),
            grass_system: None,
            road_system: None,
            tree_system: None,
        }
    }

    fn create_spherical_scene(planet_radius: f32) -> Scene {
        let mut scene = Scene::new();

        // Create spherical world
        let world = SphericalWorld::new(planet_radius, 4); // 4 subdivisions for smooth sphere
        let sphere_node = Rc::new(RefCell::new(Node::with_mesh(
            "Planet".to_string(),
            world.generate_mesh(),
        )));
        scene.add_node(sphere_node);

        // Set light above the planet
        scene.light.position = Vec3::new(10.0, planet_radius + 20.0, 10.0);

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
                .with_title("Rust 3D Graphics Engine")
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

                                            // Set initial camera position
                                            if let Some(renderer) = &mut self.renderer {
                                                let initial_position =
                                                    Vec3::new(0.0, self.planet_radius + 1.0, 0.0);
                                                let camera = renderer.camera_mut();
                                                camera.set_position(initial_position);
                                                camera.set_up_vector(initial_position.normalize());
                                            }

                                            log!("Renderer initialized successfully");

                                            // Initialize skybox
                                            if let Some(renderer) = &mut self.renderer {
                                                if let Err(e) =
                                                    renderer.initialize_skybox(&self.skybox)
                                                {
                                                    log!("Failed to initialize skybox: {}", e);
                                                }
                                            }

                                            // Initialize grass system
                                            let grass_density = 1.0; // Increased density for smaller planet
                                            self.grass_system = Some(GrassSystem::new(
                                                self.planet_radius,
                                                grass_density,
                                            ));

                                            if let (Some(renderer), Some(grass_system)) =
                                                (&mut self.renderer, &self.grass_system)
                                            {
                                                if let Err(e) =
                                                    renderer.initialize_grass(grass_system)
                                                {
                                                    log!("Failed to initialize grass: {}", e);
                                                }
                                            }

                                            // Initialize road system
                                            // Create a road that follows the equator
                                            self.road_system = Some(RoadSystem::new(
                                                self.planet_radius,
                                                0.0,                        // start at 0 radians
                                                std::f32::consts::PI / 2.0, // end at 90 degrees
                                                3.0,                        // 3 meter wide road
                                            ));

                                            if let (Some(renderer), Some(road_system)) =
                                                (&mut self.renderer, &self.road_system)
                                            {
                                                if let Err(e) =
                                                    renderer.initialize_road(road_system)
                                                {
                                                    log!("Failed to initialize road: {}", e);
                                                }
                                            }

                                            // Initialize tree system
                                            self.tree_system = Some(TreeSystem::new(
                                                self.planet_radius,
                                                50,  // Increased from 20 to 50 trees
                                                0.0, // road start angle
                                                std::f32::consts::PI / 2.0, // road end angle
                                            ));

                                            if let (Some(renderer), Some(tree_system)) =
                                                (&mut self.renderer, &self.tree_system)
                                            {
                                                if let Err(e) =
                                                    renderer.initialize_tree(tree_system)
                                                {
                                                    log!("Failed to initialize tree: {}", e);
                                                }
                                            }
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
                    if let Some(renderer) = &mut self.renderer {
                        let camera = renderer.camera_mut();
                        camera.set_aspect_ratio(size.width as f32 / size.height as f32);
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

                // Update renderer time for skybox animation
                if let Some(renderer) = &mut self.renderer {
                    renderer.update_time(delta);
                }

                // Update camera with first-person movement
                if let Some(renderer) = &mut self.renderer {
                    let camera = renderer.camera_mut();

                    // Get current camera state
                    let current_position = camera.position();

                    // Update gravity system with current position
                    self.gravity_system.planet_center = Vec3::zero();

                    // Handle camera rotation with mouse
                    let (dx, dy) = self.input_state.mouse_delta();
                    if dx.abs() > 0.0 || dy.abs() > 0.0 {
                        let sensitivity = self.input_state.mouse_sensitivity();
                        camera.rotate(-dx * sensitivity, -dy * sensitivity);
                        self.input_state.reset_mouse_delta();
                    }

                    // Handle movement input
                    let mut movement = Vec3::zero();
                    let speed = if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::ShiftLeft))
                        || self
                            .input_state
                            .is_key_pressed(PhysicalKey::Code(KeyCode::ShiftRight))
                    {
                        10.0 // Running speed
                    } else {
                        5.0 // Walking speed
                    };

                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyW))
                    {
                        movement = movement.add(&camera.forward().scale(speed * delta));
                    }
                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyS))
                    {
                        movement = movement.sub(&camera.forward().scale(speed * delta));
                    }
                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyA))
                    {
                        movement = movement.sub(&camera.right().scale(speed * delta));
                    }
                    if self
                        .input_state
                        .is_key_pressed(PhysicalKey::Code(KeyCode::KeyD))
                    {
                        movement = movement.add(&camera.right().scale(speed * delta));
                    }

                    // Apply movement and constrain to sphere surface
                    if movement.length() > 0.0 {
                        let new_position = current_position.add(&movement);

                        // Constrain to sphere surface
                        let height_above_surface = 1.0; // Eye height
                        let constrained_position = new_position
                            .normalize()
                            .scale(self.planet_radius + height_above_surface);

                        camera.set_position(constrained_position);

                        // Update up vector to match new position on sphere
                        let new_up = constrained_position.normalize();
                        camera.set_up_vector(new_up);
                    }

                    camera.update(delta);

                    // Update grass LOD system with camera position
                    if let Some(grass_system) = &mut self.grass_system {
                        grass_system.update(camera.position());

                        // Update renderer grass buffers with new LOD data
                        if let Some(renderer) = &mut self.renderer {
                            if let Err(e) = renderer.update_grass(grass_system) {
                                log!("Failed to update grass buffers: {}", e);
                            }
                        }
                    }
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
                if let Some(renderer) = &mut self.renderer {
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
