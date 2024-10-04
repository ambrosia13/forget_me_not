# forget_me_not renderer

![glass ball](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/glass_ball.png?raw=true)

![metallic balls](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/metallic_balls.png?raw=true)

![mirror room](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/mirror_room.png?raw=true)

`forget_me_not` is a rendering engine and path tracer written in Rust. It uses the `wgpu` library, which is a native Rust implementation of the WebGPU specification.

The current path tracing implementation is pretty brute-force and simple, though I aim for this project to become a mostly-accurate brute-force path tracer that you can model scenes with.

Currently, there are three types of materials supported: 
- perfect lambertian surfaces (which don't actually exist in real life!), 
- metallic surfaces (surfaces which reflect all light), and 
- dielectric surfaces (nonmetals that may refract or reflect).

And there are three types of geometry supported: 
- planes
- spheres
- axis-aligned bounding boxes (AABBs)

To create an object, you use the command line. First, you update the current material; all geometry created will use the current material:
```
material <lambert|lambertian|metal|dielectric> <albedo_r> <albedo_g> <albedo_b> <emission_r> <emission_g> <emission_b> <roughness> <index_of_refraction>
```

For lambertian materials, roughness and index of refraction (IOR) are ignored; for metals, IOR is ignored; and for dielectrics, roughness is currently ignored but will be implemented in the future.

Once you have set the material you want, you can create a shape:
```
sphere <center_x> <center_y> <center_z> <radius>

OR

plane <normal_x> <normal_y> <normal_z> <point_x> <point_y> <point_z>

OR

aabb <min_x> <min_y> <min_z> <max_x> <max_y> <max_z>
```
where `<point_xyz>` defines a point that the plane passes through.

Some other useful commands to know before creating your scene:
```
// remove the most recently created shape of this type
deleteLast <sphere|plane|aabb>

// print the current camera position
pos

// force the camera to look at a given point
lookAt <x> <y> <z>
```

There are also some non-path-tracing features implemented, like bloom and temporal accumulation!

# notes

This project is licensed under the GNU General Public License v3.0.

Much of the code is adapted from my other big rendering project, [Forget-me-not Shaders](https://github.com/ambrosia13/ForgetMeNot-Shaders), which is a rasterization-based graphics overhaul for the game Minecraft, and it's written in GLSL.