use crate::core::{Texture, TextureFormat};
use crate::math::{Mat4, Vec3};
use crate::scene::{Camera, Mesh, Vertex};
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
use winit::raw_window_handle::RawWindowHandle;

#[repr(C)]
struct Uniforms {
    mvp_matrix: Mat4,
}

pub struct CubeRenderer {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    layer: Retained<CAMetalLayer>,
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    depth_stencil_state: Retained<ProtocolObject<dyn MTLDepthStencilState>>,
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    uniform_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    depth_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    texture: Texture,
    sampler_state: Retained<ProtocolObject<dyn MTLSamplerState>>,
    drawable_size: (u32, u32),
    camera: Camera,
}

impl CubeRenderer {
    pub fn new(window_handle: RawWindowHandle, width: u32, height: u32) -> Result<Self, String> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| "Failed to get default Metal device".to_string())?;

        let command_queue = device
            .newCommandQueue()
            .ok_or_else(|| "Failed to create command queue".to_string())?;

        let layer = Self::create_metal_layer(&device, window_handle)?;

        let mesh = Mesh::cube();
        let vertex_buffer = Self::create_vertex_buffer(&device, &mesh)?;
        let index_buffer = Self::create_index_buffer(&device, &mesh)?;
        let uniform_buffer = Self::create_uniform_buffer(&device)?;

        let pipeline_state = Self::create_pipeline_state(&device)?;
        let depth_stencil_state = Self::create_depth_stencil_state(&device)?;
        let depth_texture = Self::create_depth_texture(&device, width, height)?;

        let texture = Self::create_checkerboard_texture(&device)?;
        let sampler_state = Self::create_sampler_state(&device)?;

        let aspect_ratio = width as f32 / height as f32;
        let camera = Camera::new(
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(0.0, 0.0, 0.0),
            aspect_ratio,
        );

        Ok(Self {
            device,
            command_queue,
            layer,
            pipeline_state,
            depth_stencil_state,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            depth_texture: Some(depth_texture),
            texture,
            sampler_state,
            drawable_size: (width, height),
            camera,
        })
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

        match window_handle {
            RawWindowHandle::AppKit(handle) => unsafe {
                use objc2::runtime::AnyObject;

                let view = handle.ns_view.as_ptr().cast::<AnyObject>();
                let _: () = msg_send![view, setWantsLayer: true];
                let _: () = msg_send![view, setLayer: &*layer];
            },
            _ => return Err("Unsupported window handle type".to_string()),
        }

        Ok(layer)
    }

    fn create_vertex_buffer(
        device: &ProtocolObject<dyn MTLDevice>,
        mesh: &Mesh,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, String> {
        let vertex_data = mesh.vertices.as_ptr().cast::<std::ffi::c_void>();
        let vertex_data_size = std::mem::size_of_val(&mesh.vertices[..]);

        let buffer = unsafe {
            device.newBufferWithBytes_length_options(
                std::ptr::NonNull::new(vertex_data.cast_mut()).unwrap(),
                vertex_data_size,
                MTLResourceOptions::CPUCacheModeDefaultCache,
            )
        }
        .ok_or_else(|| "Failed to create vertex buffer".to_string())?;

        Ok(buffer)
    }

    fn create_index_buffer(
        device: &ProtocolObject<dyn MTLDevice>,
        mesh: &Mesh,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, String> {
        let index_data = mesh.indices.as_ptr().cast::<std::ffi::c_void>();
        let index_data_size = std::mem::size_of_val(&mesh.indices[..]);

        let buffer = unsafe {
            device.newBufferWithBytes_length_options(
                std::ptr::NonNull::new(index_data.cast_mut()).unwrap(),
                index_data_size,
                MTLResourceOptions::CPUCacheModeDefaultCache,
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
            .newBufferWithLength_options(buffer_size, MTLResourceOptions::CPUCacheModeDefaultCache)
            .ok_or_else(|| "Failed to create uniform buffer".to_string())?;

        Ok(buffer)
    }

    fn create_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/cube.metal");
        let source_string = NSString::from_str(shader_source);
        let compile_options = MTLCompileOptions::new();

        let library = device
            .newLibraryWithSource_options_error(&source_string, Some(&compile_options))
            .map_err(|e| format!("Failed to compile shaders: {e:?}"))?;

        let vertex_fn_name = NSString::from_str("cube_vertex");
        let vertex_function = library
            .newFunctionWithName(&vertex_fn_name)
            .ok_or_else(|| "Failed to find vertex function".to_string())?;

        let fragment_fn_name = NSString::from_str("cube_fragment");
        let fragment_function = library
            .newFunctionWithName(&fragment_fn_name)
            .ok_or_else(|| "Failed to find fragment function".to_string())?;

        let vertex_descriptor = unsafe { MTLVertexDescriptor::new() };
        unsafe {
            // Position attribute
            let position_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(0);
            position_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            position_attr.setOffset(0);
            position_attr.setBufferIndex(0);

            // Texture coordinate attribute
            let tex_coord_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(1);
            tex_coord_attr.setFormat(objc2_metal::MTLVertexFormat::Float2);
            tex_coord_attr.setOffset(std::mem::offset_of!(Vertex, tex_coord));
            tex_coord_attr.setBufferIndex(0);

            // Layout
            let layout = vertex_descriptor.layouts().objectAtIndexedSubscript(0);
            layout.setStride(std::mem::size_of::<Vertex>());
        }

        let pipeline_descriptor = MTLRenderPipelineDescriptor::new();
        unsafe {
            pipeline_descriptor.setVertexFunction(Some(&vertex_function));
            pipeline_descriptor.setFragmentFunction(Some(&fragment_function));
            pipeline_descriptor.setVertexDescriptor(Some(&vertex_descriptor));
            pipeline_descriptor.setDepthAttachmentPixelFormat(MTLPixelFormat::Depth32Float);

            let color_attachment = pipeline_descriptor
                .colorAttachments()
                .objectAtIndexedSubscript(0);
            color_attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        }

        let pipeline_state = device
            .newRenderPipelineStateWithDescriptor_error(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create pipeline state: {e:?}"))?;

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
        unsafe {
            descriptor.setPixelFormat(MTLPixelFormat::Depth32Float);
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setUsage(MTLTextureUsage::RenderTarget);
        }

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

        Texture::create_from_data(device, &data, SIZE, SIZE, TextureFormat::Rgba8)
    }

    fn create_sampler_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLSamplerState>>, String> {
        let descriptor = MTLSamplerDescriptor::new();
        descriptor.setMinFilter(MTLSamplerMinMagFilter::Linear);
        descriptor.setMagFilter(MTLSamplerMinMagFilter::Linear);

        let sampler = device
            .newSamplerStateWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create sampler state".to_string())?;

        Ok(sampler)
    }

    pub fn render(&mut self) -> Result<(), String> {
        let drawable = unsafe { self.layer.nextDrawable() }
            .ok_or_else(|| "Failed to get next drawable".to_string())?;

        let command_buffer = self
            .command_queue
            .commandBuffer()
            .ok_or_else(|| "Failed to create command buffer".to_string())?;

        let label = NSString::from_str("Cube Render Pass");
        command_buffer.setLabel(Some(&label));

        // Update uniforms
        let mvp_matrix = self.camera.view_projection_matrix();
        let uniforms = Uniforms { mvp_matrix };
        unsafe {
            let contents = self.uniform_buffer.contents();
            std::ptr::copy_nonoverlapping(
                &raw const uniforms,
                contents.as_ptr().cast::<Uniforms>(),
                1,
            );
        }

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
            let label = NSString::from_str("Cube Encoder");
            render_encoder.setLabel(Some(&label));

            render_encoder.setRenderPipelineState(&self.pipeline_state);
            render_encoder.setDepthStencilState(Some(&self.depth_stencil_state));

            unsafe {
                render_encoder.setVertexBuffer_offset_atIndex(Some(&self.vertex_buffer), 0, 0);
                render_encoder.setVertexBuffer_offset_atIndex(Some(&self.uniform_buffer), 0, 1);

                render_encoder.setFragmentTexture_atIndex(Some(&self.texture.texture), 0);
                render_encoder.setFragmentSamplerState_atIndex(Some(&self.sampler_state), 0);

                render_encoder
                    .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset(
                        MTLPrimitiveType::Triangle,
                        36,
                        MTLIndexType::UInt16,
                        &self.index_buffer,
                        0,
                    );
            }

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
}
