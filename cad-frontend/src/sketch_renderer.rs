use std::{f32::consts::PI, path::PathBuf};

use cad::{
    entity::{GuidedEntity, Point},
    sketch::Sketch,
};
use rust_ui::{
    geometry::Vector,
    render::{Color, line::LineRenderer},
    shader::Shader,
};

use crate::{SHADER_DIR, ui::viewport::ViewportData};

use glm;

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

    pub fn draw_axes(&mut self, state: &ViewportData) {
        let projection = glm::perspective(state.size.x / state.size.y, 45.0, 0.0001, 100.0);
        let model = glm::scaling(&glm::vec3(1.0, 1.0, 1.0));
        // azimuth - phi
        // polar - theta

        // Create camera position using spherical coordinates
        let camera_distance = state.distance;
        let camera_pos = glm::Vec3::new(
            camera_distance * state.azimuthal_angle.sin() * state.polar_angle.cos(),
            camera_distance * state.azimuthal_angle.sin() * state.polar_angle.sin(),
            camera_distance * state.azimuthal_angle.cos(),
        );
        let view = glm::look_at(
            &camera_pos,                    // Camera position
            &glm::Vec3::new(0.0, 0.0, 0.0), // Look at origin
            &glm::Vec3::new(0.0, 0.0, 1.0), // Up vector
        );

        self.line_r.draw_3d(
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(1.0, 0.0, 0.0),
            Color::new(1.0, 0.0, 0.0, 1.0),
            2.0,
            &projection,
            &model,
            &view,
        );
        self.line_r.draw_3d(
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
            Color::new(0.0, 1.0, 0.0, 1.0),
            2.0,
            &projection,
            &model,
            &view,
        );
        self.line_r.draw_3d(
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
            Color::new(0.0, 0.0, 1.0, 1.0),
            2.0,
            &projection,
            &model,
            &view,
        );
        self.line_r.draw_3d(
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(-1.0, 0.0, 0.0),
            Color::new(0.2, 0.0, 0.0, 1.0),
            2.0,
            &projection,
            &model,
            &view,
        );
        self.line_r.draw_3d(
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, -1.0, 0.0),
            Color::new(0.0, 0.2, 0.0, 1.0),
            2.0,
            &projection,
            &model,
            &view,
        );
        self.line_r.draw_3d(
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, -1.0),
            Color::new(0.0, 0.0, 0.2, 1.0),
            2.0,
            &projection,
            &model,
            &view,
        );
    }

    pub fn draw(&mut self, sketch: &Sketch, state: &mut ViewportData) {
        //state.horizontal_angle = PI / 2.0;
        //state.polar_angle = PI / 4.0;
        self.draw_axes(state);
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
                        glm::perspective(state.size.x / state.size.y, 60.0, 0.0001, 100.0);
                    let model = glm::scaling(&glm::vec3(1.0, -1.0, 1.0));

                    // Create camera position using spherical coordinates
                    let camera_distance = 1.0;
                    let camera_pos = glm::Vec3::new(
                        camera_distance * state.azimuthal_angle.sin() * state.polar_angle.cos(),
                        camera_distance * state.azimuthal_angle.cos(),
                        camera_distance * state.azimuthal_angle.sin() * state.polar_angle.sin(),
                    );

                    let view = glm::look_at(
                        &camera_pos,                    // Camera position
                        &glm::Vec3::new(0.0, 0.0, 0.0), // Look at origin
                        &glm::Vec3::new(0.0, 1.0, 0.0), // Up vector
                    );
                    let s_3d = glm::vec3(s.x, s.y, 0.0);
                    let e_3d = glm::vec3(e.x, e.y, 0.0);
                    // self.line_r.draw_3d(
                    //     s_3d,
                    //     e_3d,
                    //     Color::new(1.0, 1.0, 1.0, 1.0),
                    //     2.0,
                    //     &projection,
                    //     &model,
                    //     &view,
                    // );
                }
                _ => {} // TODO: Implement
            }
        }
    }
}
