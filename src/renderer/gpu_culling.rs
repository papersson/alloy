//! GPU-driven culling system for efficient vegetation rendering

use crate::math::{Mat4, Vec3, Vec4};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBuffer, MTLCommandBuffer, MTLCommandEncoder, MTLComputeCommandEncoder,
    MTLComputePipelineState, MTLDevice, MTLLibrary, MTLResourceOptions, MTLSize,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CullingUniforms {
    pub view_projection_matrix: Mat4,
    pub camera_position: Vec3,
    pub cull_distance: f32,
    pub frustum_planes: [Vec4; 6], // Left, Right, Bottom, Top, Near, Far
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceInput {
    pub transform: Mat4,
    pub color_variation: Vec3,
    pub lod_level: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawArguments {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

pub struct GpuCullingSystem {
    cull_pipeline_state: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    culling_uniforms_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    draw_arguments_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    visible_instances_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
}

impl GpuCullingSystem {
    /// Creates a new GPU culling system
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Compute shader compilation fails
    /// - Buffer creation fails
    #[must_use]
    pub fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        max_instances: usize,
    ) -> Result<Self, String> {
        // Create compute pipeline for culling
        let cull_pipeline_state = Self::create_cull_pipeline_state(device)?;

        // Create buffers
        let culling_uniforms_buffer = device
            .newBufferWithLength_options(
                std::mem::size_of::<CullingUniforms>(),
                MTLResourceOptions::empty(),
            )
            .ok_or_else(|| "Failed to create culling uniforms buffer".to_string())?;

        // 4 LOD levels × draw arguments
        let draw_arguments_buffer = device
            .newBufferWithLength_options(
                std::mem::size_of::<DrawArguments>() * 4,
                MTLResourceOptions::empty(),
            )
            .ok_or_else(|| "Failed to create draw arguments buffer".to_string())?;

        let visible_instances_buffer = device
            .newBufferWithLength_options(
                std::mem::size_of::<InstanceInput>() * max_instances,
                MTLResourceOptions::empty(),
            )
            .ok_or_else(|| "Failed to create visible instances buffer".to_string())?;

        Ok(Self {
            cull_pipeline_state,
            culling_uniforms_buffer,
            draw_arguments_buffer,
            visible_instances_buffer,
        })
    }

    fn create_cull_pipeline_state(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Result<Retained<ProtocolObject<dyn MTLComputePipelineState>>, String> {
        let shader_source = include_str!("../shaders/cull_instances.metal");
        let shader_source = NSString::from_str(shader_source);

        let library = device
            .newLibraryWithSource_options_error(&shader_source, None)
            .map_err(|e| format!("Failed to compile culling compute shader: {:?}", e))?;

        let function = library
            .newFunctionWithName(&NSString::from_str("cull_instances"))
            .ok_or_else(|| "Failed to find cull_instances function".to_string())?;

        let pipeline_state = device
            .newComputePipelineStateWithFunction_error(&function)
            .map_err(|e| format!("Failed to create culling compute pipeline state: {:?}", e))?;

        Ok(pipeline_state)
    }

    pub fn update_culling_uniforms(
        &self,
        view_projection: &Mat4,
        camera_position: &Vec3,
        cull_distance: f32,
    ) {
        // Extract frustum planes from view-projection matrix
        let frustum_planes = Self::extract_frustum_planes(view_projection);

        let uniforms = CullingUniforms {
            view_projection_matrix: *view_projection,
            camera_position: *camera_position,
            cull_distance,
            frustum_planes,
        };

        unsafe {
            let contents = self.culling_uniforms_buffer.contents();
            std::ptr::copy_nonoverlapping(
                &uniforms,
                contents.as_ptr().cast::<CullingUniforms>(),
                1,
            );
        }
    }

    fn extract_frustum_planes(view_projection: &Mat4) -> [Vec4; 6] {
        let m = view_projection;

        // Left plane
        let left = Vec4::new(
            m.cols[3].x + m.cols[0].x,
            m.cols[3].y + m.cols[0].y,
            m.cols[3].z + m.cols[0].z,
            m.cols[3].w + m.cols[0].w,
        )
        .normalize_plane();

        // Right plane
        let right = Vec4::new(
            m.cols[3].x - m.cols[0].x,
            m.cols[3].y - m.cols[0].y,
            m.cols[3].z - m.cols[0].z,
            m.cols[3].w - m.cols[0].w,
        )
        .normalize_plane();

        // Bottom plane
        let bottom = Vec4::new(
            m.cols[3].x + m.cols[1].x,
            m.cols[3].y + m.cols[1].y,
            m.cols[3].z + m.cols[1].z,
            m.cols[3].w + m.cols[1].w,
        )
        .normalize_plane();

        // Top plane
        let top = Vec4::new(
            m.cols[3].x - m.cols[1].x,
            m.cols[3].y - m.cols[1].y,
            m.cols[3].z - m.cols[1].z,
            m.cols[3].w - m.cols[1].w,
        )
        .normalize_plane();

        // Near plane
        let near = Vec4::new(
            m.cols[3].x + m.cols[2].x,
            m.cols[3].y + m.cols[2].y,
            m.cols[3].z + m.cols[2].z,
            m.cols[3].w + m.cols[2].w,
        )
        .normalize_plane();

        // Far plane
        let far = Vec4::new(
            m.cols[3].x - m.cols[2].x,
            m.cols[3].y - m.cols[2].y,
            m.cols[3].z - m.cols[2].z,
            m.cols[3].w - m.cols[2].w,
        )
        .normalize_plane();

        [left, right, bottom, top, near, far]
    }

    /// Executes GPU culling on the provided instances
    ///
    /// # Errors
    ///
    /// Returns an error if compute command encoder creation fails
    pub fn execute_culling(
        &self,
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        instance_buffer: &ProtocolObject<dyn MTLBuffer>,
        instance_count: usize,
    ) -> Result<(), String> {
        // Clear draw arguments
        unsafe {
            let draw_args = self.draw_arguments_buffer.contents();
            std::ptr::write_bytes(
                draw_args.as_ptr(),
                0,
                std::mem::size_of::<DrawArguments>() * 4,
            );
        }

        let compute_encoder = command_buffer
            .computeCommandEncoder()
            .ok_or_else(|| "Failed to create compute command encoder".to_string())?;

        compute_encoder.setComputePipelineState(&self.cull_pipeline_state);

        // Set buffers
        unsafe {
            compute_encoder.setBuffer_offset_atIndex(Some(instance_buffer), 0, 0);
            compute_encoder.setBuffer_offset_atIndex(Some(&self.culling_uniforms_buffer), 0, 1);
            compute_encoder.setBuffer_offset_atIndex(Some(&self.visible_instances_buffer), 0, 2);
            compute_encoder.setBuffer_offset_atIndex(Some(&self.draw_arguments_buffer), 0, 3);
        }

        // Set instance count
        #[allow(clippy::cast_possible_truncation)]
        let instance_count_u32 = instance_count as u32;
        unsafe {
            compute_encoder.setBytes_length_atIndex(
                std::ptr::NonNull::new(
                    (&instance_count_u32 as *const u32)
                        .cast::<std::ffi::c_void>()
                        .cast_mut(),
                )
                .unwrap(),
                std::mem::size_of::<u32>(),
                4,
            );
        }

        // Calculate thread groups
        let threads_per_threadgroup = MTLSize {
            width: 64,
            height: 1,
            depth: 1,
        };

        let threadgroups = MTLSize {
            width: (instance_count + 63) / 64,
            height: 1,
            depth: 1,
        };

        compute_encoder
            .dispatchThreadgroups_threadsPerThreadgroup(threadgroups, threads_per_threadgroup);

        compute_encoder.endEncoding();

        Ok(())
    }

    #[must_use]
    pub fn draw_arguments_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.draw_arguments_buffer
    }

    #[must_use]
    pub fn visible_instances_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.visible_instances_buffer
    }
}
