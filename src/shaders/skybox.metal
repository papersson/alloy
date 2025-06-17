#include <metal_stdlib>
using namespace metal;

struct SkyboxUniforms {
    float4x4 view_projection_matrix;
    float3 camera_pos;
    float time;
    float3 sun_direction;
    float _padding;
};

struct VertexIn {
    float3 position [[attribute(0)]];
    float2 tex_coord [[attribute(1)]];
    float3 normal [[attribute(2)]];
};

struct VertexOut {
    float4 position [[position]];
    float3 world_pos;
    float2 tex_coord;
};

vertex VertexOut skybox_vertex(
    VertexIn in [[stage_in]],
    constant SkyboxUniforms& uniforms [[buffer(1)]]
) {
    VertexOut out;
    // Position the skybox centered at the camera
    float3 sky_pos = in.position + uniforms.camera_pos;
    out.position = uniforms.view_projection_matrix * float4(sky_pos, 1.0);
    out.world_pos = in.position; // Use local position for sky calculations
    out.tex_coord = in.tex_coord;
    return out;
}

// Noise functions for procedural clouds
float hash(float2 p) {
    float3 p3 = fract(float3(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

float noise(float2 p) {
    float2 i = floor(p);
    float2 f = fract(p);
    
    float a = hash(i);
    float b = hash(i + float2(1.0, 0.0));
    float c = hash(i + float2(0.0, 1.0));
    float d = hash(i + float2(1.0, 1.0));
    
    float2 u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

float fbm(float2 p) {
    float value = 0.0;
    float amplitude = 0.5;
    float frequency = 1.0;
    
    for (int i = 0; i < 4; i++) {
        value += amplitude * noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

fragment float4 skybox_fragment(
    VertexOut in [[stage_in]],
    constant SkyboxUniforms& uniforms [[buffer(1)]]
) {
    // Normalize the world position to get the sky direction
    float3 sky_dir = normalize(in.world_pos);
    
    // Calculate base sky gradient
    float up_dot = sky_dir.y;
    float horizon_blend = smoothstep(-0.1, 0.3, up_dot);
    
    // Sky colors
    float3 horizon_color = float3(0.7, 0.8, 0.9);
    float3 zenith_color = float3(0.2, 0.4, 0.8);
    float3 ground_color = float3(0.15, 0.2, 0.25);
    
    // Blend between ground and sky
    float3 base_color;
    if (up_dot < 0.0) {
        base_color = mix(ground_color, horizon_color, smoothstep(-0.5, 0.0, up_dot));
    } else {
        base_color = mix(horizon_color, zenith_color, horizon_blend);
    }
    
    // Add sun glow
    float sun_dot = max(0.0, dot(sky_dir, uniforms.sun_direction));
    float sun_glow = pow(sun_dot, 128.0) * 2.0 + pow(sun_dot, 8.0) * 0.5;
    base_color += float3(1.0, 0.9, 0.7) * sun_glow;
    
    // Calculate cloud coverage using procedural noise
    if (up_dot > 0.0) {
        // Project onto a plane for cloud texture coordinates
        float2 cloud_uv = sky_dir.xz / (1.0 + sky_dir.y * 0.5);
        cloud_uv *= 3.0;
        cloud_uv += uniforms.time * 0.02; // Slow cloud movement
        
        // Generate cloud pattern
        float cloud_base = fbm(cloud_uv);
        float cloud_detail = fbm(cloud_uv * 3.0 + uniforms.time * 0.05);
        float clouds = cloud_base * 0.7 + cloud_detail * 0.3;
        
        // Shape the clouds
        clouds = smoothstep(0.4, 0.6, clouds);
        clouds *= smoothstep(0.0, 0.3, up_dot); // Fade clouds near horizon
        
        // Apply clouds to sky
        float3 cloud_color = float3(1.0, 1.0, 1.0);
        base_color = mix(base_color, cloud_color, clouds * 0.7);
    }
    
    return float4(base_color, 1.0);
}