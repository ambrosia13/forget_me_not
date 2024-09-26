use std::path::Path;

use crate::render_state::RenderState;

use super::shader::{WgslShader, WgslShaderSource};

pub trait RenderStateExt {
    fn load_shader<P: AsRef<Path>>(&self, relative_path: P) -> WgslShader;
}

impl RenderStateExt for RenderState {
    fn load_shader<P: AsRef<Path>>(&self, relative_path: P) -> WgslShader {
        let mut source = WgslShaderSource::load(relative_path);

        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut module = self.device.create_shader_module(source.desc());
        let err = pollster::block_on(self.device.pop_error_scope());

        if err.is_some() {
            source = WgslShaderSource::fallback();
            module = self.device.create_shader_module(source.desc());
        }

        WgslShader { source, module }
    }
}
