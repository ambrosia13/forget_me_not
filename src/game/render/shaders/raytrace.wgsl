struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
    inverse_view_projection_matrix: mat4x4<f32>,
    pos: vec3<f32>
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct PackedSphere {
    data: vec4<f32>,
    color: vec4<f32>,
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

struct Sphere {
    center: vec3<f32>,
    color: vec3<f32>,
    radius: f32,
}

fn sphere_from_data(data: PackedSphere) -> Sphere {
    var sphere: Sphere;

    sphere.center = data.data.xyz;
    sphere.color = data.color.xyz;
    sphere.radius = data.data.w;

    return sphere;
}

struct Hit {
    success: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    distance: f32,
}

fn ray_sphere_intersect(ray: Ray, sphere: Sphere) -> Hit {
    var hit: Hit;
    hit.success = false;

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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_space_pos = vec3(in.texcoord, 1.0);
    let clip_space_pos = screen_space_pos * 2.0 - 1.0;

    let temp = (camera.inverse_view_projection_matrix * vec4(clip_space_pos, 1.0));
    let world_space_pos = temp.xyz / temp.w;
    let view_space_pos = world_space_pos - camera.pos;

    let view_dir = normalize(view_space_pos);

    var ray: Ray;
    ray.pos = camera.pos;
    ray.dir = view_dir;

    var color = ray.dir * 0.5;
    var scene_dist = 1e30;

    let light_dir = normalize(vec3(0.3, 0.9, -0.5));

    // var sphere: Sphere;
    // sphere.center = vec3(0.0, 0.0, 10.0);
    // sphere.radius = 0.5;

    // let hit = ray_sphere_intersect(ray, sphere);

    // if hit.success {
    //     color = 0.5 + vec3(1.0) * max(0.0, dot(hit.normal, light_dir));
    // }

    for (var i = 0u; i < objects.num_spheres; i++) {
        let sphere = sphere_from_data(objects.spheres[i]);

        if sphere.radius == 0.0 {
            color = vec3(1.0, 0.0, 0.0);
        }

        let hit = ray_sphere_intersect(ray, sphere);

        if hit.success && hit.distance < scene_dist {
            color = sphere.color * (max(0.0, dot(hit.normal, light_dir)) + 0.1);
            scene_dist = hit.distance;
        }
    }

    return vec4(color, 1.0);
}