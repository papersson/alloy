use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::msg_send;
use objc2_foundation::NSString;
use objc2_core_foundation::CGSize;
use objc2_metal::{
    MTLClearColor, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue, MTLCreateSystemDefaultDevice, 
    MTLDevice, MTLDrawable, MTLLoadAction, MTLPixelFormat, MTLRenderPassDescriptor, MTLStoreAction,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use winit::raw_window_handle::RawWindowHandle;

pub struct Renderer {
    #[allow(dead_code)]
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    layer: Retained<CAMetalLayer>,
}

impl Renderer {
    pub fn new(window_handle: RawWindowHandle) -> Result<Self, String> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| "Failed to get default Metal device".to_string())?;

        let command_queue = device
            .newCommandQueue()
            .ok_or_else(|| "Failed to create command queue".to_string())?;

        let layer = Self::create_metal_layer(&device, window_handle)?;

        Ok(Self {
            device,
            command_queue,
            layer,
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
            let label = NSString::from_str("Clear Pass");
            render_encoder.setLabel(Some(&label));
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