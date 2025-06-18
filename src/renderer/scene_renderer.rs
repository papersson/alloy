use crate::core::{GrassSystem, Texture};
use crate::math::{Mat4, Vec3, Vec4};
use crate::scene::{Camera, Mesh, Scene, Vertex};
use crate::ui::{UIRenderer, UIVertex};
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGSize;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBlendFactor, MTLBlendOperation, MTLBuffer, MTLClearColor, MTLCommandBuffer,
    MTLCommandEncoder, MTLCommandQueue, MTLCompileOptions, MTLCreateSystemDefaultDevice,
    MTLDepthStencilDescriptor, MTLDepthStencilState, MTLDevice, MTLDrawable, MTLIndexType,
    MTLLibrary, MTLLoadAction, MTLPixelFormat, MTLPrimitiveType, MTLRenderCommandEncoder,
    MTLRenderPassDescriptor, MTLRenderPipelineDescriptor, MTLRenderPipelineState,
    MTLResourceOptions, MTLSamplerDescriptor, MTLSamplerMinMagFilter, MTLSamplerState,
    MTLStoreAction, MTLTexture, MTLTextureDescriptor, MTLTextureUsage, MTLVertexDescriptor,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use std::collections::HashMap;
use winit::raw_window_handle::RawWindowHandle;

#[repr(C)]
struct Uniforms {
    mvp_matrix: Mat4,
    model_matrix: Mat4,
    normal_matrix: Mat4,
    view_pos: Vec3,
    time: f32,
    light_pos: Vec3,
    _padding1: f32,
    light_color: Vec3,
    ambient_strength: f32,
    diffuse_strength: f32,
    specular_strength: f32,
    fog_density: f32,
    fog_color: Vec3,
    fog_start: f32,
    horizon_color: Vec3,
    _padding2: f32,
    zenith_color: Vec3,
    _padding3: f32,
}

#[repr(C)]
struct SkyboxUniforms {
    view_projection_matrix: Mat4,
    camera_pos: Vec3,
    time: f32,
    sun_direction: Vec3,
    _padding: f32,
}

struct MeshBuffers {
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    uniform_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_count: usize,
}

struct GrassBuffers {
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    instance_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    uniform_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    index_count: usize,
    instance_count: usize,
}

pub struct SceneRenderer {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    layer: Retained<CAMetalLayer>,
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    ui_pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    skybox_pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    grass_pipeline_state: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    road_pipeline_state: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    tree_pipeline_state: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    depth_stencil_state: Retained<ProtocolObject<dyn MTLDepthStencilState>>,
    ui_depth_stencil_state: Retained<ProtocolObject<dyn MTLDepthStencilState>>,
    skybox_depth_stencil_state: Retained<ProtocolObject<dyn MTLDepthStencilState>>,
    depth_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    default_texture: Texture,
    sampler_state: Retained<ProtocolObject<dyn MTLSamplerState>>,
    drawable_size: (u32, u32),
    camera: Camera,
    mesh_buffers: HashMap<*const Mesh, MeshBuffers>,
    skybox_buffers: Option<MeshBuffers>,
    grass_buffers: Option<GrassBuffers>,
    road_buffers: Option<MeshBuffers>,
    tree_buffers: Option<GrassBuffers>,
    time: f32,
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
        let ui_pipeline_state = Self::create_ui_pipeline_state(&device)?;
        let skybox_pipeline_state = Self::create_skybox_pipeline_state(&device)?;
        let depth_stencil_state = Self::create_depth_stencil_state(&device)?;
        let ui_depth_stencil_state = Self::create_ui_depth_stencil_state(&device)?;
        let skybox_depth_stencil_state = Self::create_skybox_depth_stencil_state(&device)?;
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
            ui_pipeline_state,
            skybox_pipeline_state,
            grass_pipeline_state: None,
            road_pipeline_state: None,
            tree_pipeline_state: None,
            depth_stencil_state,
            ui_depth_stencil_state,
            skybox_depth_stencil_state,
            depth_texture: Some(depth_texture),
            default_texture,
            sampler_state,
            drawable_size: (width, height),
            camera,
            mesh_buffers: HashMap::new(),
            skybox_buffers: None,
            grass_buffers: None,
            road_buffers: None,
            tree_buffers: None,
            time: 0.0,
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
        // Safety: CAMetalLayer::new() is a valid constructor
        let layer = unsafe { CAMetalLayer::new() };

        // Safety: Setting layer properties with valid values
        unsafe {
            layer.setDevice(Some(device));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setOpaque(true);
        }

        if let RawWindowHandle::AppKit(handle) = window_handle {
            let ns_view = handle.ns_view.as_ptr();
            let ns_view = ns_view.cast::<objc2::runtime::NSObject>();
            // Safety: ns_view is a valid NSView pointer from the window handle,
            // and we're setting standard layer properties
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

        let data_ptr =
            std::ptr::NonNull::new(vertex_data.as_ptr().cast::<std::ffi::c_void>().cast_mut())
                .ok_or_else(|| "Failed to create NonNull pointer for vertex data".to_string())?;

        // Safety: data_ptr points to valid vertex data that lives at least as long as this function call.
        // The Metal API will copy the data during buffer creation.
        let buffer = unsafe {
            device.newBufferWithBytes_length_options(
                data_ptr,
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

        let data_ptr =
            std::ptr::NonNull::new(index_data.as_ptr().cast::<std::ffi::c_void>().cast_mut())
                .ok_or_else(|| "Failed to create NonNull pointer for index data".to_string())?;

        // Safety: data_ptr points to valid index data that lives at least as long as this function call.
        // The Metal API will copy the data during buffer creation.
        let buffer = unsafe {
            device.newBufferWithBytes_length_options(
                data_ptr,
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

            let normal_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(2);
            normal_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            normal_attr.setOffset(std::mem::offset_of!(Vertex, normal));
            normal_attr.setBufferIndex(0);

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

    fn create_ui_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/ui.metal");
        let shader_source = NSString::from_str(shader_source);

        let compile_options = MTLCompileOptions::new();
        let library = device
            .newLibraryWithSource_options_error(&shader_source, Some(&compile_options))
            .map_err(|e| format!("Failed to compile UI shaders: {:?}", e))?;

        let vertex_function = library
            .newFunctionWithName(&NSString::from_str("ui_vertex"))
            .ok_or_else(|| "Failed to find UI vertex shader".to_string())?;

        let fragment_function = library
            .newFunctionWithName(&NSString::from_str("ui_fragment"))
            .ok_or_else(|| "Failed to find UI fragment shader".to_string())?;

        let vertex_descriptor = unsafe { MTLVertexDescriptor::new() };

        unsafe {
            // Position attribute
            let position_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(0);
            position_attr.setFormat(objc2_metal::MTLVertexFormat::Float2);
            position_attr.setOffset(0);
            position_attr.setBufferIndex(0);

            // TexCoord attribute
            let tex_coord_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(1);
            tex_coord_attr.setFormat(objc2_metal::MTLVertexFormat::Float2);
            tex_coord_attr.setOffset(std::mem::offset_of!(UIVertex, uv));
            tex_coord_attr.setBufferIndex(0);

            // Color attribute
            let color_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(2);
            color_attr.setFormat(objc2_metal::MTLVertexFormat::Float4);
            color_attr.setOffset(std::mem::offset_of!(UIVertex, color));
            color_attr.setBufferIndex(0);

            let layout = vertex_descriptor.layouts().objectAtIndexedSubscript(0);
            layout.setStride(std::mem::size_of::<UIVertex>());
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

            // Enable alpha blending for UI overlay
            color_attachment.setBlendingEnabled(true);
            color_attachment.setSourceRGBBlendFactor(MTLBlendFactor::SourceAlpha);
            color_attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
            color_attachment.setRgbBlendOperation(MTLBlendOperation::Add);
            color_attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
            color_attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
            color_attachment.setAlphaBlendOperation(MTLBlendOperation::Add);
        }

        pipeline_descriptor.setDepthAttachmentPixelFormat(MTLPixelFormat::Depth32Float);

        let pipeline_state = device
            .newRenderPipelineStateWithDescriptor_error(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create UI pipeline state: {:?}", e))?;

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

    fn create_skybox_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/skybox.metal");
        let shader_source = NSString::from_str(shader_source);

        let compile_options = MTLCompileOptions::new();
        let library = device
            .newLibraryWithSource_options_error(&shader_source, Some(&compile_options))
            .map_err(|e| format!("Failed to compile skybox shaders: {:?}", e))?;

        let vertex_function = library
            .newFunctionWithName(&NSString::from_str("skybox_vertex"))
            .ok_or_else(|| "Failed to find skybox vertex shader".to_string())?;

        let fragment_function = library
            .newFunctionWithName(&NSString::from_str("skybox_fragment"))
            .ok_or_else(|| "Failed to find skybox fragment shader".to_string())?;

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

            let normal_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(2);
            normal_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            normal_attr.setOffset(std::mem::offset_of!(Vertex, normal));
            normal_attr.setBufferIndex(0);

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
            .map_err(|e| format!("Failed to create skybox pipeline state: {:?}", e))?;

        Ok(pipeline_state)
    }

    fn create_skybox_depth_stencil_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLDepthStencilState>>, String> {
        let descriptor = unsafe { MTLDepthStencilDescriptor::new() };
        // Skybox should render behind everything but still depth test
        descriptor.setDepthCompareFunction(objc2_metal::MTLCompareFunction::LessEqual);
        descriptor.setDepthWriteEnabled(false); // Don't write to depth buffer

        let state = device
            .newDepthStencilStateWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create skybox depth stencil state".to_string())?;

        Ok(state)
    }

    fn create_grass_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/grass.metal");
        let shader_source = NSString::from_str(shader_source);

        let compile_options = MTLCompileOptions::new();
        let library = device
            .newLibraryWithSource_options_error(&shader_source, Some(&compile_options))
            .map_err(|e| format!("Failed to compile grass shaders: {:?}", e))?;

        let vertex_function = library
            .newFunctionWithName(&NSString::from_str("grass_vertex"))
            .ok_or_else(|| "Failed to find grass vertex shader".to_string())?;

        let fragment_function = library
            .newFunctionWithName(&NSString::from_str("grass_fragment"))
            .ok_or_else(|| "Failed to find grass fragment shader".to_string())?;

        let vertex_descriptor = unsafe { MTLVertexDescriptor::new() };

        unsafe {
            // Per-vertex attributes
            let position_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(0);
            position_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            position_attr.setOffset(0);
            position_attr.setBufferIndex(0);

            let tex_coord_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(1);
            tex_coord_attr.setFormat(objc2_metal::MTLVertexFormat::Float2);
            tex_coord_attr.setOffset(std::mem::offset_of!(Vertex, tex_coord));
            tex_coord_attr.setBufferIndex(0);

            let normal_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(2);
            normal_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            normal_attr.setOffset(std::mem::offset_of!(Vertex, normal));
            normal_attr.setBufferIndex(0);

            let layout = vertex_descriptor.layouts().objectAtIndexedSubscript(0);
            layout.setStride(std::mem::size_of::<Vertex>());
            layout.setStepFunction(objc2_metal::MTLVertexStepFunction::PerVertex);
            layout.setStepRate(1);
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
            .map_err(|e| format!("Failed to create grass pipeline state: {:?}", e))?;

        Ok(pipeline_state)
    }

    fn create_road_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/road.metal");
        let shader_source = NSString::from_str(shader_source);

        let compile_options = MTLCompileOptions::new();
        let library = device
            .newLibraryWithSource_options_error(&shader_source, Some(&compile_options))
            .map_err(|e| format!("Failed to compile road shaders: {:?}", e))?;

        let vertex_function = library
            .newFunctionWithName(&NSString::from_str("road_vertex"))
            .ok_or_else(|| "Failed to find road vertex shader".to_string())?;

        let fragment_function = library
            .newFunctionWithName(&NSString::from_str("road_fragment"))
            .ok_or_else(|| "Failed to find road fragment shader".to_string())?;

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

            let normal_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(2);
            normal_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            normal_attr.setOffset(std::mem::offset_of!(Vertex, normal));
            normal_attr.setBufferIndex(0);

            let layout = vertex_descriptor.layouts().objectAtIndexedSubscript(0);
            layout.setStride(std::mem::size_of::<Vertex>());
            layout.setStepFunction(objc2_metal::MTLVertexStepFunction::PerVertex);
            layout.setStepRate(1);
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
            .map_err(|e| format!("Failed to create road pipeline state: {:?}", e))?;

        Ok(pipeline_state)
    }

    fn create_tree_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/tree.metal");
        let shader_source = NSString::from_str(shader_source);

        let compile_options = MTLCompileOptions::new();
        let library = device
            .newLibraryWithSource_options_error(&shader_source, Some(&compile_options))
            .map_err(|e| format!("Failed to compile tree shaders: {:?}", e))?;

        let vertex_function = library
            .newFunctionWithName(&NSString::from_str("tree_vertex"))
            .ok_or_else(|| "Failed to find tree vertex shader".to_string())?;

        let fragment_function = library
            .newFunctionWithName(&NSString::from_str("tree_fragment"))
            .ok_or_else(|| "Failed to find tree fragment shader".to_string())?;

        let vertex_descriptor = unsafe { MTLVertexDescriptor::new() };

        unsafe {
            // Per-vertex attributes
            let position_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(0);
            position_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            position_attr.setOffset(0);
            position_attr.setBufferIndex(0);

            let tex_coord_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(1);
            tex_coord_attr.setFormat(objc2_metal::MTLVertexFormat::Float2);
            tex_coord_attr.setOffset(std::mem::offset_of!(Vertex, tex_coord));
            tex_coord_attr.setBufferIndex(0);

            let normal_attr = vertex_descriptor.attributes().objectAtIndexedSubscript(2);
            normal_attr.setFormat(objc2_metal::MTLVertexFormat::Float3);
            normal_attr.setOffset(std::mem::offset_of!(Vertex, normal));
            normal_attr.setBufferIndex(0);

            let layout = vertex_descriptor.layouts().objectAtIndexedSubscript(0);
            layout.setStride(std::mem::size_of::<Vertex>());
            layout.setStepFunction(objc2_metal::MTLVertexStepFunction::PerVertex);
            layout.setStepRate(1);
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
            .map_err(|e| format!("Failed to create tree pipeline state: {:?}", e))?;

        Ok(pipeline_state)
    }

    fn create_ui_depth_stencil_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLDepthStencilState>>, String> {
        let descriptor = unsafe { MTLDepthStencilDescriptor::new() };
        // UI should always render on top, so we disable depth testing and writing
        descriptor.setDepthCompareFunction(objc2_metal::MTLCompareFunction::Always);
        descriptor.setDepthWriteEnabled(false);

        let state = device
            .newDepthStencilStateWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create UI depth stencil state".to_string())?;

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

    pub fn render(
        &mut self,
        scene: &Scene,
        ui_renderer: Option<&UIRenderer>,
    ) -> Result<(), String> {
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

        // Define sky colors
        let horizon_color = Vec3::new(0.7, 0.8, 0.9); // Light blue-gray
        let zenith_color = Vec3::new(0.2, 0.4, 0.8); // Deeper blue

        unsafe {
            color_attachment.setTexture(Some(&drawable.texture()));
            color_attachment.setLoadAction(MTLLoadAction::Clear);

            // Simple sky gradient based on camera up direction
            let camera_up = self.camera.up_vector();
            let up_y = camera_up.y.clamp(-1.0, 1.0);

            let t = (up_y + 1.0) * 0.5; // Map from [-1, 1] to [0, 1]
            let red = horizon_color.x + (zenith_color.x - horizon_color.x) * t;
            let green = horizon_color.y + (zenith_color.y - horizon_color.y) * t;
            let blue = horizon_color.z + (zenith_color.z - horizon_color.z) * t;

            color_attachment.setClearColor(MTLClearColor {
                red: red as f64,
                green: green as f64,
                blue: blue as f64,
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

            // Render skybox first
            if let Some(skybox_buffers) = &self.skybox_buffers {
                render_encoder.setRenderPipelineState(&self.skybox_pipeline_state);
                render_encoder.setDepthStencilState(Some(&self.skybox_depth_stencil_state));

                // Update skybox uniforms
                let skybox_uniforms = SkyboxUniforms {
                    view_projection_matrix: self.camera.view_projection_matrix(),
                    camera_pos: self.camera.position(),
                    time: self.time,
                    sun_direction: Vec3::new(0.5, 0.8, 0.3).normalize(),
                    _padding: 0.0,
                };

                // Safety: The uniform buffer was created with at least sizeof(SkyboxUniforms) bytes.
                unsafe {
                    let contents = skybox_buffers.uniform_buffer.contents();
                    std::ptr::copy_nonoverlapping(
                        &raw const skybox_uniforms,
                        contents.as_ptr().cast::<SkyboxUniforms>(),
                        1,
                    );
                }

                unsafe {
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&skybox_buffers.vertex_buffer),
                        0,
                        0,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&skybox_buffers.uniform_buffer),
                        0,
                        1,
                    );
                    render_encoder.setFragmentBuffer_offset_atIndex(
                        Some(&skybox_buffers.uniform_buffer),
                        0,
                        1,
                    );

                    render_encoder
                        .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset(
                            MTLPrimitiveType::Triangle,
                            skybox_buffers.index_count,
                            MTLIndexType::UInt16,
                            &skybox_buffers.index_buffer,
                            0,
                        );
                }
            }

            // Switch back to regular pipeline for scene objects
            render_encoder.setRenderPipelineState(&self.pipeline_state);
            render_encoder.setDepthStencilState(Some(&self.depth_stencil_state));

            // Render grass if available
            if let (Some(grass_buffers), Some(grass_pipeline)) =
                (&self.grass_buffers, &self.grass_pipeline_state)
            {
                render_encoder.setRenderPipelineState(grass_pipeline);

                // Update grass uniforms
                let grass_uniforms = Uniforms {
                    mvp_matrix: self.camera.view_projection_matrix(), // Not used in grass shader but keeping struct consistent
                    model_matrix: Mat4::identity(),                   // Not used
                    normal_matrix: Mat4::identity(),                  // Not used
                    view_pos: self.camera.position(),
                    time: self.time,
                    light_pos: scene.light.position,
                    _padding1: 0.0,
                    light_color: scene.light.color,
                    ambient_strength: scene.light.ambient,
                    diffuse_strength: scene.light.diffuse,
                    specular_strength: scene.light.specular,
                    fog_density: 0.02,
                    fog_color: Vec3::new(0.7, 0.8, 0.9),
                    fog_start: 10.0,
                    horizon_color,
                    _padding2: 0.0,
                    zenith_color,
                    _padding3: 0.0,
                };

                unsafe {
                    let contents = grass_buffers.uniform_buffer.contents();
                    std::ptr::copy_nonoverlapping(
                        &raw const grass_uniforms,
                        contents.as_ptr().cast::<Uniforms>(),
                        1,
                    );
                }

                unsafe {
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&grass_buffers.vertex_buffer),
                        0,
                        0,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&grass_buffers.uniform_buffer),
                        0,
                        1,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&grass_buffers.instance_buffer),
                        0,
                        2,
                    );

                    render_encoder.setFragmentBuffer_offset_atIndex(
                        Some(&grass_buffers.uniform_buffer),
                        0,
                        1,
                    );

                    // Use drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:
                    let _: () = msg_send![
                        &*render_encoder,
                        drawIndexedPrimitives: MTLPrimitiveType::Triangle,
                        indexCount: grass_buffers.index_count,
                        indexType: MTLIndexType::UInt16,
                        indexBuffer: &*grass_buffers.index_buffer,
                        indexBufferOffset: 0usize,
                        instanceCount: grass_buffers.instance_count
                    ];
                }

                // Switch back to regular pipeline
                render_encoder.setRenderPipelineState(&self.pipeline_state);
            }

            // Render road if available
            if let (Some(road_buffers), Some(road_pipeline)) =
                (&self.road_buffers, &self.road_pipeline_state)
            {
                render_encoder.setRenderPipelineState(road_pipeline);

                // Update road uniforms
                let road_uniforms = Uniforms {
                    mvp_matrix: self.camera.view_projection_matrix(),
                    model_matrix: Mat4::identity(),
                    normal_matrix: Mat4::identity(),
                    view_pos: self.camera.position(),
                    time: self.time,
                    light_pos: scene.light.position,
                    _padding1: 0.0,
                    light_color: scene.light.color,
                    ambient_strength: scene.light.ambient,
                    diffuse_strength: scene.light.diffuse,
                    specular_strength: scene.light.specular,
                    fog_density: 0.02,
                    fog_color: Vec3::new(0.7, 0.8, 0.9),
                    fog_start: 10.0,
                    horizon_color,
                    _padding2: 0.0,
                    zenith_color,
                    _padding3: 0.0,
                };

                unsafe {
                    let contents = road_buffers.uniform_buffer.contents();
                    std::ptr::copy_nonoverlapping(
                        &raw const road_uniforms,
                        contents.as_ptr().cast::<Uniforms>(),
                        1,
                    );
                }

                unsafe {
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&road_buffers.vertex_buffer),
                        0,
                        0,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&road_buffers.uniform_buffer),
                        0,
                        1,
                    );
                    render_encoder.setFragmentBuffer_offset_atIndex(
                        Some(&road_buffers.uniform_buffer),
                        0,
                        1,
                    );

                    // Set default texture for road (will be replaced with actual texture later)
                    render_encoder
                        .setFragmentTexture_atIndex(Some(&self.default_texture.texture), 0);
                    render_encoder.setFragmentSamplerState_atIndex(Some(&self.sampler_state), 0);

                    render_encoder
                        .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset(
                            MTLPrimitiveType::Triangle,
                            road_buffers.index_count,
                            MTLIndexType::UInt16,
                            &road_buffers.index_buffer,
                            0,
                        );
                }

                // Switch back to regular pipeline
                render_encoder.setRenderPipelineState(&self.pipeline_state);
            }

            // Render trees if available
            if let (Some(tree_buffers), Some(tree_pipeline)) =
                (&self.tree_buffers, &self.tree_pipeline_state)
            {
                render_encoder.setRenderPipelineState(tree_pipeline);

                // Update tree uniforms (using the new shader uniforms structure)
                #[repr(C)]
                struct TreeUniforms {
                    view_matrix: Mat4,
                    projection_matrix: Mat4,
                    light_position: Vec3,
                    time: f32,
                    view_position: Vec3,
                    _padding: f32,
                    sky_gradient_bottom: Vec4,
                    sky_gradient_top: Vec4,
                    sun_direction: Vec3,
                    fog_density: f32,
                    fog_start: f32,
                    _padding2: [f32; 3],
                }

                let tree_uniforms = TreeUniforms {
                    view_matrix: self.camera.view_matrix(),
                    projection_matrix: self.camera.projection_matrix(),
                    light_position: scene.light.position,
                    time: self.time,
                    view_position: self.camera.position(),
                    _padding: 0.0,
                    sky_gradient_bottom: Vec4::new(
                        horizon_color.x,
                        horizon_color.y,
                        horizon_color.z,
                        1.0,
                    ),
                    sky_gradient_top: Vec4::new(
                        zenith_color.x,
                        zenith_color.y,
                        zenith_color.z,
                        1.0,
                    ),
                    sun_direction: Vec3::new(0.5, 0.8, 0.3).normalize(),
                    fog_density: 0.02,
                    fog_start: 10.0,
                    _padding2: [0.0, 0.0, 0.0],
                };

                unsafe {
                    let contents = tree_buffers.uniform_buffer.contents();
                    std::ptr::copy_nonoverlapping(
                        &raw const tree_uniforms,
                        contents.as_ptr().cast::<TreeUniforms>(),
                        1,
                    );
                }

                unsafe {
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&tree_buffers.vertex_buffer),
                        0,
                        0,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&tree_buffers.uniform_buffer),
                        0,
                        1,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(&tree_buffers.instance_buffer),
                        0,
                        2,
                    );
                    render_encoder.setFragmentBuffer_offset_atIndex(
                        Some(&tree_buffers.uniform_buffer),
                        0,
                        1,
                    );

                    // Draw instanced trees
                    let _: () = msg_send![
                        &*render_encoder,
                        drawIndexedPrimitives: MTLPrimitiveType::Triangle,
                        indexCount: tree_buffers.index_count,
                        indexType: MTLIndexType::UInt16,
                        indexBuffer: &*tree_buffers.index_buffer,
                        indexBufferOffset: 0_usize,
                        instanceCount: tree_buffers.instance_count,
                    ];
                }

                // Switch back to regular pipeline
                render_encoder.setRenderPipelineState(&self.pipeline_state);
            }

            // Render all nodes in the scene
            scene.traverse(|node, world_transform| {
                if let Some(mesh) = &node.mesh {
                    let mesh_ptr = mesh as *const Mesh;
                    if let Some(buffers) = self.mesh_buffers.get(&mesh_ptr) {
                        // Update uniforms with MVP matrix and lighting data for this node
                        let view_proj = self.camera.view_projection_matrix();
                        let mvp_matrix = view_proj.multiply(world_transform);

                        // Calculate normal matrix (transpose of inverse of model matrix)
                        // For now, we'll use the model matrix directly since we're only using uniform scaling
                        let normal_matrix = world_transform.clone();
                        let uniforms = Uniforms {
                            mvp_matrix,
                            model_matrix: world_transform.clone(),
                            normal_matrix,
                            view_pos: self.camera.position(),
                            time: self.time,
                            light_pos: scene.light.position,
                            _padding1: 0.0,
                            light_color: scene.light.color,
                            ambient_strength: scene.light.ambient,
                            diffuse_strength: scene.light.diffuse,
                            specular_strength: scene.light.specular,
                            fog_density: 0.02,
                            fog_color: Vec3::new(0.7, 0.8, 0.9), // Base fog color
                            fog_start: 10.0,
                            horizon_color,
                            _padding2: 0.0,
                            zenith_color,
                            _padding3: 0.0,
                        };
                        // Safety: The uniform buffer was created with at least sizeof(Uniforms) bytes.
                        // The buffer contents pointer is valid for the lifetime of the buffer.
                        // We're copying exactly one Uniforms struct which matches the buffer size.
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
                            render_encoder.setFragmentBuffer_offset_atIndex(Some(&buffers.uniform_buffer), 0, 1);

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

            // Render UI overlay if provided
            if let Some(ui_renderer) = ui_renderer {
                // Switch to UI pipeline
                render_encoder.setRenderPipelineState(&self.ui_pipeline_state);

                // Use UI depth stencil state (no depth write, always pass depth test)
                render_encoder.setDepthStencilState(Some(&self.ui_depth_stencil_state));

                // Bind UI buffers
                unsafe {
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(ui_renderer.vertex_buffer()),
                        0,
                        0,
                    );
                    render_encoder.setVertexBuffer_offset_atIndex(
                        Some(ui_renderer.uniform_buffer()),
                        0,
                        1,
                    );

                    // Bind font texture and sampler
                    render_encoder.setFragmentTexture_atIndex(Some(ui_renderer.font_texture()), 0);
                    render_encoder.setFragmentSamplerState_atIndex(Some(&self.sampler_state), 0);

                    // Issue draw call for UI
                    if ui_renderer.index_count() > 0 {
                        render_encoder
                            .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset(
                                MTLPrimitiveType::Triangle,
                                ui_renderer.index_count(),
                                MTLIndexType::UInt16,
                                ui_renderer.index_buffer(),
                                0,
                            );
                    }
                }
            }

            render_encoder.endEncoding();
        }

        // Safety: The drawable is a valid CAMetalDrawable that conforms to MTLDrawable protocol.
        // The cast is safe because CAMetalDrawable implements MTLDrawable.
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

    pub fn initialize_skybox(&mut self, skybox: &crate::core::Skybox) -> Result<(), String> {
        let vertex_buffer = Self::create_vertex_buffer(&self.device, &skybox.mesh)?;
        let index_buffer = Self::create_index_buffer(&self.device, &skybox.mesh)?;
        let uniform_buffer = self
            .device
            .newBufferWithLength_options(
                std::mem::size_of::<SkyboxUniforms>(),
                MTLResourceOptions::empty(),
            )
            .ok_or_else(|| "Failed to create skybox uniform buffer".to_string())?;

        self.skybox_buffers = Some(MeshBuffers {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            index_count: skybox.mesh.indices.len(),
        });

        Ok(())
    }

    pub fn update_time(&mut self, delta_time: f32) {
        self.time += delta_time;
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn device(&self) -> &ProtocolObject<dyn MTLDevice> {
        &self.device
    }

    pub fn initialize_grass(&mut self, grass_system: &GrassSystem) -> Result<(), String> {
        // Create grass pipeline if not already created
        if self.grass_pipeline_state.is_none() {
            self.grass_pipeline_state = Some(Self::create_grass_pipeline_state(&self.device)?);
        }

        let instanced_mesh = grass_system.instanced_mesh();

        // Create buffers
        let vertex_buffer = Self::create_vertex_buffer(&self.device, &instanced_mesh.base_mesh)?;
        let index_buffer = Self::create_index_buffer(&self.device, &instanced_mesh.base_mesh)?;

        // Create instance buffer
        let instance_data = instanced_mesh.instances.as_slice();
        let instance_buffer_size = std::mem::size_of_val(instance_data);

        let instance_data_ptr =
            std::ptr::NonNull::new(instance_data.as_ptr().cast::<std::ffi::c_void>().cast_mut())
                .ok_or_else(|| "Failed to create NonNull pointer for instance data".to_string())?;

        let instance_buffer = unsafe {
            self.device.newBufferWithBytes_length_options(
                instance_data_ptr,
                instance_buffer_size,
                MTLResourceOptions::empty(),
            )
        }
        .ok_or_else(|| "Failed to create instance buffer".to_string())?;

        // Create uniform buffer for grass
        let uniform_buffer = self
            .device
            .newBufferWithLength_options(
                std::mem::size_of::<Uniforms>(),
                MTLResourceOptions::empty(),
            )
            .ok_or_else(|| "Failed to create grass uniform buffer".to_string())?;

        self.grass_buffers = Some(GrassBuffers {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            index_count: instanced_mesh.base_mesh.indices.len(),
            instance_count: instanced_mesh.instances.len(),
        });

        Ok(())
    }

    pub fn initialize_road(&mut self, road_system: &crate::core::RoadSystem) -> Result<(), String> {
        // Create road pipeline if not already created
        if self.road_pipeline_state.is_none() {
            self.road_pipeline_state = Some(Self::create_road_pipeline_state(&self.device)?);
        }

        let mesh = road_system.mesh();

        // Create buffers
        let vertex_buffer = Self::create_vertex_buffer(&self.device, mesh)?;
        let index_buffer = Self::create_index_buffer(&self.device, mesh)?;
        let uniform_buffer = Self::create_uniform_buffer(&self.device)?;

        self.road_buffers = Some(MeshBuffers {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            index_count: mesh.indices.len(),
        });

        Ok(())
    }

    pub fn initialize_tree(&mut self, tree_system: &crate::core::TreeSystem) -> Result<(), String> {
        // Create tree pipeline if not already created
        if self.tree_pipeline_state.is_none() {
            self.tree_pipeline_state = Some(Self::create_tree_pipeline_state(&self.device)?);
        }

        let instanced_mesh = tree_system.instanced_mesh();

        // Create buffers
        let vertex_buffer = Self::create_vertex_buffer(&self.device, &instanced_mesh.base_mesh)?;
        let index_buffer = Self::create_index_buffer(&self.device, &instanced_mesh.base_mesh)?;

        // Create instance buffer
        let instance_data = instanced_mesh.instances.as_slice();
        let instance_buffer_size = std::mem::size_of_val(instance_data);

        let instance_data_ptr =
            std::ptr::NonNull::new(instance_data.as_ptr().cast::<std::ffi::c_void>().cast_mut())
                .ok_or_else(|| "Failed to create NonNull pointer for instance data".to_string())?;

        let instance_buffer = unsafe {
            self.device.newBufferWithBytes_length_options(
                instance_data_ptr,
                instance_buffer_size,
                MTLResourceOptions::empty(),
            )
        }
        .ok_or_else(|| "Failed to create instance buffer".to_string())?;

        // Create uniform buffer for tree
        let uniform_buffer = self
            .device
            .newBufferWithLength_options(
                std::mem::size_of::<Uniforms>(),
                MTLResourceOptions::empty(),
            )
            .ok_or_else(|| "Failed to create tree uniform buffer".to_string())?;

        self.tree_buffers = Some(GrassBuffers {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            index_count: instanced_mesh.base_mesh.indices.len(),
            instance_count: instanced_mesh.instances.len(),
        });

        Ok(())
    }
}
