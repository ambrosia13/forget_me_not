struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = vec4(in.position.xy, 0.0, 1.0);
    out.uv = in.uv;
    out.texcoord = in.texcoord;

    return out;
}