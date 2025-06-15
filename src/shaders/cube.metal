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

fragment float4 cube_fragment(VertexOut in [[stage_in]]) {
    // For now, just use texture coordinates as color
    return float4(in.tex_coord.x, in.tex_coord.y, 0.5, 1.0);
}