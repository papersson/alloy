#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float4x4 view_matrix;
    float4x4 projection_matrix;
    float3 light_position;
    float time;
    float3 view_position;
    float _padding;
    float4 sky_gradient_bottom;
    float4 sky_gradient_top;
    float3 sun_direction;
    float fog_density;
    float fog_start;
    float _padding2[3];
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
    float3 world_position;
    float3 normal;
    float2 tex_coord;
    float3 color;
    float distance_to_camera;
};

vertex VertexOut tree_vertex(
    VertexIn in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]],
    constant InstanceData& instance [[buffer(2)]],
    uint vertex_id [[vertex_id]]
) {
    VertexOut out;
    
    // Transform vertex position by instance transform
    float4 world_pos = instance.transform * float4(in.position, 1.0);
    
    // Apply wind animation
    float wind_strength = 0.3;
    float wind_frequency = 1.0;
    
    // Only apply wind to vertices above ground (y > 0.1 in model space)
    if (in.position.y > 0.1) {
        // Use world position for wind offset to create wave effect
        float wind_phase = world_pos.x * 0.1 + world_pos.z * 0.1 + uniforms.time * wind_frequency;
        
        // Stronger effect at the top
        float height_factor = in.position.y / 2.5; // Normalize by max tree height
        
        // Apply wind displacement
        float2 wind_offset = float2(sin(wind_phase), cos(wind_phase * 0.7)) * wind_strength * height_factor;
        
        // Different sway for trunk (lower vertices) vs foliage (higher vertices)
        if (in.position.y < 1.0) {
            // Trunk: gentle bend
            wind_offset *= 0.3;
        } else {
            // Foliage: more pronounced wobble
            wind_offset *= 1.0 + sin(uniforms.time * 3.0 + world_pos.x) * 0.2;
        }
        
        world_pos.x += wind_offset.x;
        world_pos.z += wind_offset.y;
    }
    
    out.world_position = world_pos.xyz;
    out.position = uniforms.projection_matrix * uniforms.view_matrix * world_pos;
    
    // Transform normal by instance rotation (upper-left 3x3 of transform)
    float3x3 normal_matrix = float3x3(
        instance.transform[0].xyz,
        instance.transform[1].xyz,
        instance.transform[2].xyz
    );
    out.normal = normalize(normal_matrix * in.normal);
    
    out.tex_coord = in.tex_coord;
    
    // Calculate distance for fog
    out.distance_to_camera = length(uniforms.view_position - out.world_position);
    
    // Determine color based on vertex height (trunk vs foliage)
    if (in.position.y < 1.0) {
        // Trunk: brown color with slight variation
        out.color = float3(0.4, 0.25, 0.1) + instance.color_variation * 0.1;
    } else {
        // Foliage: green color with variation
        float3 base_green = float3(0.1, 0.5, 0.1);
        out.color = base_green + float3(0.0, instance.color_variation.g * 0.2, 0.0);
    }
    
    return out;
}

fragment float4 tree_fragment(
    VertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Basic lighting
    float3 light_dir = normalize(uniforms.light_position - in.world_position);
    float3 normal = normalize(in.normal);
    
    // Diffuse lighting
    float diffuse = max(dot(normal, light_dir), 0.0);
    
    // Ambient light
    float3 ambient = float3(0.3, 0.3, 0.3);
    
    // Final color with lighting
    float3 final_color = in.color * (ambient + diffuse * 0.7);
    
    // Apply fog
    float fog_distance = max(in.distance_to_camera - uniforms.fog_start, 0.0);
    float fog_factor = 1.0 - exp(-uniforms.fog_density * fog_distance);
    fog_factor = clamp(fog_factor, 0.0, 1.0);
    
    // Get fog color from sky gradient (use bottom color for fog)
    float3 fog_color = uniforms.sky_gradient_bottom.rgb;
    final_color = mix(final_color, fog_color, fog_factor);
    
    return float4(final_color, 1.0);
}