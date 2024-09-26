use std::{
    borrow::Cow,
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

use regex::Regex;

use crate::render_state::RenderState;

// pub struct WgslShader {
//     pub name: String,
//     pub source: String,
//     pub path: PathBuf,

//     module: wgpu::ShaderModule,
// }

// impl WgslShader {
//     pub fn from_path(render_state: &RenderState, path: &Path) -> Result<Self, std::io::Error> {
//         let name = String::from(path.file_name().unwrap().to_str().unwrap());
//         let source = std::fs::read_to_string(path)?;
//         let path = path.to_owned();

//         let module = render_state
//             .device
//             .create_shader_module(wgpu::ShaderModuleDescriptor {
//                 label: Some(&name),
//                 source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&source)),
//             });

//         Ok(Self {
//             name,
//             source,
//             path,
//             module,
//         })
//     }

// pub fn reload(&mut self) -> Result<(), std::io::Error> {
//     self.name = String::from(self.path.file_name().unwrap().to_str().unwrap());
//     self.source = std::fs::read_to_string(self.path)?;

//     }
// }

pub fn resolve_includes(mut source: String, parent_dir: &Path) -> Result<String, std::io::Error> {
    let mut include_source = String::new();

    let mut index_of_include = 0;
    let mut length_of_include: usize = 0;

    let mut index: usize = 0;
    let mut found_include_directive = false;
    for s in source.split_ascii_whitespace() {
        if found_include_directive {
            let path = parent_dir.join(Path::new(s));
            include_source = std::fs::read_to_string(path)?;

            index_of_include = index.saturating_sub("#include ".len());
            length_of_include = "#include ".len() + s.len();

            break;
        }

        found_include_directive = s == "#include";
        index += s.len();
    }

    let (before_include, include) = source.split_at_mut(index_of_include);
    let (_, after_include) = include.split_at_mut(length_of_include);

    let new_source = if found_include_directive {
        format!("{}{}{}", before_include, include_source, after_include)
    } else {
        source
    };

    Ok(new_source)
}
