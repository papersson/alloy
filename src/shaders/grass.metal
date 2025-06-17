#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float4x4 mvp_matrix;
    float4x4 model_matrix;
    float4x4 normal_matrix;
    float3 view_pos;
    float time;
    float3 light_pos;
    float _padding1;
    float3 light_color;
    float ambient_strength;
    float diffuse_strength;
    float specular_strength;
    float fog_density;
    float3 fog_color;
    float fog_start;
    float3 horizon_color;
    float _padding2;
    float3 zenith_color;
    float _padding3;
};

struct InstanceData {
    float4x4 transform;
    float3 color_variation;
    float _padding;
};

struct VertexIn {
    float3 position [[attribute(0)]];
    float2 tex_coord [[attribute(1)]];
    float3 normal [[attribute(2)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 tex_coord;
    float3 world_pos;
    float3 normal;
    float3 color_variation;
};

vertex VertexOut grass_vertex(
    VertexIn in [[stage_in]],
    uint instance_id [[instance_id]],
    constant Uniforms& uniforms [[buffer(1)]],
    constant InstanceData* instances [[buffer(2)]]
) {
    VertexOut out;
    
    // Get instance data
    constant InstanceData& instance = instances[instance_id];
    
    // Apply wind animation to the vertex position
    float3 local_pos = in.position;
    if (in.tex_coord.y < 0.5) { // Only animate the top part of the grass
        float wind_strength = 0.05;
        float wind_speed = 2.0;
        
        // Use instance transform position for wind offset
        float3 world_pos = (instance.transform * float4(0, 0, 0, 1)).xyz;
        float wind_offset = world_pos.x * 0.1 + world_pos.z * 0.1;
        
        // Calculate wind displacement
        float wind_time = uniforms.time * wind_speed + wind_offset;
        float wind_x = sin(wind_time) * wind_strength;
        float wind_z = cos(wind_time * 0.7) * wind_strength * 0.5;
        
        // Apply wind based on height (more at the top)
        float height_factor = 1.0 - in.tex_coord.y;
        local_pos.x += wind_x * height_factor;
        local_pos.z += wind_z * height_factor;
    }
    
    // Transform to world space
    float4 world_pos = instance.transform * float4(local_pos, 1.0);
    out.world_pos = world_pos.xyz;
    
    // Transform to clip space
    out.position = uniforms.mvp_matrix * world_pos;
    
    // Transform normal
    out.normal = normalize((instance.transform * float4(in.normal, 0.0)).xyz);
    
    out.tex_coord = in.tex_coord;
    out.color_variation = instance.color_variation;
    
    return out;
}

fragment float4 grass_fragment(
    VertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Base grass color
    float3 base_color = float3(0.2, 0.5, 0.1); // Dark green
    
    // Apply color variation
    float3 grass_color = base_color + in.color_variation;
    
    // Add slight gradient from bottom to top
    float gradient = mix(0.7, 1.0, in.tex_coord.y);
    grass_color *= gradient;
    
    // Simple ambient + diffuse lighting
    float3 ambient = uniforms.ambient_strength * uniforms.light_color;
    
    float3 light_dir = normalize(uniforms.light_pos - in.world_pos);
    float diff = max(dot(in.normal, light_dir), 0.0);
    float3 diffuse = uniforms.diffuse_strength * diff * uniforms.light_color;
    
    // Combine lighting
    float3 result = (ambient + diffuse) * grass_color;
    
    // Apply fog
    float distance = length(uniforms.view_pos - in.world_pos);
    float fog_factor = 1.0 - exp(-uniforms.fog_density * max(0.0, distance - uniforms.fog_start));
    fog_factor = clamp(fog_factor, 0.0, 1.0);
    
    // Calculate sky gradient for fog color
    float3 normalized_view = normalize(in.world_pos - uniforms.view_pos);
    float up_dot = dot(normalized_view, float3(0.0, 1.0, 0.0));
    float sky_gradient = (up_dot + 1.0) * 0.5;
    float3 current_fog_color = mix(uniforms.horizon_color, uniforms.zenith_color, sky_gradient);
    
    result = mix(result, current_fog_color, fog_factor);
    
    return float4(result, 1.0);
}