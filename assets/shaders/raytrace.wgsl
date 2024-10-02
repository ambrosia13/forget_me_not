#include assets/shaders/header.wgsl

const PI: f32 = 3.1415926535897932384626433832795;
const HALF_PI: f32 = 1.57079632679489661923; 
const TAU: f32 = 6.2831853071795864769252867665590; 

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

const MATERIAL_LAMBERTIAN: u32 = 0u;
const MATERIAL_METAL: u32 = 1u;

struct Material {
    ty: u32,
    albedo: vec3<f32>,
    emission: vec3<f32>,
    roughness: f32,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
}

struct Plane {
    normal: vec3<f32>,
    point: vec3<f32>,
    material: Material,
}

struct ObjectsUniform {
    num_spheres: u32,
    num_planes: u32,
    spheres: array<Sphere, 32>,
    planes: array<Plane, 32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> objects: ObjectsUniform;

@group(2) @binding(0)
var previous_color_texture: texture_2d<f32>;

@group(2) @binding(1)
var previous_color_sampler: sampler;

// NOISE ---------------------------

var<private> rng_state: u32;

fn init_rng(texcoord: vec2<f32>) {
    let frag_coord: vec2<f32> = vec2(texcoord.x * f32(camera.view_width), texcoord.y * f32(camera.view_height));

    let rng_ptr = &rng_state;
    *rng_ptr = u32(camera.view_width * camera.view_height) * (camera.frame_count + 1) * u32(frag_coord.x + frag_coord.y * f32(camera.view_width));
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

// END NOISE ---------------------------

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct Hit {
    success: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    distance: f32,
    material: Material,
}

fn merge_hit(a: Hit, b: Hit) -> Hit {
    var hit: Hit;

    if !(a.success || b.success) {
        hit.success = false;
        return hit;
    } else if a.success && !b.success {
        return a;
    } else if b.success && !a.success {
        return b;
    } else {
        if a.distance < b.distance {
            hit = a;
        } else {
            hit = b;
        }
    }

    return hit;
}

fn ray_sphere_intersect(ray: Ray, sphere: Sphere) -> Hit {
    var hit: Hit;
    hit.success = false;
    hit.material = sphere.material;

    let origin_to_center = ray.pos - sphere.center;

    let b = dot(origin_to_center, ray.dir);
    let a = dot(ray.dir, ray.dir);
    let c = dot(origin_to_center, origin_to_center) - sphere.radius * sphere.radius;

    let determinant = b * b - a * c;

    if determinant >= 0.0 {
        let determinant_sqrt = sqrt(determinant);
        var t = (-b - determinant_sqrt) / a;

        if t < 0.0 {
            t = (-b + determinant_sqrt) / a;
        }

        if t >= 0.0 {
            let point = ray.pos + ray.dir * t;
            let outward_normal = normalize(point - sphere.center);
            let front_face = dot(ray.dir, outward_normal) < 0.0;

            var normal: vec3<f32>;

            if front_face {
                normal = outward_normal;
            } else {
                normal = -outward_normal;
            }

            hit.success = true;
            hit.position = point;
            hit.normal = normal;
            hit.distance = t;
        }
    }

    return hit;
}

fn ray_plane_intersect(ray: Ray, plane: Plane) -> Hit {
    var hit: Hit;
    hit.success = false;
    hit.material = plane.material;

    let denom = dot(plane.normal, ray.dir);

    if abs(denom) < 1e-6 {
        return hit;
    }

    let t = dot(plane.normal, plane.point - ray.pos) / denom;

    if t < 0.0 {
        return hit;
    }

    hit.success = true;
    hit.position = ray.pos + ray.dir * t;
    hit.normal = plane.normal;
    hit.distance = t;

    return hit;
}

fn sky(ray: Ray) -> vec3<f32> {
    return mix(vec3(1.0, 1.0, 1.0), vec3(0.05, 0.1, 1.0), smoothstep(-0.4, 0.2, ray.dir.y));
}

fn get_random_sphere() -> Sphere {
    return objects.spheres[next_u32() % objects.num_spheres];
}

fn generate_ray_to_sphere(ray: Ray, sphere: Sphere) -> Ray {
    let point = sphere.center + generate_unit_vector() * sphere.radius;
    let dir = normalize(point - ray.pos);

    return Ray(ray.pos, dir);
}

fn raytrace(ray: Ray) -> Hit {
    var closest_hit: Hit;

    for (var i = 0u; i < objects.num_spheres; i++) {
        let sphere = objects.spheres[i];

        let hit = ray_sphere_intersect(ray, sphere);
        closest_hit = merge_hit(closest_hit, hit);
    }

    for (var i = 0u; i < objects.num_planes; i++) {
        let plane = objects.planes[i];

        let hit = ray_plane_intersect(ray, plane);
        closest_hit = merge_hit(closest_hit, hit);
    }

    return closest_hit;
}

fn brdf(material: Material, ray: Ray) -> vec3<f32> {
    if material.ty == MATERIAL_LAMBERTIAN {
        return material.albedo / PI;
    } else if material.ty == MATERIAL_METAL {
        return material.albedo;
    } else {
        return vec3(0.0);
    }
}

fn next_ray(hit: Hit, ray: Ray) -> Ray {
    if hit.material.ty == MATERIAL_LAMBERTIAN {
        return Ray(hit.position + hit.normal * 0.0001, generate_cosine_vector(hit.normal));
    } else if hit.material.ty == MATERIAL_METAL {
        let dir = reflect(ray.dir, hit.normal);
        return Ray(
            hit.position + hit.normal * 0.0001, 
            mix(dir, generate_cosine_vector(hit.normal), hit.material.roughness)
        );
    } else {
        return ray;
    }
}

fn pathtrace(ray: Ray) -> vec3<f32> {
    var incoming_normal = vec3(10.0);

    var throughput = vec3(1.0);
    var radiance = vec3(0.0);

    var current_ray = ray;

    for (var i = 0; i < 5; i++) {
        var hit: Hit;
        var weight: f32 = 1.0 / TAU;

        hit = raytrace(current_ray);

        if !hit.success {
            // hit sky
            radiance += throughput * sky(current_ray);
            break;
        }

        incoming_normal = hit.normal;
        radiance += throughput * hit.material.emission;
        throughput *= brdf(hit.material, current_ray);

        current_ray = next_ray(hit, current_ray);
    }

    return radiance;
}

fn from_screen_space(screen_space_pos: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    let clip_space_pos = screen_space_pos * 2.0 - 1.0;
    let temp = matrix * vec4(clip_space_pos, 1.0);
    return temp.xyz / temp.w;
}

fn to_screen_space(pos: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    let temp = matrix * vec4(pos, 1.0);
    return (temp.xyz / temp.w) * 0.5 + 0.5;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    init_rng(in.texcoord);

    let screen_space_pos = vec3(in.texcoord, 1.0);
    let world_space_pos = from_screen_space(screen_space_pos, camera.inverse_view_projection_matrix);
    let scene_space_pos = world_space_pos - camera.pos;

    let view_dir = normalize(scene_space_pos);

    let position_difference = camera.pos - camera.previous_pos;
    let previous_screen_space_pos = to_screen_space(scene_space_pos + position_difference, camera.previous_view_projection_matrix);

    let previous_uv = vec2(
        previous_screen_space_pos.x,
        1.0 - previous_screen_space_pos.y,
    );

    var camera_view = from_screen_space(vec3(0.5, 0.5, 1.0), camera.inverse_view_projection_matrix);
    let previous_camera_view = normalize(to_screen_space(camera_view, camera.previous_view_projection_matrix));

    camera_view = normalize(camera_view);

    // if any(camera_view != previous_camera_view) {
    //     return vec4(1.0, 0.0, 0.0, 1.0);
    // }

    var ray: Ray;
    ray.pos = camera.pos;
    ray.dir = view_dir;

    var color = vec3(0.0);

    let rays = 5;
    for (var i = 0; i < rays; i++) {
        color += pathtrace(ray) / f32(rays);
    }

    let sample = textureSample(previous_color_texture, previous_color_sampler, in.uv);
    let previous_color = sample.rgb;
    var frame_age = sample.a;

    if camera.should_accumulate == 0 {
        frame_age = 0.0;
    }

    color = mix(previous_color, color, 1.0 / (frame_age + 1.0));

    return vec4(color, frame_age + 1.0);
}