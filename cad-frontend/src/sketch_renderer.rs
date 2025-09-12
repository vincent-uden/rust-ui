use std::path::PathBuf;

use cad::{
    entity::{GuidedEntity, Point},
    sketch::Sketch,
};
use rust_ui::{
    geometry::Vector,
    render::{Color, line::LineRenderer},
    shader::{Shader, ShaderName},
};

use crate::ui::viewport::ViewportData;

pub struct SketchRenderer {
    line_r: LineRenderer,
}

impl SketchRenderer {
    pub fn new() -> Self {
        let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();
        Self {
            line_r: LineRenderer::new(line_shader),
        }
    }

    fn projection(&self, state: &ViewportData) -> glm::Mat4 {
        glm::perspective(state.size.x / state.size.y, 45.0, 0.0001, 100.0)
    }

    fn model(&self, _state: &ViewportData) -> glm::Mat4 {
        glm::scaling(&glm::vec3(1.0, 1.0, 1.0))
    }

    fn view(&self, state: &ViewportData) -> glm::Mat4 {
        // Create camera position using spherical coordinates
        let camera_distance = state.distance;
        let camera_pos = glm::Vec3::new(
            camera_distance * state.azimuthal_angle.sin() * state.polar_angle.cos(),
            camera_distance * state.azimuthal_angle.sin() * state.polar_angle.sin(),
            camera_distance * state.azimuthal_angle.cos(),
        );
        glm::look_at(
            &camera_pos,                    // Camera position
            &glm::Vec3::new(0.0, 0.0, 0.0), // Look at origin
            &glm::Vec3::new(0.0, 0.0, 1.0), // Up vector
        )
    }

    pub fn draw_axes(&mut self, state: &ViewportData) {
        let projection = self.projection(state);
        let model = self.model(state);
        let view = self.view(state);
        let axes = &[
            (glm::vec3(1.0, 0.0, 0.0), Color::new(1.0, 0.0, 0.0, 1.0)),
            (glm::vec3(0.0, 1.0, 0.0), Color::new(0.0, 1.0, 0.0, 1.0)),
            (glm::vec3(0.0, 0.0, 1.0), Color::new(0.0, 0.0, 1.0, 1.0)),
            (glm::vec3(-1.0, 0.0, 0.0), Color::new(0.2, 0.0, 0.0, 1.0)),
            (glm::vec3(0.0, -1.0, 0.0), Color::new(0.0, 0.2, 0.0, 1.0)),
            (glm::vec3(0.0, 0.0, -1.0), Color::new(0.0, 0.0, 0.2, 1.0)),
        ];
        for (ax, color) in axes {
            self.line_r.draw_3d(
                glm::vec3(0.0, 0.0, 0.0),
                *ax,
                *color,
                2.0,
                &projection,
                &model,
                &view,
            );
        }
    }

    /// `x_axis` and `y_axis` define the plane the sketch lies in and its local coordinate system.
    /// They must both be normalized. Otherwise entities in the sketch would not be the same size
    /// as entities elsewhere.
    pub fn draw(
        &mut self,
        sketch: &Sketch,
        state: &mut ViewportData,
        x_axis: glm::Vec3,
        y_axis: glm::Vec3,
    ) {
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
                    let projection = self.projection(state);
                    let model = self.model(state);
                    let view = self.view(state);

                    let s_3d = s.x * x_axis + s.y * y_axis;
                    let e_3d = e.x * x_axis + e.y * y_axis;
                    self.line_r.draw_3d(
                        s_3d,
                        e_3d,
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
