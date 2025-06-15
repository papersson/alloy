mod font;

use crate::math::{Mat4, Vec2};
use font::BitmapFont;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLBuffer, MTLDevice, MTLRenderCommandEncoder, MTLTexture};

pub struct UIRenderer {
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    uniform_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    font: BitmapFont,
    vertices: Vec<UIVertex>,
    indices: Vec<u16>,
    vertex_count: usize,
    index_count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UIVertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UIUniforms {
    pub projection: Mat4,
}

impl UIRenderer {
    pub fn new(device: &ProtocolObject<dyn MTLDevice>) -> Self {
        let max_vertices = 4096;
        let max_indices = 6144;

        let vertex_buffer = device
            .newBufferWithLength_options(
                max_vertices * std::mem::size_of::<UIVertex>(),
                objc2_metal::MTLResourceOptions::CPUCacheModeWriteCombined,
            )
            .expect("Failed to create vertex buffer");

        let index_buffer = device
            .newBufferWithLength_options(
                max_indices * std::mem::size_of::<u16>(),
                objc2_metal::MTLResourceOptions::CPUCacheModeWriteCombined,
            )
            .expect("Failed to create index buffer");

        let uniform_buffer = device
            .newBufferWithLength_options(
                std::mem::size_of::<UIUniforms>(),
                objc2_metal::MTLResourceOptions::CPUCacheModeWriteCombined,
            )
            .expect("Failed to create uniform buffer");

        let font = BitmapFont::create_default(device);

        Self {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            font,
            vertices: Vec::with_capacity(max_vertices),
            indices: Vec::with_capacity(max_indices),
            vertex_count: 0,
            index_count: 0,
        }
    }

    pub fn update_projection(&self, width: f32, height: f32) {
        let projection = Mat4::orthographic(0.0, width, height, 0.0, -1.0, 1.0);

        unsafe {
            let uniforms = self.uniform_buffer.contents().as_ptr() as *mut UIUniforms;
            (*uniforms).projection = projection;
        }
    }

    pub fn begin_frame(&mut self) {
        // Reset vertex/index counts for new frame
        self.vertices.clear();
        self.indices.clear();
        self.vertex_count = 0;
        self.index_count = 0;
    }

    pub fn draw_text(&mut self, text: &str, position: Vec2, color: [f32; 4]) {
        let mut x = position.x;
        let y = position.y;
        let char_size = self.font.char_size();

        for ch in text.chars() {
            if let Some((uv0, uv1)) = self.font.get_char_uv(ch) {
                let base_idx = self.vertex_count as u16;

                // Add 4 vertices for the character quad
                self.vertices.push(UIVertex {
                    position: Vec2::new(x, y),
                    uv: uv0,
                    color,
                });
                self.vertices.push(UIVertex {
                    position: Vec2::new(x + char_size.x, y),
                    uv: Vec2::new(uv1.x, uv0.y),
                    color,
                });
                self.vertices.push(UIVertex {
                    position: Vec2::new(x + char_size.x, y + char_size.y),
                    uv: uv1,
                    color,
                });
                self.vertices.push(UIVertex {
                    position: Vec2::new(x, y + char_size.y),
                    uv: Vec2::new(uv0.x, uv1.y),
                    color,
                });

                // Add 6 indices for the two triangles
                self.indices.push(base_idx);
                self.indices.push(base_idx + 1);
                self.indices.push(base_idx + 2);
                self.indices.push(base_idx);
                self.indices.push(base_idx + 2);
                self.indices.push(base_idx + 3);

                self.vertex_count += 4;
                self.index_count += 6;
            }

            x += char_size.x + 1.0; // Add 1 pixel spacing between characters
        }
    }

    pub fn draw_rect(&mut self, position: Vec2, size: Vec2, color: [f32; 4]) {
        let base_idx = self.vertex_count as u16;

        // Add 4 vertices for the rectangle
        self.vertices.push(UIVertex {
            position,
            uv: Vec2::new(0.0, 0.0),
            color,
        });
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x + size.x, position.y),
            uv: Vec2::new(1.0, 0.0),
            color,
        });
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x + size.x, position.y + size.y),
            uv: Vec2::new(1.0, 1.0),
            color,
        });
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x, position.y + size.y),
            uv: Vec2::new(0.0, 1.0),
            color,
        });

        // Add 6 indices for the two triangles
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx + 3);

        self.vertex_count += 4;
        self.index_count += 6;
    }

    pub fn end_frame(&mut self) {
        // Upload vertex data to GPU
        if self.vertex_count > 0 {
            unsafe {
                let vertex_ptr = self.vertex_buffer.contents().as_ptr() as *mut UIVertex;
                let vertex_slice = std::slice::from_raw_parts_mut(vertex_ptr, self.vertex_count);
                vertex_slice.copy_from_slice(&self.vertices[..self.vertex_count]);

                let index_ptr = self.index_buffer.contents().as_ptr() as *mut u16;
                let index_slice = std::slice::from_raw_parts_mut(index_ptr, self.index_count);
                index_slice.copy_from_slice(&self.indices[..self.index_count]);
            }
        }
    }

    pub fn font_texture(&self) -> &ProtocolObject<dyn MTLTexture> {
        self.font.texture()
    }

    pub fn vertex_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.index_buffer
    }

    pub fn uniform_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.uniform_buffer
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }
}

pub struct FPSCounter {
    frame_times: Vec<f32>,
    last_update: std::time::Instant,
    current_fps: f32,
}

impl FPSCounter {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(60),
            last_update: std::time::Instant::now(),
            current_fps: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.frame_times.push(delta_time);

        let now = std::time::Instant::now();
        if now.duration_since(self.last_update).as_secs_f32() >= 1.0 {
            if !self.frame_times.is_empty() {
                let avg_frame_time =
                    self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
                self.current_fps = 1.0 / avg_frame_time;
                self.frame_times.clear();
            }
            self.last_update = now;
        }
    }

    pub fn fps(&self) -> f32 {
        self.current_fps
    }
}
