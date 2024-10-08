use bevy_ecs::prelude::*;
use render_state_ext::RenderStateExt;

use std::{collections::HashMap, error::Error, fmt::Debug, path::Path};

use shader::WgslShader;
use texture::WgpuTexture;

use crate::{render_state::RenderState, util};

pub mod render_state_ext;
pub mod shader;
pub mod texture;

#[derive(Resource, Default)]
pub struct WgpuResourceRegistry {
    shaders: HashMap<String, WgslShader>,
    textures: HashMap<String, WgpuTexture>,
}

impl WgpuResourceRegistry {
    pub fn insert_shader(&mut self, shader: WgslShader) {
        self.shaders.insert(shader.source.name().to_owned(), shader);
    }

    pub fn get_shader(&self, name: &str) -> Option<&WgslShader> {
        self.shaders.get(name)
    }

    pub fn insert_texture(&mut self, texture: WgpuTexture) {
        self.textures.insert(texture.name().to_owned(), texture);
    }

    pub fn get_texture(&self, name: &str) -> Option<&WgpuTexture> {
        self.textures.get(name)
    }

    pub fn get_or_create_texture<P: AsRef<Path> + Debug>(
        &mut self,
        render_state: &RenderState,
        relative_path: P,
    ) -> Result<&WgpuTexture, Box<dyn Error>> {
        let name = util::name_from_path(&relative_path);

        if !self.textures.contains_key(&name) {
            let texture = if relative_path.as_ref().is_dir() {
                // we assume a directory is a cubemap folder
                render_state.load_cubemap_texture(relative_path)?
            } else {
                todo!()
            };

            self.insert_texture(texture);
        }

        Ok(self.get_texture(&name).unwrap())
    }

    pub fn init(world: &mut World) {
        world.insert_resource(Self::default());
    }
}
