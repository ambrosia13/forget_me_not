const PI: f32 = 3.1415926535897932384626433832795;
const HALF_PI: f32 = 1.57079632679489661923; 
const TAU: f32 = 6.2831853071795864769252867665590; 

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
    inverse_view_projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    inverse_view_matrix: mat4x4<f32>,
    pos: vec3<f32>,
    view_width: u32,
    view_height: u32,
    frame_count: u32,
    padding_1: u32,
    padding_2: u32,
}

struct PackedSphere {
    data: vec4<f32>,
    color: vec4<f32>,
    emission: vec4<f32>,
}

struct PackedPlane {
    bleh: u32,
    bleh_2: u32,
    bleh_3: u32,
    bleh_4: u32,
}

struct ObjectsUniform {
    spheres: array<PackedSphere, 32>,
    planes: array<PackedPlane, 32>,
    num_spheres: u32,
    num_planes: u32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> objects: ObjectsUniform;

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

// NOISE ---------------------------

fn unpack_sphere(data: PackedSphere) -> Sphere {
    var sphere: Sphere;

    sphere.center = data.data.xyz;
    sphere.color = data.color.xyz;
    sphere.radius = data.data.w;
    sphere.emission = data.emission.xyz;

    return sphere;
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct Material {
    albedo: vec3<f32>,
    emission: vec3<f32>,
}

struct Sphere {
    center: vec3<f32>,
    color: vec3<f32>,
    emission: vec3<f32>,
    radius: f32,
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

    var material: Material;
    material.albedo = sphere.color;
    material.emission = sphere.emission;

    hit.material = material;

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

fn sky(ray: Ray) -> vec3<f32> {
    return mix(vec3(1.0, 1.0, 1.0), vec3(0.05, 0.1, 1.0), smoothstep(-0.4, 0.2, ray.dir.y));
}

fn get_random_sphere() -> Sphere {
    return unpack_sphere(objects.spheres[next_u32() % objects.num_spheres]);
}

fn generate_ray_to_sphere(ray: Ray, sphere: Sphere) -> Ray {
    let point = sphere.center + generate_unit_vector() * sphere.radius;
    let dir = normalize(point - ray.pos);

    return Ray(ray.pos, dir);
}

// fn pdf_sphere(ray: Ray, sphere: Sphere) -> f32 {
//     let dist = distance(ray.pos, sphere.center);
//     return TAU * (1.0 - dist / sqrt(dist * dist + sphere.radius * sphere.radius));
// }

fn raytrace(ray: Ray) -> Hit {
    var closest_hit: Hit;

    for (var i = 0u; i < objects.num_spheres; i++) {
        let sphere = unpack_sphere(objects.spheres[i]);

        let hit = ray_sphere_intersect(ray, sphere);
        closest_hit = merge_hit(closest_hit, hit);
    }

    return closest_hit;
}

fn pathtrace(ray: Ray) -> vec3<f32> {
    var incoming_normal = vec3(10.0);

    var throughput = vec3(1.0);
    var radiance = vec3(0.0);

    var current_ray = ray;

    for (var i = 0; i < 5; i++) {
        var hit: Hit;
        var weight: f32 = 1.0 / TAU;

        // if next_f32() > 1.0 && i != 0 {
        //     let sphere = get_random_sphere();
        //     let d = distance(current_ray.pos, sphere.center);

        //     current_ray = generate_ray_to_sphere(current_ray, sphere);
        //     weight = TAU * (1.0 - sqrt(d * d - sphere.radius * sphere.radius) / d) * dot(incoming_normal, current_ray.dir);
        // }

        // if next_f32() > 0.5 {
        hit = raytrace(current_ray);
        // weight = 1.0;
        // } else {
        //     let sphere = get_random_sphere();
        //     let ray_to_sphere = generate_ray_to_sphere(current_ray, sphere);
        //     hit = raytrace(ray_to_sphere);

        //     if incoming_normal.x == 10.0 {
        //         weight = 0.0;
        //     } else {
        //         weight = max(0.0, dot(incoming_normal, ray_to_sphere.dir));
        //     }
        // }

        if !hit.success {
            // hit sky
            radiance += throughput * sky(current_ray);
            break;
        }

        incoming_normal = hit.normal;
        radiance += throughput * hit.material.emission;
        throughput *= hit.material.albedo / PI;

        current_ray = Ray(hit.position + hit.normal * 0.0001, generate_cosine_vector(hit.normal));
    }

    return radiance;
}

// fn pathtrace(ray: Ray) -> vec3<f32> {
//     let radiance = vec3(0.0);
//     let throughput = vec3(1.0);

//     let bounces = 4;

//     for (var bounce = 0; bounce < bounces; bounce++) {
//         let hit = raytrace(ray);

//         if !hit.success {
//             radiance += throughput * sky(ray);
//             break;
//         }

//         let new_dir = generate_cosine_vector(hit.normal);
//         let pdf = dot(hit.normal, new_dir) / PI;

//         let brdf = vec3(1.0 / PI);
//     }
// }

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    init_rng(in.texcoord);

    let screen_space_pos = vec3(in.texcoord, 1.0);
    let clip_space_pos = screen_space_pos * 2.0 - 1.0;

    let temp = (camera.inverse_view_projection_matrix * vec4(clip_space_pos, 1.0));
    let world_space_pos = temp.xyz / temp.w;
    let view_space_pos = world_space_pos - camera.pos;

    let view_dir = normalize(view_space_pos);

    var ray: Ray;
    ray.pos = camera.pos;
    ray.dir = view_dir;

    // var color = sky(ray);

    var sphere: Sphere;
    sphere.center = vec3(0.0, 0.0, 10.0);
    sphere.radius = 0.5;

    let light_dir = normalize(vec3(0.3, 0.9, -0.5));

    // let hit = raytrace(ray);

    // if hit.success {
    //     color = hit.material.color * max(0.0, dot(hit.normal, light_dir));
    // }

    // let rng = init_rng(in.texcoord);
    // let rng_ptr = &rng;

    var color = vec3(0.0);

    let rays = 5;
    for (var i = 0; i < rays; i++) {
        color += pathtrace(ray) / f32(rays);
    }

    // let color = pathtrace(ray);

    // let slices = floor(in.texcoord.x * 10.0);
    // if in.texcoord.y < 0.1 && f32(objects.num_spheres) >= slices {
    //     color = vec3(0.0, 0.0, 1.0);
    // }

    return vec4(color, 1.0);
    //return vec4(step(0.5, pow(next_f32(), 2.2)));
}