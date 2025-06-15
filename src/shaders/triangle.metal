#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    packed_float3 position;
};

struct VertexOut {
    float4 position [[position]];
};

vertex VertexOut triangle_vertex(
    uint vid [[vertex_id]]
) {
    VertexOut out;
    
    // Define triangle vertices directly in shader
    float2 positions[3] = {
        float2( 0.0,  0.5),  // Top
        float2(-0.5, -0.5),  // Bottom left
        float2( 0.5, -0.5)   // Bottom right
    };
    
    out.position = float4(positions[vid], 0.0, 1.0);
    return out;
}

fragment float4 triangle_fragment(VertexOut in [[stage_in]]) {
    return float4(1.0, 0.5, 0.2, 1.0);
}