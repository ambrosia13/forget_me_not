use std::{str::FromStr, sync::Arc};

use bevy_ecs::prelude::*;
use crossbeam_queue::SegQueue;
use derived_deref::Deref;
use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::game::material::MaterialType;

use super::{
    camera::Camera,
    input::KeyboardInput,
    material::Material,
    object::{Aabb, Objects, Plane, Sphere},
    render::ReloadRenderContextEvent,
};

pub struct GameCommandArgs<'a> {
    current: usize,
    strs: Vec<&'a str>,
}

impl<'a> GameCommandArgs<'a> {
    pub fn from_input(input: &'a str) -> Self {
        Self {
            current: 0,
            strs: input.split_ascii_whitespace().collect(),
        }
    }

    pub fn cmd_name(&mut self) -> Option<&'a str> {
        self.current += 1;
        self.strs.first().copied()
    }

    pub fn next_str(&mut self) -> Option<&'a str> {
        let next = self.strs.get(self.current).copied();
        self.current += 1;

        next
    }

    fn next<T: Copy + FromStr>(&mut self) -> Option<T> {
        let next = self.strs.get(self.current)?;

        let Ok(next) = next.parse::<T>() else {
            return None;
        };

        self.current += 1;
        Some(next)
    }

    pub fn next_i32(&mut self) -> Option<i32> {
        self.next::<i32>()
    }

    pub fn next_f32(&mut self) -> Option<f32> {
        self.next::<f32>()
    }

    pub fn next_f32_gamma_corrected(&mut self) -> Option<f32> {
        self.next_f32().map(|f| f.powf(2.2))
    }

    pub fn next_bool(&mut self) -> Option<bool> {
        self.next::<bool>()
    }

    pub fn num_args(&self) -> usize {
        self.strs.len() - 1
    }
}

#[derive(Debug)]
pub enum GeometryType {
    Sphere,
    Plane,
    Aabb,
}

#[derive(Debug)]
pub enum GameCommand {
    PrintPos,
    PrintCamera,
    Sphere(Sphere),
    Plane(Plane),
    Aabb(Aabb),
    DeleteLast(GeometryType),
    Clear,
    RandomScene,
    LookAtSphere,
    LookAt(Vec3),
    ReloadShaders,
    NoOp,
}

impl GameCommand {
    pub fn parse(input: &str, material: &mut Option<Material>) -> Option<Self> {
        let mut args = GameCommandArgs::from_input(input);
        let cmd_str = args.cmd_name()?;

        let cmd = match cmd_str {
            "pos" => GameCommand::PrintPos,
            "camera" => GameCommand::PrintCamera,
            "sphere" => {
                let center = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);
                let radius = args.next_f32()?;

                GameCommand::Sphere(Sphere::new(center, radius, material.as_mut().copied()?))
            }
            "plane" => {
                let normal = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);
                let point = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);

                GameCommand::Plane(Plane::new(
                    normal.normalize(),
                    point,
                    material.as_mut().copied()?,
                ))
            }
            "aabb" => {
                let min = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);
                let max = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);

                GameCommand::Aabb(Aabb::new(min, max, material.as_mut().copied()?))
            }
            "deleteLast" => {
                let ty = match args.next_str()? {
                    "sphere" => GeometryType::Sphere,
                    "plane" => GeometryType::Plane,
                    "aabb" => GeometryType::Aabb,
                    _ => return None,
                };

                GameCommand::DeleteLast(ty)
            }
            "clear" => GameCommand::Clear,
            "randomScene" => GameCommand::RandomScene,
            "lookAtSphere" => GameCommand::LookAtSphere,
            "lookAt" => {
                let x = args.next_f32()?;
                let y = args.next_f32()?;
                let z = args.next_f32()?;

                GameCommand::LookAt(Vec3::new(x, y, z))
            }
            "reload" => GameCommand::ReloadShaders,
            "material" => {
                let ty = match args.next_str()? {
                    "lambertian" | "lambert" => MaterialType::Lambertian,
                    "metal" => MaterialType::Metal,
                    "dielectric" => MaterialType::Dielectric,
                    _ => return None,
                };

                let albedo = Vec3::new(
                    args.next_f32_gamma_corrected()?,
                    args.next_f32_gamma_corrected()?,
                    args.next_f32_gamma_corrected()?,
                );
                let emission = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);
                let roughness = args.next_f32()?;
                let ior = args.next_f32()?;

                let new_material = Material {
                    ty,
                    albedo,
                    emission,
                    roughness: roughness.powi(2),
                    ior,
                };

                log::info!("Current material set to {:#?}", new_material);
                *material = Some(new_material);

                GameCommand::NoOp
            }
            _ => return None,
        };

        log::info!("Received command {:?}", &cmd);
        Some(cmd)
    }
}

#[derive(Debug)]
pub struct GameCommands {
    queue: SegQueue<GameCommand>,
}

impl GameCommands {
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
        }
    }

    pub fn push(&self, cmd: GameCommand) {
        self.queue.push(cmd);
    }

    pub fn pop(&self) -> Option<GameCommand> {
        self.queue.pop()
    }
}

impl Default for GameCommands {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Deref)]
pub struct GameCommandsResource(Arc<GameCommands>);

impl GameCommandsResource {
    pub fn init(game_commands: Arc<GameCommands>, world: &mut World) {
        world.insert_resource(GameCommandsResource(game_commands));
    }
}

pub fn send_game_commands_via_keybinds(
    input: Res<KeyboardInput>,
    game_commands: Res<GameCommandsResource>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        game_commands.push(GameCommand::ReloadShaders);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn receive_game_commands(
    game_commands: Res<GameCommandsResource>,
    mut camera: ResMut<Camera>,
    mut objects: ResMut<Objects>,
    mut reload_raytrace_events: EventWriter<ReloadRenderContextEvent>,
) {
    if let Some(command) = game_commands.pop() {
        match command {
            GameCommand::PrintPos => log::info!("{}", camera.position),
            GameCommand::PrintCamera => log::info!("{:#?}", camera),
            GameCommand::Sphere(sphere) => objects.push_sphere(sphere),
            GameCommand::Plane(plane) => objects.push_plane(plane),
            GameCommand::Aabb(aabb) => objects.push_aabb(aabb),
            GameCommand::DeleteLast(ty) => match ty {
                GeometryType::Sphere => {
                    if !objects.spheres.is_empty() {
                        objects.spheres.remove(0);
                    }
                }
                GeometryType::Plane => {
                    if !objects.planes.is_empty() {
                        objects.planes.remove(0);
                    }
                }
                GeometryType::Aabb => {
                    if !objects.aabbs.is_empty() {
                        objects.aabbs.remove(0);
                    }
                }
            },
            GameCommand::Clear => {
                objects.spheres.clear();
                objects.planes.clear();
                objects.aabbs.clear();
            }
            GameCommand::RandomScene => objects.random_scene(),
            GameCommand::LookAtSphere => camera.look_at(objects.spheres[0].center()),
            GameCommand::LookAt(pos) => camera.look_at(pos),
            GameCommand::ReloadShaders => {
                reload_raytrace_events.send(ReloadRenderContextEvent);
            }
            GameCommand::NoOp => {}
        }
    }
}
