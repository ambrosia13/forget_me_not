#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/space.wgsl
#include assets/shaders/lib/rand.wgsl
#include assets/shaders/lib/rt/intersect.wgsl
#include assets/shaders/lib/rt/stack.wgsl

const PI: f32 = 3.1415926535897932384626433832795;
const HALF_PI: f32 = 1.57079632679489661923; 
const TAU: f32 = 6.2831853071795864769252867665590; 

const MATERIAL_LAMBERTIAN: u32 = 0u;
const MATERIAL_METAL: u32 = 1u;
const MATERIAL_DIELECTRIC: u32 = 2u;

const IOR_AIR: f32 = 1.000293;

struct ObjectsUniform {
    num_spheres: u32,
    num_planes: u32,
    num_aabbs: u32,
    spheres: array<Sphere, 32>,
    planes: array<Plane, 32>,
    aabbs: array<Aabb, 32>,
}

@group(0) @binding(0)
var<uniform> screen: ScreenUniforms;

@group(1) @binding(0)
var<uniform> objects: ObjectsUniform;

@group(2) @binding(0)
var previous_color_texture: texture_2d<f32>;

@group(2) @binding(1)
var previous_color_sampler: sampler;

@group(2) @binding(2)
var cubemap_texture: texture_cube<f32>;

@group(2) @binding(3)
var cubemap_sampler: sampler;

fn sky(ray: Ray) -> vec3<f32> {
    return pow(textureSample(cubemap_texture, cubemap_sampler, ray.dir).rgb, vec3(2.2));
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

    for (var i = 0u; i < objects.num_aabbs; i++) {
        let aabb = objects.aabbs[i];

        let hit = ray_aabb_intersect(ray, aabb);
        closest_hit = merge_hit(closest_hit, hit);
    }

    return closest_hit;
}

// Schlick approximation for reflectance
fn reflectance(cos_theta: f32, ior: f32) -> f32 {
    var r0 = (1.0 - ior) / (1.0 + ior);
    r0 *= r0;

    return r0 + (1.0 - r0) * pow(1.0 - cos_theta, 5.0);
}

struct MaterialHitResult {
    brdf: vec3<f32>,
    next_ray: Ray,
}

fn material_hit_result(hit: Hit, ray: Ray, stack: ptr<function, Stack>) -> MaterialHitResult {
    if hit.material.ty == MATERIAL_LAMBERTIAN {
        let brdf = hit.material.albedo / PI;
        let next_ray = Ray(hit.position + hit.normal * 0.0001, generate_cosine_vector(hit.normal));

        return MaterialHitResult(brdf, next_ray);
    } else if hit.material.ty == MATERIAL_METAL {
        let brdf = hit.material.albedo;
        
        let reflect_dir = reflect(ray.dir, hit.normal);
        let next_ray = Ray(
            hit.position + hit.normal * 0.0001, 
            mix(reflect_dir, generate_cosine_vector(hit.normal), hit.material.roughness)
        );

        return MaterialHitResult(brdf, next_ray);
    } else if hit.material.ty == MATERIAL_DIELECTRIC {
        let cos_theta = dot(-ray.dir, hit.normal);
        let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

        let previous_ior = top_of_stack_or(stack, IOR_AIR);
        let current_ior = hit.material.ior;

        var ior: f32;

        if hit.front_face {
            ior = previous_ior / current_ior;
        } else {
            ior = current_ior / previous_ior;
        }

        let cannot_refract = ior * sin_theta > 1.0;

        var brdf = vec3(0.0);
        var pos = hit.position;
        var dir = vec3(0.0);

        if cannot_refract || reflectance(cos_theta, ior) > next_f32() {
            brdf = vec3(1.0);
            
            dir = reflect(ray.dir, hit.normal);
            dir = mix(dir, generate_cosine_vector(hit.normal), hit.material.roughness);

            pos += hit.normal * 0.0001;
        } else {
            if hit.front_face {
                push_to_stack(stack, current_ior);
            } else {
                pop_from_stack(stack);
            }

            // dir = generate_cosine_vector(hit.normal);
            // pos += hit.normal * 0.0001;

            brdf = hit.material.albedo;

            dir = refract(ray.dir, hit.normal, ior);
            dir = normalize(dir + generate_unit_vector() * hit.material.roughness);

            pos -= hit.normal * 0.0001;
        }


        return MaterialHitResult(brdf, Ray(pos, dir));
    } else {
        return MaterialHitResult(vec3(0.0), Ray(vec3(0.0), vec3(0.0)));
    }
}

fn pathtrace(ray: Ray) -> vec3<f32> {
    var incoming_normal = vec3(10.0);
    var ior_stack = new_stack();

    var throughput = vec3(1.0);
    var radiance = vec3(0.0);

    var current_ray = ray;

    var bounces = 5;
    
    let should_accumulate = 
        all(screen.camera.position == screen.camera.previous_position) &&
        all(screen.camera.view == screen.camera.previous_view);

    if should_accumulate {
        bounces = 50;
    }

    for (var i = 0; i < bounces; i++) {
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

        let material_hit_result = material_hit_result(hit, current_ray, &ior_stack);
        throughput *= material_hit_result.brdf;

        current_ray = material_hit_result.next_ray;
    }

    return radiance;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    init_rng(in.texcoord, screen.view.width, screen.view.height, screen.view.frame_count);

    let screen_space_pos = vec3(in.texcoord, 1.0);
    let world_space_pos = from_screen_space(screen_space_pos, screen.camera.inverse_view_projection_matrix);
    let scene_space_pos = world_space_pos - screen.camera.position;

    let view_dir = normalize(scene_space_pos);

    var ray: Ray;
    ray.pos = screen.camera.position;
    ray.dir = view_dir;

    var color = vec3(0.0);

    let rays = 5;
    for (var i = 0; i < rays; i++) {
        color += pathtrace(ray) / f32(rays);
    }

    let sample = textureSample(previous_color_texture, previous_color_sampler, in.uv);
    let previous_color = sample.rgb;
    var frame_age = sample.a;

    let should_accumulate = 
        all(screen.camera.position == screen.camera.previous_position) &&
        all(screen.camera.view == screen.camera.previous_view);

    if !should_accumulate {
        frame_age = 0.0;
    }

    color = mix(previous_color, color, 1.0 / (frame_age + 1.0));

    return vec4(color, frame_age + 1.0);
}