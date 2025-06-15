use crate::core::Texture;
use crate::math::{Mat4, Vec3};
use crate::scene::{Camera, Mesh, Scene, Vertex};
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGSize;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBuffer, MTLClearColor, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue,
    MTLCompileOptions, MTLCreateSystemDefaultDevice, MTLDepthStencilDescriptor,
    MTLDepthStencilState, MTLDevice, MTLDrawable, MTLIndexType, MTLLibrary, MTLLoadAction,
    MTLPixelFormat, MTLPrimitiveType, MTLRenderCommandEncoder, MTLRenderPassDescriptor,
    MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLResourceOptions, MTLSamplerDescriptor,
    MTLSamplerMinMagFilter, MTLSamplerState, MTLStoreAction, MTLTexture, MTLTextureDescriptor,
    MTLTextureUsage, MTLVertexDescriptor,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use std::collections::HashMap;
use winit::raw_window_handle::RawWindowHandle;

#[repr(C)]
struct Uniforms {
    mvp_matrix: Mat4,
}

struct MeshBuffers {
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    uniform_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_count: usize,
}

pub struct SceneRenderer {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    layer: Retained<CAMetalLayer>,
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    depth_stencil_state: Retained<ProtocolObject<dyn MTLDepthStencilState>>,
    depth_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    default_texture: Texture,
    sampler_state: Retained<ProtocolObject<dyn MTLSamplerState>>,
    drawable_size: (u32, u32),
    camera: Camera,
    mesh_buffers: HashMap<*const Mesh, MeshBuffers>,
}

impl SceneRenderer {
    pub fn new(window_handle: RawWindowHandle, width: u32, height: u32) -> Result<Self, String> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| "Failed to get default Metal device".to_string())?;

        let command_queue = device
            .newCommandQueue()
            .ok_or_else(|| "Failed to create command queue".to_string())?;

        let layer = Self::create_metal_layer(&device, window_handle)?;

        let pipeline_state = Self::create_pipeline_state(&device)?;
        let depth_stencil_state = Self::create_depth_stencil_state(&device)?;
        let depth_texture = Self::create_depth_texture(&device, width, height)?;

        let default_texture = Self::create_checkerboard_texture(&device)?;
        let sampler_state = Self::create_sampler_state(&device)?;

        let aspect_ratio = width as f32 / height as f32;
        let camera = Camera::new(
            Vec3::new(3.0, 3.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            aspect_ratio,
        );

        Ok(Self {
            device,
            command_queue,
            layer,
            pipeline_state,
            depth_stencil_state,
            depth_texture: Some(depth_texture),
            default_texture,
            sampler_state,
            drawable_size: (width, height),
            camera,
            mesh_buffers: HashMap::new(),
        })
    }

    fn get_or_create_mesh_buffers(&mut self, mesh: &Mesh) -> Result<&MeshBuffers, String> {
        let mesh_ptr = mesh as *const Mesh;

        if !self.mesh_buffers.contains_key(&mesh_ptr) {
            let vertex_buffer = Self::create_vertex_buffer(&self.device, mesh)?;
            let index_buffer = Self::create_index_buffer(&self.device, mesh)?;
            let uniform_buffer = Self::create_uniform_buffer(&self.device)?;
            let buffers = MeshBuffers {
                vertex_buffer,
                index_buffer,
                uniform_buffer,
                index_count: mesh.indices.len(),
            };
            self.mesh_buffers.insert(mesh_ptr, buffers);
        }

        self.mesh_buffers
            .get(&mesh_ptr)
            .ok_or_else(|| "Failed to get mesh buffers".to_string())
    }

    fn create_metal_layer(
        device: &ProtocolObject<dyn MTLDevice>,
        window_handle: RawWindowHandle,
    ) -> Result<Retained<CAMetalLayer>, String> {
        let layer = unsafe { CAMetalLayer::new() };

        unsafe {
            layer.setDevice(Some(device));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setOpaque(true);
        }

        if let RawWindowHandle::AppKit(handle) = window_handle {
            let ns_view = handle.ns_view.as_ptr();
            let ns_view = ns_view.cast::<objc2::runtime::NSObject>();
            let _: () = unsafe { msg_send![ns_view, setWantsLayer: true] };
            let _: () = unsafe { msg_send![ns_view, setLayer: &*layer] };
        }

        Ok(layer)
    }

    fn create_vertex_buffer(
        device: &ProtocolObject<dyn MTLDevice>,
        mesh: &Mesh,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, String> {
        let vertex_data = mesh.vertices.as_slice();
        let buffer_size = std::mem::size_of_val(vertex_data);

        let buffer = unsafe {
            device.newBufferWithBytes_length_options(
                std::ptr::NonNull::new(vertex_data.as_ptr().cast::<std::ffi::c_void>().cast_mut())
                    .unwrap(),
                buffer_size,
                MTLResourceOptions::empty(),
            )
        }
        .ok_or_else(|| "Failed to create vertex buffer".to_string())?;

        Ok(buffer)
    }

    fn create_index_buffer(
        device: &ProtocolObject<dyn MTLDevice>,
        mesh: &Mesh,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, String> {
        let index_data = mesh.indices.as_slice();
        let buffer_size = std::mem::size_of_val(index_data);

        let buffer = unsafe {
            device.newBufferWithBytes_length_options(
                std::ptr::NonNull::new(index_data.as_ptr().cast::<std::ffi::c_void>().cast_mut())
                    .unwrap(),
                buffer_size,
                MTLResourceOptions::empty(),
            )
        }
        .ok_or_else(|| "Failed to create index buffer".to_string())?;

        Ok(buffer)
    }

    fn create_uniform_buffer(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, String> {
        let buffer_size = std::mem::size_of::<Uniforms>();

        let buffer = device
            .newBufferWithLength_options(buffer_size, MTLResourceOptions::empty())
            .ok_or_else(|| "Failed to create uniform buffer".to_string())?;

        Ok(buffer)
    }

    fn create_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/cube.metal");
        let shader_source = NSString::from_str(shader_source);

        let compile_options = MTLCompileOptions::new();
        let library = device
            .newLibraryWithSource_options_error(&shader_source, Some(&compile_options))
            .map_err(|e| format!("Failed to compile shaders: {:?}", e))?;

        let vertex_function = library
            .newFunctionWithName(&NSString::from_str("cube_vertex"))
            .ok_or_else(|| "Failed to find vertex shader".to_string())?;

        let fragment_function = library
            .newFunctionWithName(&NSString::from_str("cube_fragment"))
            .ok_or_else(|| "Failed to find fragment shader".to_string())?;

        let vertex_descriptor = unsafe { MTLVertexDescriptor::new() };

        unsafe {
            let position_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(0);
            position_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            position_attr.setOffset(0);
            position_attr.setBufferIndex(0);

            let tex_coord_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(1);
            tex_coord_attr.setFormat(objc2_metal::MTLVertexFormat::Float2);
            tex_coord_attr.setOffset(std::mem::offset_of!(Vertex, tex_coord));
            tex_coord_attr.setBufferIndex(0);

            let layout = vertex_descriptor.layouts().objectAtIndexedSubscript(0);
            layout.setStride(std::mem::size_of::<Vertex>());
        }

        let pipeline_descriptor = MTLRenderPipelineDescriptor::new();
        pipeline_descriptor.setVertexFunction(Some(&vertex_function));
        pipeline_descriptor.setFragmentFunction(Some(&fragment_function));
        pipeline_descriptor.setVertexDescriptor(Some(&vertex_descriptor));

        unsafe {
            let color_attachment = pipeline_descriptor
                .colorAttachments()
                .objectAtIndexedSubscript(0);
            color_attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        }

        pipeline_descriptor.setDepthAttachmentPixelFormat(MTLPixelFormat::Depth32Float);

        let pipeline_state = device
            .newRenderPipelineStateWithDescriptor_error(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create pipeline state: {:?}", e))?;

        Ok(pipeline_state)
    }

    fn create_depth_stencil_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLDepthStencilState>>, String> {
        let descriptor = unsafe { MTLDepthStencilDescriptor::new() };
        descriptor.setDepthCompareFunction(objc2_metal::MTLCompareFunction::Less);
        descriptor.setDepthWriteEnabled(true);

        let state = device
            .newDepthStencilStateWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create depth stencil state".to_string())?;

        Ok(state)
    }

    fn create_depth_texture(
        device: &ProtocolObject<dyn MTLDevice>,
        width: u32,
        height: u32,
    ) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, String> {
        let descriptor = unsafe { MTLTextureDescriptor::new() };
        descriptor.setTextureType(objc2_metal::MTLTextureType::Type2D);
        descriptor.setPixelFormat(MTLPixelFormat::Depth32Float);
        unsafe {
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
        }
        descriptor.setUsage(MTLTextureUsage::RenderTarget);

        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create depth texture".to_string())?;

        Ok(texture)
    }

    fn create_checkerboard_texture(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Texture, String> {
        const SIZE: u32 = 64;
        const CHECKER_SIZE: u32 = 8;
        let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];

        for y in 0..SIZE {
            for x in 0..SIZE {
                let is_white = ((x / CHECKER_SIZE) + (y / CHECKER_SIZE)) % 2 == 0;
                let color = if is_white { 255u8 } else { 0u8 };
                let idx = ((y * SIZE + x) * 4) as usize;
                data[idx] = color; // R
                data[idx + 1] = color; // G
                data[idx + 2] = color; // B
                data[idx + 3] = 255; // A
            }
        }

        Texture::create_from_data(device, &data, SIZE, SIZE, crate::core::TextureFormat::Rgba8)
    }

    fn create_sampler_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLSamplerState>>, String> {
        let descriptor = MTLSamplerDescriptor::new();
        descriptor.setMinFilter(MTLSamplerMinMagFilter::Nearest);
        descriptor.setMagFilter(MTLSamplerMinMagFilter::Nearest);

        let sampler = device
            .newSamplerStateWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create sampler state".to_string())?;

        Ok(sampler)
    }

    fn ensure_mesh_buffers(&mut self, scene: &Scene) -> Result<(), String> {
        // Pre-create buffers for all meshes in the scene
        scene.traverse(|node, _| {
            if let Some(mesh) = &node.mesh {
                let mesh_ptr = mesh as *const Mesh;
                if !self.mesh_buffers.contains_key(&mesh_ptr) {
                    if let Ok(vertex_buffer) = Self::create_vertex_buffer(&self.device, mesh) {
                        if let Ok(index_buffer) = Self::create_index_buffer(&self.device, mesh) {
                            if let Ok(uniform_buffer) = Self::create_uniform_buffer(&self.device) {
                                let buffers = MeshBuffers {
                                    vertex_buffer,
                                    index_buffer,
                                    uniform_buffer,
                                    index_count: mesh.indices.len(),
                                };
                                self.mesh_buffers.insert(mesh_ptr, buffers);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }

    pub fn render(&mut self, scene: &Scene) -> Result<(), String> {
        // Ensure all mesh buffers are created before rendering
        self.ensure_mesh_buffers(scene)?;
        let drawable = unsafe { self.layer.nextDrawable() }
            .ok_or_else(|| "Failed to get next drawable".to_string())?;

        let command_buffer = self
            .command_queue
            .commandBuffer()
            .ok_or_else(|| "Failed to create command buffer".to_string())?;

        let label = NSString::from_str("Scene Render Pass");
        command_buffer.setLabel(Some(&label));

        let render_pass_descriptor = unsafe { MTLRenderPassDescriptor::new() };
        let color_attachment = unsafe {
            render_pass_descriptor
                .colorAttachments()
                .objectAtIndexedSubscript(0)
        };

        unsafe {
            color_attachment.setTexture(Some(&drawable.texture()));
            color_attachment.setLoadAction(MTLLoadAction::Clear);
            color_attachment.setClearColor(MTLClearColor {
                red: 0.2,
                green: 0.3,
                blue: 0.4,
                alpha: 1.0,
            });
            color_attachment.setStoreAction(MTLStoreAction::Store);
        }

        if let Some(depth_texture) = &self.depth_texture {
            let depth_attachment = render_pass_descriptor.depthAttachment();
            depth_attachment.setTexture(Some(depth_texture));
            depth_attachment.setLoadAction(MTLLoadAction::Clear);
            depth_attachment.setClearDepth(1.0);
            depth_attachment.setStoreAction(MTLStoreAction::DontCare);
        }

        if let Some(render_encoder) =
            command_buffer.renderCommandEncoderWithDescriptor(&render_pass_descriptor)
        {
            let label = NSString::from_str("Scene Encoder");
            render_encoder.setLabel(Some(&label));

            render_encoder.setRenderPipelineState(&self.pipeline_state);
            render_encoder.setDepthStencilState(Some(&self.depth_stencil_state));

            // Render all nodes in the scene
            scene.traverse(|node, world_transform| {
                if let Some(mesh) = &node.mesh {
                    let mesh_ptr = mesh as *const Mesh;
                    if let Some(buffers) = self.mesh_buffers.get(&mesh_ptr) {
                        // Update uniforms with MVP matrix for this node
                        let mvp_matrix = self.camera.view_projection_matrix().multiply(world_transform);
                        let uniforms = Uniforms { mvp_matrix };
                        unsafe {
                            let contents = buffers.uniform_buffer.contents();
                            std::ptr::copy_nonoverlapping(
                                &raw const uniforms,
                                contents.as_ptr().cast::<Uniforms>(),
                                1,
                            );
                        }

                        unsafe {
                            render_encoder.setVertexBuffer_offset_atIndex(Some(&buffers.vertex_buffer), 0, 0);
                            render_encoder.setVertexBuffer_offset_atIndex(Some(&buffers.uniform_buffer), 0, 1);

                            render_encoder.setFragmentTexture_atIndex(Some(&self.default_texture.texture), 0);
                            render_encoder.setFragmentSamplerState_atIndex(Some(&self.sampler_state), 0);

                            render_encoder
                                .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset(
                                    MTLPrimitiveType::Triangle,
                                    buffers.index_count,
                                    MTLIndexType::UInt16,
                                    &buffers.index_buffer,
                                    0,
                                );
                        }
                    }
                }
            });

            render_encoder.endEncoding();
        }

        unsafe {
            let mtl_drawable = (&raw const *drawable).cast::<ProtocolObject<dyn MTLDrawable>>();
            command_buffer.presentDrawable(&*mtl_drawable);
        }

        command_buffer.commit();

        Ok(())
    }

    pub fn update_drawable_size(&mut self, width: u32, height: u32) {
        self.drawable_size = (width, height);

        let size = CGSize {
            width: f64::from(width),
            height: f64::from(height),
        };
        unsafe {
            self.layer.setDrawableSize(size);
        }

        // Update camera aspect ratio
        self.camera.set_aspect_ratio(width as f32 / height as f32);

        // Recreate depth texture with new size
        if let Ok(depth_texture) = Self::create_depth_texture(&self.device, width, height) {
            self.depth_texture = Some(depth_texture);
        }
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
