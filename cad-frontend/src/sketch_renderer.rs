use cad::{
    SketchInfo,
    entity::{Circle, EntityId, GuidedEntity, Point},
    sketch::Sketch,
};
use rust_ui::{
    geometry::Vector,
    render::{
        COLOR_DANGER, COLOR_SUCCESS, Color, NORD1, circle::CircleRenderer, line::LineRenderer,
        point::PointRenderer,
    },
    shader::{Shader, ShaderName},
};

use crate::{
    app::AppMutableState,
    entity_picker::EntityPicker,
    modes::{AppMode, BindableMessage, ModeStack},
    ui::viewport::ViewportData,
};

pub const PENDING_COLOR: Color = COLOR_SUCCESS;
pub const HOVER_COLOR: Color = COLOR_DANGER;

pub struct SketchRenderer {
    line_r: LineRenderer,
    point_r: PointRenderer,
    circle_r: CircleRenderer,
}

impl SketchRenderer {
    pub fn new() -> Self {
        let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();
        Self {
            line_r: LineRenderer::new(line_shader.clone()),
            point_r: PointRenderer::new(line_shader.clone()),
            circle_r: CircleRenderer::new(line_shader),
        }
    }

    pub fn draw_axes(&mut self, state: &ViewportData) {
        let projection = state.projection();
        let model = state.model();
        let view = state.view();
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
        hovered: Option<EntityId>,
    ) {
        let projection = state.projection();
        let model = state.model();
        let view = state.view();
        for (id, eid) in sketch.guided_entities.iter() {
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

                    let s_3d = s.x * x_axis + s.y * y_axis;
                    let e_3d = e.x * x_axis + e.y * y_axis;
                    self.line_r.draw_3d(
                        s_3d,
                        e_3d,
                        if *id == hovered.unwrap_or_default() {
                            HOVER_COLOR
                        } else {
                            Color::new(1.0, 1.0, 1.0, 1.0)
                        },
                        2.0,
                        &projection,
                        &model,
                        &view,
                    );
                }
                GuidedEntity::Point { id: pid } => {
                    let point: Point = sketch.fundamental_entities[*pid].try_into().unwrap();
                    let p = Vector::new(point.pos.x as f32, point.pos.y as f32);
                    let p_3d = p.x * x_axis + p.y * y_axis;
                    self.point_r.draw_3d(
                        p_3d,
                        if *id == hovered.unwrap_or_default() {
                            HOVER_COLOR
                        } else {
                            Color::new(1.0, 1.0, 1.0, 1.0)
                        },
                        4.0,
                        &projection,
                        &model,
                        &view,
                    );
                }
                GuidedEntity::Circle { id: cid } => {
                    let circle: Circle = sketch.fundamental_entities[*cid].try_into().unwrap();
                    let center = Vector::new(circle.pos.x as f32, circle.pos.y as f32);
                    let center_3d = center.x * x_axis + center.y * y_axis;
                    self.circle_r.draw_3d_oriented(
                        center_3d,
                        circle.radius as f32,
                        if *id == hovered.unwrap_or_default() {
                            HOVER_COLOR
                        } else {
                            Color::new(1.0, 1.0, 1.0, 1.0)
                        },
                        2.0,
                        &projection,
                        &model,
                        &view,
                        x_axis,
                        y_axis,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn draw_pending(
        &mut self,
        sketch_info: &SketchInfo,
        vp_state: &mut ViewportData,
        app_state: &AppMutableState,
        mode_stack: &ModeStack<AppMode, BindableMessage>,
    ) {
        let projection = vp_state.projection();
        let model = vp_state.model();
        let view = vp_state.view();
        let x_axis = sketch_info.plane.x.cast();
        let y_axis = sketch_info.plane.y.cast();

        match mode_stack.outermost() {
            Some(AppMode::Point) => {
                if let Some(p) = app_state.point_mode_data.pending {
                    let p_3d = (p.x as f32) * x_axis + (p.y as f32) * y_axis;
                    self.point_r
                        .draw_3d(p_3d, PENDING_COLOR, 4.0, &projection, &model, &view);
                }
            }
            Some(AppMode::Line) => {
                for w in app_state.line_mode_data.points.windows(2) {
                    let start = w[0];
                    let end = w[1];
                    let start_3d = (start.x as f32) * x_axis + (start.y as f32) * y_axis;
                    let end_3d = (end.x as f32) * x_axis + (end.y as f32) * y_axis;
                    self.line_r.draw_3d(
                        start_3d,
                        end_3d,
                        PENDING_COLOR,
                        2.0,
                        &projection,
                        &model,
                        &view,
                    );
                }

                for p in &app_state.line_mode_data.points {
                    let p_3d = (p.x as f32) * x_axis + (p.y as f32) * y_axis;
                    self.point_r
                        .draw_3d(p_3d, PENDING_COLOR, 4.0, &projection, &model, &view);
                }
            }
            _ => {}
        }
    }
}

pub struct SketchPicker {
    line_r: LineRenderer,
    point_r: PointRenderer,
    circle_r: CircleRenderer,
    pub picker: EntityPicker,
    pub window_width: i32,
    pub window_height: i32,
}

impl SketchPicker {
    pub fn new(window_width: i32, window_height: i32) -> Self {
        let line_shader = Shader::new_from_name(&ShaderName::Pick).unwrap();
        Self {
            line_r: LineRenderer::new(line_shader.clone()),
            point_r: PointRenderer::new(line_shader.clone()),
            circle_r: CircleRenderer::new(line_shader),
            picker: EntityPicker::new(window_width, window_height),
            window_width,
            window_height,
        }
    }

    /// Extremely similar to [SketchRenderer::draw]
    pub fn compute_pick_locations(
        &mut self,
        si: &SketchInfo,
        state: &mut ViewportData,
        x_axis: glm::Vec3,
        y_axis: glm::Vec3,
    ) {
        self.picker.enable_writing();
        // Maybe allow for selection of axes in the future. For example it is useful when
        // constructing planes
        for (EntityId(id), eid) in si.sketch.guided_entities.iter() {
            match eid {
                GuidedEntity::CappedLine {
                    start,
                    end,
                    line: _,
                } => {
                    let start: Point = si.sketch.fundamental_entities[*start].try_into().unwrap();
                    let end: Point = si.sketch.fundamental_entities[*end].try_into().unwrap();
                    let s = Vector::new(start.pos.x as f32, start.pos.y as f32);
                    let e = Vector::new(end.pos.x as f32, end.pos.y as f32);
                    let projection = state.projection();
                    let model = state.model();
                    let view = state.view();

                    let s_3d = s.x * x_axis + s.y * y_axis;
                    let e_3d = e.x * x_axis + e.y * y_axis;
                    self.line_r.shader.use_shader();
                    self.line_r.shader.set_uniform("entityId", &(*id as u32));
                    self.line_r.shader.set_uniform("sketchId", &(si.id as u32));
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
                GuidedEntity::Point { id: pid } => {
                    let point: Point = si.sketch.fundamental_entities[*pid].try_into().unwrap();
                    let p = Vector::new(point.pos.x as f32, point.pos.y as f32);
                    let projection = state.projection();
                    let model = state.model();
                    let view = state.view();
                    let p_3d = p.x * x_axis + p.y * y_axis;
                    self.point_r.shader.use_shader();
                    self.point_r.shader.set_uniform("entityId", &(*id as u32));
                    self.point_r.shader.set_uniform("sketchId", &(si.id as u32));
                    self.point_r.draw_3d(
                        p_3d,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        4.0,
                        &projection,
                        &model,
                        &view,
                    );
                }
                GuidedEntity::Circle { id: cid } => {
                    let circle: Circle = si.sketch.fundamental_entities[*cid].try_into().unwrap();
                    let center = Vector::new(circle.pos.x as f32, circle.pos.y as f32);
                    let projection = state.projection();
                    let model = state.model();
                    let view = state.view();
                    let center_3d = center.x * x_axis + center.y * y_axis;
                    self.circle_r.shader.use_shader();
                    self.circle_r.shader.set_uniform("entityId", &(*id as u32));
                    self.circle_r
                        .shader
                        .set_uniform("sketchId", &(si.id as u32));
                    self.circle_r.draw_3d_oriented(
                        center_3d,
                        circle.radius as f32,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        2.0,
                        &projection,
                        &model,
                        &view,
                        x_axis,
                        y_axis,
                    );
                }
                _ => {}
            }
        }
        self.picker.disable_writing();
    }

    pub fn hovered(&self, mouse_pos: Vector<i32>, viewport_height: f32) -> Option<(EntityId, u16)> {
        let opengl_y = viewport_height as i32 - mouse_pos.y;
        let info = self.picker.read_pixel(mouse_pos.x, opengl_y);
        let entity_id = info.r as u16 | ((info.g as u16) << 8);
        let sketch_id = info.b as u16 | ((info.a as u16) << 8);
        if entity_id == 0 {
            None
        } else {
            Some((EntityId(entity_id), sketch_id))
        }
    }
}
