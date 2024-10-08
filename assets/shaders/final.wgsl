#include assets/shaders/lib/header.wgsl

@group(0) @binding(0)
var color_texture: texture_2d<f32>;
@group(0) @binding(1)
var color_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

const FRX_ACES_INPUT_MATRIX: mat3x3<f32> = mat3x3(
    vec3(0.59719, 0.07600, 0.02840),
    vec3(0.35458, 0.90834, 0.13383),
    vec3(0.04823, 0.01566, 0.83777)
);

// ODT_SAT => XYZ => D60_2_D65 => sRGB
const FRX_ACES_OUTPUT_MATRIX: mat3x3<f32> = mat3x3(
    vec3(1.60475, -0.10208, -0.00327),
    vec3(-0.53108, 1.10813, -0.07276),
    vec3(-0.07367, -0.00605, 1.07602)
);

fn FRX_RRT_AND_ODTF_FIT(v: vec3<f32>) -> vec3<f32> {
	let a = v * (v + 0.0245786f) - 0.000090537f;
	let b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
	
    return a / b;
}

fn frx_tone_map(col: vec3<f32>) -> vec3<f32> {
    var color = col;

	color = FRX_ACES_INPUT_MATRIX * color;
	color = FRX_RRT_AND_ODTF_FIT(color);
	return FRX_ACES_OUTPUT_MATRIX * color;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(color_texture, color_sampler, in.uv);

    let screen_space_pos = vec3(in.texcoord, 1.0);
    let clip_space_pos = screen_space_pos * 2.0 - 1.0;

    let temp = (camera.inverse_view_projection_matrix * vec4(clip_space_pos, 1.0));
    let world_space_pos = temp.xyz / temp.w;
    let view_space_pos = world_space_pos - camera.pos;

    let view_dir = normalize(view_space_pos);

    // if depth == 1.0 {
    //     let mix_factor = smoothstep(0.0, 0.2, clamp(view_dir.y, 0.0, 1.0));
    //     color = vec4(mix(vec3(1.0, 1.0, 1.0), vec3(0.25, 0.5, 1.0), mix_factor), 1.0);
    // }

    color = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);
    color = vec4(frx_tone_map(color.rgb), color.a);

    return color;
}