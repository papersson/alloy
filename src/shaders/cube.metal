#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float4x4 mvp_matrix;
};

struct VertexIn {
    float3 position [[attribute(0)]];
    float2 tex_coord [[attribute(1)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 tex_coord;
};

vertex VertexOut cube_vertex(
    VertexIn in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    VertexOut out;
    out.position = uniforms.mvp_matrix * float4(in.position, 1.0);
    out.tex_coord = in.tex_coord;
    return out;
}

fragment float4 cube_fragment(
    VertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]],
    sampler tex_sampler [[sampler(0)]]
) {
    float4 color = tex.sample(tex_sampler, in.tex_coord);
    return color;
}