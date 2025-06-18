#include <metal_stdlib>
using namespace metal;

struct CullingUniforms {
    float4x4 view_projection_matrix;
    float3 camera_position;
    float cull_distance;
    float4 frustum_planes[6]; // Left, Right, Bottom, Top, Near, Far
};

struct InstanceInput {
    float4x4 transform;
    float3 color_variation;
    uint lod_level;
    uint texture_index;
    uint _padding[3];
};

struct DrawArguments {
    uint index_count;
    uint instance_count;
    uint first_index;
    int base_vertex;
    uint first_instance;
};

// Check if a sphere is inside the frustum
bool sphere_in_frustum(float3 center, float radius, constant float4* frustum_planes) {
    for (int i = 0; i < 6; i++) {
        float4 plane = frustum_planes[i];
        float distance = dot(plane.xyz, center) + plane.w;
        if (distance < -radius) {
            return false;
        }
    }
    return true;
}

kernel void cull_instances(
    device const InstanceInput* instances [[buffer(0)]],
    constant CullingUniforms& uniforms [[buffer(1)]],
    device InstanceInput* visible_instances [[buffer(2)]],
    device DrawArguments* draw_args [[buffer(3)]],
    constant uint& instance_count [[buffer(4)]],
    uint thread_id [[thread_position_in_grid]]
) {
    if (thread_id >= instance_count) {
        return;
    }
    
    // Get instance data
    InstanceInput instance = instances[thread_id];
    
    // Extract position from transform matrix (column 3)
    float3 position = instance.transform[3].xyz;
    
    // Distance culling
    float3 to_camera = position - uniforms.camera_position;
    float distance = length(to_camera);
    
    if (distance > uniforms.cull_distance) {
        return;
    }
    
    // Frustum culling with bounding sphere
    // Estimate bounding radius based on grass blade size
    float bounding_radius = 1.0; // Adjust based on actual grass size
    
    if (!sphere_in_frustum(position, bounding_radius, uniforms.frustum_planes)) {
        return;
    }
    
    // Get LOD level for this instance
    uint lod_level = instance.lod_level;
    
    // Atomically increment instance count for this LOD level
    uint visible_index = atomic_fetch_add_explicit(
        &draw_args[lod_level].instance_count,
        1,
        memory_order_relaxed
    );
    
    // Store the visible instance at the appropriate offset
    // Each LOD level gets a section of the visible instances buffer
    uint lod_offset = lod_level * (instance_count / 4); // Assume even distribution
    visible_instances[lod_offset + visible_index] = instance;
}

// Reset draw arguments before culling
kernel void reset_draw_arguments(
    device DrawArguments* draw_args [[buffer(0)]],
    uint thread_id [[thread_position_in_grid]]
) {
    if (thread_id < 4) { // 4 LOD levels
        draw_args[thread_id].instance_count = 0;
        draw_args[thread_id].first_instance = thread_id * 16384; // Offset for each LOD
        
        // These values depend on the actual mesh data
        // They should be set based on the LOD mesh properties
        switch (thread_id) {
            case 0: // Full LOD
                draw_args[thread_id].index_count = 60; // 5 segments * 2 triangles * 3 indices
                break;
            case 1: // Reduced LOD
                draw_args[thread_id].index_count = 24; // 2 segments * 2 triangles * 3 indices
                break;
            case 2: // Billboard LOD
            case 3: // Fade LOD
                draw_args[thread_id].index_count = 6; // 1 quad * 2 triangles * 3 indices
                break;
        }
        draw_args[thread_id].first_index = 0;
        draw_args[thread_id].base_vertex = 0;
    }
}