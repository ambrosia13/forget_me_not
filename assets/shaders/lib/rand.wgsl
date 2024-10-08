var<private> rng_state: u32;

fn init_rng(texcoord: vec2<f32>, view_width: u32, view_height: u32, frame_count: u32) {
    let frag_coord: vec2<f32> = vec2(texcoord.x * f32(view_width), texcoord.y * f32(view_height));

    let rng_ptr = &rng_state;
    *rng_ptr = u32(view_width * view_height) * (frame_count + 1) * u32(frag_coord.x + frag_coord.y * f32(view_width));
}

fn pcg(seed: ptr<private, u32>) {
    let state: u32 = *seed * 747796405u + 2891336453u;
    let word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    *seed = (word >> 22u) ^ word;
}

fn next_u32() -> u32 {
    pcg(&rng_state);
    return rng_state;
}

fn next_f32() -> f32 {
    return f32(next_u32()) / f32(0xFFFFFFFFu);
}

fn generate_unit_vector() -> vec3<f32> {
    var xy = vec2(next_f32(), next_f32());
    xy.x *= TAU;
    xy.y = 2.0 * xy.y - 1.0;

    return (vec3(vec2(sin(xy.x), cos(xy.x)) * sqrt(1.0 - xy.y * xy.y), xy.y));
}

fn generate_cosine_vector(normal: vec3<f32>) -> vec3<f32> {
    return normalize(normal + generate_unit_vector());
}

fn generate_cosine_vector_from_roughness(normal: vec3<f32>, roughness: f32) -> vec3<f32> {
    return normalize(normal + generate_unit_vector() * roughness);
}