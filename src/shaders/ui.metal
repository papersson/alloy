#include <metal_stdlib>
using namespace metal;

struct UIVertex {
    float2 position [[attribute(0)]];
    float2 texcoord [[attribute(1)]];
    float4 color [[attribute(2)]];
};

struct UIVertexOut {
    float4 position [[position]];
    float2 texcoord;
    float4 color;
};

struct UIUniforms {
    float4x4 projection;
};

vertex UIVertexOut ui_vertex(UIVertex in [[stage_in]],
                             constant UIUniforms& uniforms [[buffer(1)]]) {
    UIVertexOut out;
    out.position = uniforms.projection * float4(in.position, 0.0, 1.0);
    out.texcoord = in.texcoord;
    out.color = in.color;
    return out;
}

fragment float4 ui_fragment(UIVertexOut in [[stage_in]],
                           texture2d<float> font_texture [[texture(0)]],
                           sampler font_sampler [[sampler(0)]]) {
    float4 texColor = font_texture.sample(font_sampler, in.texcoord);
    return in.color * texColor.a; // Use alpha from texture, color from vertex
}