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
    float fade_alpha;
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
    if (in.tex_coord.y < 0.8) { // Animate most of the grass blade
        float wind_strength = 0.1;  // Increased from 0.05
        float wind_speed = 1.5;     // Slightly slower
        
        // Use instance transform position for wind offset
        float3 world_pos = (instance.transform * float4(0, 0, 0, 1)).xyz;
        float wind_offset = world_pos.x * 0.1 + world_pos.z * 0.1;
        
        // Calculate multi-frequency wind displacement for more natural movement
        float wind_time = uniforms.time * wind_speed + wind_offset;
        float wind_time2 = uniforms.time * wind_speed * 0.37 + wind_offset * 1.3;
        
        // Primary wind wave
        float wind_x = sin(wind_time) * wind_strength;
        float wind_z = cos(wind_time * 0.7) * wind_strength * 0.5;
        
        // Secondary wind wave (gusty effect)
        wind_x += sin(wind_time2 * 2.3) * wind_strength * 0.3;
        wind_z += cos(wind_time2 * 1.9) * wind_strength * 0.2;
        
        // Apply wind based on height (more at the top)
        float height_factor = pow(1.0 - in.tex_coord.y, 2.0);
        local_pos.x += wind_x * height_factor;
        local_pos.z += wind_z * height_factor;
    }
    
    // Transform to world space
    float4 world_pos = instance.transform * float4(local_pos, 1.0);
    out.world_pos = world_pos.xyz;
    
    // Transform to clip space
    out.position = uniforms.mvp_matrix * world_pos;
    
    // Transform normal - for grass, we want two-sided lighting
    float3 transformed_normal = normalize((instance.transform * float4(in.normal, 0.0)).xyz);
    
    // Ensure normal faces towards camera for two-sided lighting
    float3 view_vec = uniforms.view_pos - world_pos.xyz;
    if (dot(transformed_normal, view_vec) < 0.0) {
        transformed_normal = -transformed_normal;
    }
    
    out.normal = transformed_normal;
    
    out.tex_coord = in.tex_coord;
    out.color_variation = instance.color_variation;
    out.fade_alpha = instance._padding; // Using padding field for fade alpha
    
    return out;
}

fragment float4 grass_fragment(
    VertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Base grass color with more natural tones
    float3 base_color = float3(0.3, 0.6, 0.2); // Brighter, more natural green
    
    // Apply color variation
    float3 grass_color = base_color + in.color_variation;
    
    // Add gradient from bottom to top (darker at base, lighter at tips)
    float gradient = mix(0.5, 1.0, pow(in.tex_coord.y, 0.5));
    grass_color *= gradient;
    
    // Calculate light vectors
    float3 light_dir = normalize(uniforms.light_pos - in.world_pos);
    float3 view_dir = normalize(uniforms.view_pos - in.world_pos);
    float3 half_dir = normalize(light_dir + view_dir);
    
    // Enhanced subsurface scattering for realistic light transmission
    // Calculate back-face illumination
    float3 light_to_point = normalize(in.world_pos - uniforms.light_pos);
    float back_light = max(0.0, dot(view_dir, -light_to_point));
    float subsurface_wrap = max(0.0, dot(in.normal, light_dir) + 0.5) * 0.7;
    
    // Translucency effect - light passing through the grass blade
    float translucency = pow(back_light, 3.0) * 0.8;
    float thickness = 1.0 - in.tex_coord.y; // Thicker at base, thinner at tips
    translucency *= (1.0 - thickness * 0.5);
    
    // Calculate subsurface color with warmer tones
    float3 subsurface_color = float3(0.4, 0.7, 0.2) * uniforms.light_color;
    float3 subsurface_contribution = subsurface_color * (subsurface_wrap + translucency * 0.6);
    
    // Standard diffuse lighting with wrap-around for softer shadows
    float NdotL = dot(in.normal, light_dir);
    float wrapped_diffuse = max(0.0, (NdotL + 0.3) / 1.3);
    float3 diffuse = uniforms.diffuse_strength * wrapped_diffuse * uniforms.light_color;
    
    // Soft specular for wet grass effect
    float NdotH = max(0.0, dot(in.normal, half_dir));
    float specular = pow(NdotH, 32.0) * 0.2;
    specular *= (1.0 - in.tex_coord.y); // Less specular at tips
    
    // Enhanced ambient with color bleeding from ground
    float3 ambient = uniforms.ambient_strength * uniforms.light_color;
    ambient += float3(0.05, 0.08, 0.02) * (1.0 - in.tex_coord.y); // Ground color influence
    
    // Combine all lighting components
    float3 lighting = ambient + diffuse + subsurface_contribution;
    float3 result = grass_color * lighting + specular * uniforms.light_color;
    
    // Rim lighting for added depth
    float rim = 1.0 - max(0.0, dot(view_dir, in.normal));
    rim = pow(rim, 2.0) * 0.15;
    result += rim * uniforms.light_color * grass_color;
    
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
    
    // Apply LOD fade alpha for smooth transitions
    return float4(result, in.fade_alpha);
}