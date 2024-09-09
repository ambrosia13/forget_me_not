use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy_ecs::prelude::*;
use crossbeam_queue::SegQueue;
use derived_deref::Deref;
use glam::Vec3;

use crate::render_state::{CommandEncoderResource, RenderState};

use super::{
    camera::{Camera, CameraBuffer},
    object::{Objects, ObjectsBuffer, Sphere},
    render::post::{FullscreenQuad, RaytraceRenderContext},
};

pub struct GameCommandArgs<'a> {
    current: usize,
    strs: Vec<&'a str>,
}

impl<'a> GameCommandArgs<'a> {
    pub fn from_input(input: &'a str) -> GameCommandArgs<'a> {
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
        self.current += 1;
        self.strs.get(self.current - 1).copied()
    }

    pub fn next_i32(&mut self) -> Option<i32> {
        let next = self.strs.get(self.current)?;

        let Ok(next) = next.parse::<i32>() else {
            return None;
        };

        self.current += 1;
        Some(next)
    }

    pub fn next_f32(&mut self) -> Option<f32> {
        let next = self.strs.get(self.current)?;

        let Ok(next) = next.parse::<f32>() else {
            return None;
        };

        self.current += 1;
        Some(next)
    }

    pub fn next_bool(&mut self) -> Option<bool> {
        let next = self.strs.get(self.current)?;

        let Ok(next) = next.parse::<bool>() else {
            return None;
        };

        self.current += 1;
        Some(next)
    }

    pub fn num_args(&self) -> usize {
        self.strs.len() - 1
    }
}

#[derive(Debug)]
pub enum ShaderPass {
    Raytrace,
    Final,
}

#[derive(Debug)]
pub enum GameCommand {
    PrintPos,
    PrintCamera,
    Sphere(Sphere),
    LookAtSphere,
    LookAt(Vec3),
    LoadShaders(ShaderPass, PathBuf),
    ReloadShaders(ShaderPass),
    UnloadShaders(ShaderPass),
}

impl GameCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let mut args = GameCommandArgs::from_input(input);
        let cmd_str = args.cmd_name()?;

        let cmd = match cmd_str {
            "pos" => GameCommand::PrintPos,
            "camera" => GameCommand::PrintCamera,
            "sphere" => {
                let center = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);
                let radius = args.next_f32()?;
                let color = Vec3::new(args.next_f32()?, args.next_f32()?, args.next_f32()?);

                GameCommand::Sphere(Sphere::new(center, radius, color))
            }
            "lookAtSphere" => GameCommand::LookAtSphere,
            "lookAt" => {
                let x = args.next_f32()?;
                let y = args.next_f32()?;
                let z = args.next_f32()?;

                GameCommand::LookAt(Vec3::new(x, y, z))
            }
            "loadShaders" => {
                let shader_type = args.next_str()?;

                let pass = match shader_type {
                    "raytrace" | "rt" => ShaderPass::Raytrace,
                    "final" => ShaderPass::Final,
                    _ => {
                        return None;
                    }
                };

                let path = args.next_str()?;
                let path = PathBuf::from(path);

                GameCommand::LoadShaders(pass, path)
            }
            "reloadShaders" => match args.next_str()? {
                "raytrace" | "rt" => GameCommand::ReloadShaders(ShaderPass::Raytrace),
                "final" => GameCommand::ReloadShaders(ShaderPass::Final),
                _ => {
                    return None;
                }
            },
            "unloadShaders" => match args.next_str()? {
                "raytrace" | "rt" => GameCommand::UnloadShaders(ShaderPass::Raytrace),
                "final" => GameCommand::UnloadShaders(ShaderPass::Final),
                _ => {
                    return None;
                }
            },
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

#[derive(Resource, Deref)]
pub struct GameCommandsResource(Arc<GameCommands>);

impl GameCommandsResource {
    pub fn init(game_commands: Arc<GameCommands>, world: &mut World) {
        world.insert_resource(GameCommandsResource(game_commands));
    }
}

pub fn receive_game_commands(
    game_commands: Res<GameCommandsResource>,
    mut camera: ResMut<Camera>,
    mut objects: ResMut<Objects>,
    render_state: Res<RenderState>,
    mut raytrace_render_context: ResMut<RaytraceRenderContext>,
    fullscreen_quad: Res<FullscreenQuad>,
    camera_buffer: Res<CameraBuffer>,
    objects_buffer: Res<ObjectsBuffer>,
) {
    if let Some(command) = game_commands.pop() {
        match command {
            GameCommand::PrintPos => log::info!("{}", camera.position),
            GameCommand::PrintCamera => log::info!("{:#?}", camera),
            GameCommand::Sphere(sphere) => objects.push_sphere(sphere),
            GameCommand::LookAtSphere => camera.look_at(objects.spheres[0].center()),
            GameCommand::LookAt(pos) => camera.look_at(pos),
            GameCommand::LoadShaders(pass, path) => {
                raytrace_render_context.set_shader_path(Some(path));

                raytrace_render_context.recreate(
                    &render_state,
                    &fullscreen_quad,
                    &camera_buffer,
                    &objects_buffer,
                );
            }
            GameCommand::ReloadShaders(pass) => raytrace_render_context.recreate(
                &render_state,
                &fullscreen_quad,
                &camera_buffer,
                &objects_buffer,
            ),
            GameCommand::UnloadShaders(pass) => {
                raytrace_render_context.set_shader_path(None);

                raytrace_render_context.recreate(
                    &render_state,
                    &fullscreen_quad,
                    &camera_buffer,
                    &objects_buffer,
                );
            }
        }
    }
}
