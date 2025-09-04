use std::path::PathBuf;

use cad::{
    entity::{FundamentalEntity, GuidedEntity, Point},
    sketch::Sketch,
};
use rust_ui::{
    geometry::Vector,
    render::{COLOR_LIGHT, Color, line::LineRenderer},
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
                    let mut view = glm::Mat4::identity();
                    view =
                        glm::Mat4::new_rotation(glm::Vec3::new(state.horizontal_angle, 0.0, 0.0))
                            * view;
                    view =
                        glm::Mat4::new_rotation(glm::Vec3::new(0.0, 0.0, state.polar_angle)) * view;
                    self.line_r.draw_3d(
                        s,
                        e,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        2.0,
                        &projection,
                        &model,
                        &view,
                    );
                    debug!("Drawing line {:?} {:?}", s, e);
                }
                _ => {} // TODO: Implement
            }
        }
    }
}
