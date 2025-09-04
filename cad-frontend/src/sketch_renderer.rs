use std::path::PathBuf;

use cad::{
    entity::{GuidedEntity, Point},
    sketch::Sketch,
};
use rust_ui::{
    geometry::Vector,
    render::{Color, line::LineRenderer},
    shader::Shader,
};
use tracing::debug;

use crate::{SHADER_DIR, ui::viewport::ViewportData};

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

    pub fn draw(&mut self, sketch: &Sketch, state: &mut ViewportData) {
        state.horizontal_angle += 0.01;
        for eid in sketch.guided_entities.values() {
            match eid {
                GuidedEntity::CappedLine {
                    start,
                    end,
                    line: _,
                } => {
                    let start: Point = sketch.fundamental_entities[*start].try_into().unwrap();
                    let end: Point = sketch.fundamental_entities[*end].try_into().unwrap();
                    let s = Vector::new(start.pos.x as f32, start.pos.y as f32);
                    let e = Vector::new(end.pos.x as f32, end.pos.y as f32);
                    let projection =
                        glm::perspective(state.size.x / state.size.y, 60.0, 0.0001, 1000.0);
                    let model = glm::Mat4::identity();

                    // Create camera position using spherical coordinates
                    let camera_distance = 5.0;
                    let camera_pos = glm::Vec3::new(
                        camera_distance * state.horizontal_angle.sin() * state.polar_angle.cos(),
                        camera_distance * state.horizontal_angle.cos(),
                        camera_distance * state.horizontal_angle.sin() * state.polar_angle.sin(),
                    );

                    let view = glm::look_at(
                        &camera_pos,                    // Camera position
                        &glm::Vec3::new(0.0, 0.0, 0.0), // Look at origin
                        &glm::Vec3::new(0.0, 1.0, 0.0), // Up vector
                    );
                    self.line_r.draw_3d(
                        s,
                        e,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        2.0,
                        &projection,
                        &model,
                        &view,
                    );
                }
                _ => {} // TODO: Implement
            }
        }
    }
}
