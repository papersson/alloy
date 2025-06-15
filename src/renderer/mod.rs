use crate::math::Vec3;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGSize;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBuffer, MTLClearColor, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue,
    MTLCompileOptions, MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable, MTLLibrary,
    MTLLoadAction, MTLPixelFormat, MTLPrimitiveType, MTLRenderCommandEncoder,
    MTLRenderPassDescriptor, MTLRenderPipelineDescriptor, MTLRenderPipelineState,
    MTLResourceOptions, MTLStoreAction,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use winit::raw_window_handle::RawWindowHandle;

pub struct Renderer {
    #[allow(dead_code)]
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    layer: Retained<CAMetalLayer>,
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    #[allow(dead_code)]
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
}

impl Renderer {
    pub fn new(window_handle: RawWindowHandle) -> Result<Self, String> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| "Failed to get default Metal device".to_string())?;

        let command_queue = device
            .newCommandQueue()
            .ok_or_else(|| "Failed to create command queue".to_string())?;

        let layer = Self::create_metal_layer(&device, window_handle)?;

        let vertex_buffer = Self::create_vertex_buffer(&device)?;
        let pipeline_state = Self::create_pipeline_state(&device)?;

        Ok(Self {
            device,
            command_queue,
            layer,
            pipeline_state,
            vertex_buffer,
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
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, String> {
        let vertices = [
            Vec3::new(0.0, 0.5, 0.0),   // Top
            Vec3::new(-0.5, -0.5, 0.0), // Bottom left
            Vec3::new(0.5, -0.5, 0.0),  // Bottom right
        ];

        let vertex_data = vertices.as_ptr().cast::<std::ffi::c_void>();
        let vertex_data_size = std::mem::size_of_val(&vertices);

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

    fn create_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
        let shader_source = include_str!("../shaders/triangle.metal");
        let source_string = NSString::from_str(shader_source);
        let compile_options = MTLCompileOptions::new();

        let library = device
            .newLibraryWithSource_options_error(&source_string, Some(&compile_options))
            .map_err(|e| format!("Failed to compile shaders: {e:?}"))?;

        let vertex_fn_name = NSString::from_str("triangle_vertex");
        let vertex_function = library
            .newFunctionWithName(&vertex_fn_name)
            .ok_or_else(|| "Failed to find vertex function".to_string())?;

        let fragment_fn_name = NSString::from_str("triangle_fragment");
        let fragment_function = library
            .newFunctionWithName(&fragment_fn_name)
            .ok_or_else(|| "Failed to find fragment function".to_string())?;

        let pipeline_descriptor = MTLRenderPipelineDescriptor::new();
        unsafe {
            pipeline_descriptor.setVertexFunction(Some(&vertex_function));
            pipeline_descriptor.setFragmentFunction(Some(&fragment_function));

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

    pub fn render(&self) -> Result<(), String> {
        let drawable = unsafe { self.layer.nextDrawable() }
            .ok_or_else(|| "Failed to get next drawable".to_string())?;

        let command_buffer = self
            .command_queue
            .commandBuffer()
            .ok_or_else(|| "Failed to create command buffer".to_string())?;

        let label = NSString::from_str("Main Render Pass");
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

        if let Some(render_encoder) =
            command_buffer.renderCommandEncoderWithDescriptor(&render_pass_descriptor)
        {
            let label = NSString::from_str("Triangle Pass");
            render_encoder.setLabel(Some(&label));

            render_encoder.setRenderPipelineState(&self.pipeline_state);
            // No vertex buffer needed - vertices defined in shader

            unsafe {
                render_encoder.drawPrimitives_vertexStart_vertexCount(
                    MTLPrimitiveType::Triangle,
                    0,
                    3,
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

    pub fn update_drawable_size(&self, width: u32, height: u32) {
        let size = CGSize {
            width: f64::from(width),
            height: f64::from(height),
        };
        unsafe {
            self.layer.setDrawableSize(size);
        }
    }
}
