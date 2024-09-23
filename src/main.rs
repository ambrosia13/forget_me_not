use env_logger::Env;
use game::object::Sphere;
use glam::Vec3;
use util::buffer::{AsGpuBytes, GpuBytes};

pub mod game;
pub mod render_state;
pub mod util;

fn main() {
    // struct Objects {
    //     first: u32,
    //     second: u32,
    //     third: Sphere,
    // }

    // let sphere = Sphere::new(
    //     Vec3::new(0.1, 0.3, 0.5),
    //     0.2,
    //     Vec3::new(1.0, 0.9, 0.8),
    //     Vec3::splat(0.6),
    // );

    // let objects = Objects {
    //     first: 1000500000,
    //     second: 1412400000,
    //     third: sphere,
    // };

    // let mut buf = GpuBytes::new();

    // buf.write_u32(objects.first);
    // buf.write_u32(objects.second);
    // buf.write_struct(&objects.third);

    // let slice = buf.as_slice();

    // for i in (0..slice.len()).step_by(4) {
    //     let u8s = &slice[i..(i + 4)];
    //     let as_f32: &[f32] = bytemuck::cast_slice(u8s);

    //     print!("{:?} ", as_f32);
    //     if i != 0 && i % 16 == 12 {
    //         println!()
    //     }
    // }

    // println!("\n");

    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .filter_module("forget_me_not", log::LevelFilter::Info)
        .init();

    pollster::block_on(game::run());
}
