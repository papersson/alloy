#include <metal_stdlib>
using namespace metal;

struct Vertex {
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

vertex VertexOut road_vertex(
    Vertex in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    VertexOut out;
    
    // Transform vertex position
    float4 world_pos = uniforms.model_matrix * float4(in.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.position = uniforms.mvp_matrix * float4(in.position, 1.0);
    
    // Transform normal
    out.normal = normalize((uniforms.normal_matrix * float4(in.normal, 0.0)).xyz);
    
    // Pass through texture coordinates
    out.tex_coord = in.tex_coord;
    
    return out;
}

fragment float4 road_fragment(
    VertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]],
    texture2d<float> road_texture [[texture(0)]],
    sampler texture_sampler [[sampler(0)]]
) {
    // Sample road texture
    float4 texture_color = road_texture.sample(texture_sampler, in.tex_coord);
    
    // If no texture bound, use a default dirt color
    if (texture_color.a == 0.0) {
        texture_color = float4(0.5, 0.35, 0.2, 1.0); // Brown dirt color
    }
    
    // Calculate lighting
    float3 normal = normalize(in.normal);
    float3 light_dir = normalize(uniforms.light_pos - in.world_pos);
    float3 view_dir = normalize(uniforms.view_pos - in.world_pos);
    
    // Ambient
    float3 ambient = uniforms.ambient_strength * uniforms.light_color;
    
    // Diffuse
    float diff = max(dot(normal, light_dir), 0.0);
    float3 diffuse = uniforms.diffuse_strength * diff * uniforms.light_color;
    
    // Specular (reduced for road material)
    float3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 16.0); // Lower shininess for road
    float3 specular = uniforms.specular_strength * 0.2 * spec * uniforms.light_color; // Reduced specular
    
    // Combine lighting
    float3 lighting = ambient + diffuse + specular;
    float3 result = texture_color.rgb * lighting;
    
    // Apply fog
    float distance = length(in.world_pos - uniforms.view_pos);
    float fog_distance = max(distance - uniforms.fog_start, 0.0);
    float fog_factor = 1.0 - exp(-uniforms.fog_density * fog_distance);
    fog_factor = clamp(fog_factor, 0.0, 1.0);
    
    // Mix with fog color
    result = mix(result, uniforms.fog_color, fog_factor);
    
    return float4(result, texture_color.a);
}