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
};

vertex VertexOut cube_vertex(
    VertexIn in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    VertexOut out;
    out.position = uniforms.mvp_matrix * float4(in.position, 1.0);
    out.tex_coord = in.tex_coord;
    out.world_pos = (uniforms.model_matrix * float4(in.position, 1.0)).xyz;
    out.normal = normalize((uniforms.normal_matrix * float4(in.normal, 0.0)).xyz);
    return out;
}

fragment float4 cube_fragment(
    VertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]],
    sampler tex_sampler [[sampler(0)]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Use a natural earth-tone color for the planet surface
    float4 object_color = float4(0.4, 0.55, 0.3, 1.0); // Muted green color
    
    // Ambient lighting
    float3 ambient = uniforms.ambient_strength * uniforms.light_color;
    
    // Diffuse lighting
    float3 light_dir = normalize(uniforms.light_pos - in.world_pos);
    float diff = max(dot(in.normal, light_dir), 0.0);
    float3 diffuse = uniforms.diffuse_strength * diff * uniforms.light_color;
    
    // Specular lighting
    float3 view_dir = normalize(uniforms.view_pos - in.world_pos);
    float3 reflect_dir = reflect(-light_dir, in.normal);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    float3 specular = uniforms.specular_strength * spec * uniforms.light_color;
    
    // Combine all lighting components
    float3 result = (ambient + diffuse + specular) * object_color.rgb;
    
    // Calculate fog based on distance from camera
    float distance = length(uniforms.view_pos - in.world_pos);
    float fog_factor = 1.0 - exp(-uniforms.fog_density * max(0.0, distance - uniforms.fog_start));
    fog_factor = clamp(fog_factor, 0.0, 1.0);
    
    // Mix with fog color (using the dynamic sky color)
    // Calculate view direction for sky gradient
    float3 normalized_view = normalize(in.world_pos - uniforms.view_pos);
    float up_dot = dot(normalized_view, float3(0.0, 1.0, 0.0));
    float sky_gradient = (up_dot + 1.0) * 0.5; // Map from [-1,1] to [0,1]
    float3 current_fog_color = mix(uniforms.horizon_color, uniforms.zenith_color, sky_gradient);
    
    result = mix(result, current_fog_color, fog_factor);
    
    return float4(result, object_color.a);
}