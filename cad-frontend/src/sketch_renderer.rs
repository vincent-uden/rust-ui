use std::path::PathBuf;

use rust_ui::{render::line::LineRenderer, shader::Shader};

use crate::SHADER_DIR;

pub struct SketchRenderer {
    line_r: LineRenderer,
}

impl SketchRenderer {
    pub fn new() -> Self {
        let line_shader = Shader::from_paths(
            &PathBuf::from(format!("{}/line.vs", SHADER_DIR)),
            &PathBuf::from(format!("{}/line.frag", SHADER_DIR)),
            None,
        )
        .unwrap();

        Self {
            line_r: LineRenderer::new(line_shader),
        }
    }
}
