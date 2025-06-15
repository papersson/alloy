#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float4x4 mvp_matrix;
    float4x4 model_matrix;
    float4x4 normal_matrix;
    float3 view_pos;
    float _padding0;
    float3 light_pos;
    float _padding1;
    float3 light_color;
    float ambient_strength;
    float diffuse_strength;
    float specular_strength;
    float2 _padding2;
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
    // Sample texture
    float4 object_color = tex.sample(tex_sampler, in.tex_coord);
    
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
    
    return float4(result, object_color.a);
}