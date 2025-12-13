use core::f32;
use std::{cell::RefCell, f64::consts::PI, time::Instant};

use cad::{
    Plane, Scene, SketchInfo,
    entity::GeometricEntity,
    topology::{Edge, Face, TopoEntity, TopoId},
};
use glfw::{Action, Key, Modifiers, Scancode, WindowEvent};
use modes::{Config, ModeStack};
use rust_ui::{
    geometry::Vector,
    perf_overlay::PerformanceOverlay,
    render::{
        Color,
        line::LineRenderer,
        renderer::{AppState, RenderLayout, visual_log},
    },
};
use tracing::{debug, error, info};

use crate::{
    input::{self, glfw_key_to_key_input},
    modes::{AppBindableMessage, AppMode, AppMouseAction, default_config},
    sketch_renderer::{SketchPicker, SketchRenderer},
    ui::{
        area::{AreaData, AreaType},
        area_manager::AreaManager,
        boundary::{BoundaryId, BoundaryOrientation},
        settings::Settings,
        viewport::{self, ViewportData},
    },
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SketchModeData {
    pub sketch_id: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PointModeData {
    /// The mouse pos in sketch space
    pub pending: Option<glm::DVec2>,
}

#[derive(Debug, Clone, Default)]
pub struct LineModeData {
    pub points: Vec<glm::DVec2>,
}

#[derive(Debug, Clone, Default)]
pub struct CircleModeData {
    pub center: Option<glm::DVec2>,
    pub boundary: Option<glm::DVec2>,
}

#[derive(Debug)]
pub(crate) struct AppMutableState {
    pub scene: Scene,
    pub sketch_mode_data: SketchModeData,
    pub point_mode_data: PointModeData,
    pub line_mode_data: LineModeData,
    pub circle_mode_data: CircleModeData,
}

/// Returns the TopoIds of edges belonging to the loop that contains the given point,
/// or None if the point is not inside any loop.
fn find_face_at_point(sketch: &cad::sketch::Sketch, point: glm::DVec2) -> Option<Vec<TopoId>> {
    for l in sketch.loops() {
        if sketch.is_inside(l, point.into()) {
            return Some(l.ids.clone());
        }
    }
    None
}

pub struct App {
    pub perf_overlay: PerformanceOverlay<Self>,
    pub dragging_boundary: Option<BoundaryId>,
    pub mouse_pos: Vector<f32>,
    pub debug_draw: bool, // Eventually turn this into a menu
    pub debug_picker: bool,
    pub original_window_size: Vector<f32>,
    pub area_manager: AreaManager,
    pub settings: Settings,
    pub settings_open: bool,
    pub sketch_renderer: SketchRenderer,
    pub sketch_picker: SketchPicker,
    pub mutable_state: RefCell<AppMutableState>,

    pub config: Config<AppMode, AppBindableMessage, AppMouseAction>,
    pub mode_stack: ModeStack<AppMode, AppBindableMessage>,
}

impl App {
    fn base_layer(&mut self, _window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        let mut out = vec![];
        // Areas can't be calculated using taffy since they're a directed graph, not a tree.
        // Return one RenderLayout per area. They will technically be on different layers, but that
        // doesn' matter as they'll all be scissored.

        for area in self.area_manager.area_map.values_mut() {
            out.push(area.generate_layout(&self.mutable_state.borrow(), &self.mode_stack));
        }
        for area in self.area_manager.area_map.values_mut() {
            out.push(area.area_kind_picker_layout());
        }

        out
    }

    pub fn resize_areas(&mut self, new_window_size: Vector<f32>) {
        self.area_manager
            .resize_areas(self.original_window_size, new_window_size);
        self.original_window_size = new_window_size;
    }

    pub fn debug_draw(&mut self, line_renderer: &LineRenderer, window_size: Vector<f32>) {
        for bdry in self.area_manager.bdry_map.values() {
            for aid1 in &bdry.side1 {
                let a1 = &self.area_manager.area_map[*aid1];
                for aid2 in &bdry.side2 {
                    let a2 = &self.area_manager.area_map[*aid2];
                    line_renderer.draw(
                        a1.bbox.center(),
                        a2.bbox.center(),
                        Color::new(1.0, 0.0, 0.0, 1.0),
                        2.0,
                        window_size,
                    );
                }
            }
        }
    }

    pub fn save_layout(&self) {
        let json =
            serde_json::to_string_pretty(&self.area_manager).expect("Failed to serialize layout");
        std::fs::write("layout.json", json).expect("Failed to write layout file");
        info!("Layout saved to layout.json");
    }

    pub fn load_layout(&mut self) {
        match std::fs::read_to_string("layout.json") {
            Ok(json) => match serde_json::from_str::<AreaManager>(&json) {
                Ok(loaded_manager) => {
                    self.area_manager = loaded_manager;
                    info!("Layout loaded from layout.json");
                }
                Err(e) => {
                    error!("Failed to deserialize layout: {}", e);
                }
            },
            Err(e) => {
                error!("Failed to read layout file: {}", e);
            }
        }
    }

    pub fn update_areas(&mut self) {
        let _span = tracy_client::span!("Update areas");
        for area in self.area_manager.area_map.values_mut() {
            area.update();
        }
    }

    /// Some areas contain stuff that isn't part of the regular UI tree such as the viewport that
    /// renders 3D scenes. Those are rendered here, before the UI pass.
    pub fn draw_special_areas(&mut self) {
        let _span = tracy_client::span!("Special areas");
        // Render pass
        for area in self.area_manager.area_map.values_mut() {
            match area.area_type {
                AreaType::Viewport => {
                    let data: &mut ViewportData = (&mut area.area_data).try_into().unwrap();
                    data.size = area.bbox.size();

                    self.sketch_picker.picker.enable_writing();
                    unsafe {
                        gl::BlendFunc(gl::ONE, gl::ZERO);
                        gl::Viewport(0, 0, area.bbox.width() as i32, area.bbox.height() as i32);
                        gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
                        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    }
                    for si in &self.mutable_state.borrow().scene.sketches {
                        if si.visible {
                            self.sketch_picker.compute_pick_locations(
                                si,
                                data,
                                si.plane.x.cast(),
                                si.plane.y.cast(),
                            );
                        }
                    }
                    self.sketch_picker.picker.disable_writing();
                    unsafe {
                        let opengl_y =
                            self.original_window_size.y - area.bbox.x0.y - area.bbox.height();
                        gl::Viewport(
                            area.bbox.x0.x as i32,
                            opengl_y as i32,
                            area.bbox.width() as i32,
                            area.bbox.height() as i32,
                        );
                        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    }
                    self.sketch_renderer.draw_axes(data);
                    for si in &self.mutable_state.borrow().scene.sketches {
                        if si.visible {
                            // Compute mouse position in sketch coordinates for face detection
                            let mouse_in_area = self.mouse_pos - area.bbox.x0;
                            let face_edges = data
                                .screen_to_sketch_coords(mouse_in_area, &si.plane)
                                .and_then(|sketch_pos| find_face_at_point(&si.sketch, sketch_pos));

                            self.sketch_renderer.draw(
                                &si.sketch,
                                data,
                                si.plane.x.cast(),
                                si.plane.y.cast(),
                                face_edges.as_deref(),
                            );
                        }
                        let active_sketch =
                            { self.mutable_state.borrow().sketch_mode_data.sketch_id };
                        if si.id == active_sketch {
                            self.sketch_renderer.draw_pending(
                                &si,
                                data,
                                &self.mutable_state.borrow(),
                                &self.mode_stack,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        unsafe {
            gl::Viewport(
                0,
                0,
                self.original_window_size.x as i32,
                self.original_window_size.y as i32,
            );
        }
    }

    pub fn edit_sketch(&mut self, id: u16) {
        let mut state = self.mutable_state.borrow_mut();
        // Move camera
        if let Some(sketch) = state.scene.sketches.iter().find(|s| s.id == id) {
            let normal = sketch.plane.x.cross(&sketch.plane.y);
            let polar = normal.y.atan2(normal.x);
            let horizontal_hypotenuse = (normal.x.powi(2) + normal.y.powi(2)).sqrt();
            let azimuthal = normal.z.atan2(horizontal_hypotenuse) + PI / 2.0;
            // What happens if you have two viewports open?
            // Should both rotate to face the sketch? Probably not?
            // How do we indicate a primary or secondary viewport?
            // Alternatively how is the viewport selected to rotate?
            // Blender does NOT have anything similar to this
            //
            // For now:
            // - Rotate all viewports
            //
            // In the future:
            // - If there are multiple viewports, let the user click the one to rotate
            for area in self.area_manager.area_map.values_mut() {
                match &mut area.area_data {
                    AreaData::Viewport(data) => {
                        data.target_polar_angle = polar as f32;
                        data.target_azimuthal_angle = azimuthal as f32;
                        data.start_polar_angle = data.polar_angle;
                        data.start_azimuthal_angle = data.azimuthal_angle;
                        data.auto_move_start = Instant::now();
                        data.interaction_state = viewport::InteractionState::AutoMoving;
                    }
                    _ => {}
                }
            }
            state.sketch_mode_data.sketch_id = sketch.id;
            self.mode_stack.pop_until(&AppMode::Base);
            self.mode_stack.push(AppMode::Sketch);
        }
    }

    pub fn handle_area_events(&mut self, window_events: &[WindowEvent], mouse_hit_layer: i32) {
        for e in window_events {
            match *e {
                WindowEvent::MouseButton(button, action, modifiers) => {
                    match *self.mode_stack.outermost().unwrap() {
                        AppMode::Base => match action {
                            Action::Release => {
                                self.dragging_boundary = None;
                            }
                            Action::Press => match button {
                                glfw::MouseButton::Button1 => {
                                    if mouse_hit_layer
                                        < (self.area_manager.area_map.len() as i32) * 2
                                    {
                                        if let Some(bid) =
                                            self.area_manager.find_boundary(self.mouse_pos)
                                        {
                                            self.dragging_boundary = Some(bid);
                                        }
                                    }
                                }
                                _ => {}
                            },
                            Action::Repeat => {}
                        },
                        AppMode::Sketch => {}
                        AppMode::Point | AppMode::Line | AppMode::Circle => {}
                    }
                    if self.dragging_boundary.is_none() {
                        let mut state = self.mutable_state.borrow_mut();
                        for (i, area) in self.area_manager.area_map.values_mut().enumerate() {
                            if ((i as i32) * 2) >= mouse_hit_layer {
                                area.handle_mouse_button(
                                    &mut state,
                                    &self.mode_stack,
                                    button,
                                    action,
                                    modifiers,
                                );
                            }
                        }
                    }
                }
                WindowEvent::Scroll(x, y) => {
                    let mut state = self.mutable_state.borrow_mut();
                    for (i, area) in self.area_manager.area_map.values_mut().enumerate() {
                        if ((i as i32) * 2) >= mouse_hit_layer {
                            area.handle_mouse_scroll(&mut state, Vector::new(x as f32, y as f32));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn toggle_visibility(&self, sketch_id: u16) {
        for s in self.mutable_state.borrow_mut().scene.sketches.iter_mut() {
            if s.id == sketch_id {
                s.visible = !s.visible;
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let original_size = Vector::new(1000.0, 800.0);

        // Try to load saved layout first
        let area_manager = match std::fs::read_to_string("layout.json") {
            Ok(json) => match serde_json::from_str::<AreaManager>(&json) {
                Ok(manager) => {
                    info!("Loaded layout from layout.json on startup");
                    manager
                }
                Err(e) => {
                    error!("Failed to deserialize layout on startup: {}", e);
                    AreaManager::new(original_size)
                }
            },
            Err(_) => {
                // File doesn't exist, create default layout
                error!("No saved layout found, creating default layout");
                AreaManager::new(original_size)
            }
        };

        let mut sketch = cad::sketch::Sketch::new("Test sketch".into());
        let p1 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: glm::vec2(0.0, 0.0),
        });
        let p2 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: glm::vec2(1.0, 0.0),
        });
        let p3 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: glm::vec2(0.0, 1.0),
        });
        // Doesnt matter for rendering atm
        let l1 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: glm::vec2(0.0, 0.0),
            direction: glm::vec2(0.0, 0.0),
        });
        let l2 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: glm::vec2(0.0, 0.0),
            direction: glm::vec2(0.0, 0.0),
        });
        let l3 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: glm::vec2(0.0, 0.0),
            direction: glm::vec2(0.0, 0.0),
        });
        sketch.topo_entities.insert(
            Edge::CappedLine {
                start: p1,
                end: p2,
                line: l1,
            }
            .into(),
        );
        sketch.topo_entities.insert(
            Edge::CappedLine {
                start: p1,
                end: p3,
                line: l2,
            }
            .into(),
        );
        sketch.topo_entities.insert(
            Edge::CappedLine {
                start: p2,
                end: p3,
                line: l3,
            }
            .into(),
        );
        sketch.topo_entities.insert(TopoEntity::Point { id: p1 });
        sketch.topo_entities.insert(TopoEntity::Point { id: p2 });
        sketch.topo_entities.insert(TopoEntity::Point { id: p3 });

        let circle = sketch.geo_entities.insert(GeometricEntity::Circle {
            pos: glm::vec2(0.5, 0.5),
            radius: 0.3,
        });
        sketch
            .topo_entities
            .insert(TopoEntity::Circle { id: circle });

        let scene = Scene {
            path: None,
            sketches: vec![
                SketchInfo {
                    id: 0,
                    plane: Plane {
                        x: glm::vec3(1.0, 0.0, 0.0),
                        y: glm::vec3(0.0, 1.0, 0.0),
                    },
                    sketch: sketch.clone(),
                    name: "Sketch 1".into(),
                    visible: true,
                },
                SketchInfo {
                    id: 1,
                    plane: Plane {
                        x: glm::vec3(0.0, 0.0, 1.0),
                        y: glm::vec3(0.0, 1.0, 0.0),
                    },
                    sketch: sketch.clone(),
                    name: "Sketch 2".into(),
                    visible: true,
                },
            ],
            solids: vec![],
        };

        Self {
            perf_overlay: PerformanceOverlay::default(),
            dragging_boundary: None,
            mouse_pos: Vector::zero(),
            original_window_size: original_size,
            area_manager,
            debug_draw: false,
            debug_picker: true,
            settings: Settings {},
            settings_open: false,
            sketch_renderer: SketchRenderer::new(),
            sketch_picker: SketchPicker::new(original_size.x as i32, original_size.y as i32),
            mutable_state: RefCell::new(AppMutableState {
                scene,
                sketch_mode_data: SketchModeData::default(),
                point_mode_data: PointModeData::default(),
                line_mode_data: LineModeData::default(),
                circle_mode_data: CircleModeData::default(),
            }),
            config: default_config(),
            mode_stack: ModeStack::with_base(AppMode::Base),
        }
    }
}

impl AppState for App {
    type SpriteKey = String;

    fn generate_layout(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        {
            let loops: Vec<Face> = self.mutable_state.borrow().scene.sketches[0]
                .sketch
                .loops()
                .map(|x| x.clone())
                .collect();
            visual_log("loops", format!("{:?}", loops));
        }
        let mut out = vec![];
        out.extend(self.base_layer(window_size));
        if self.perf_overlay.visible {
            out.push(self.perf_overlay.generate_layout(window_size));
        }
        if self.settings_open {
            out.push(self.settings.generate_layout(window_size));
        }
        out
    }

    fn handle_key(&mut self, key: Key, scancode: Scancode, action: Action, modifiers: Modifiers) {
        #[allow(clippy::single_match)]
        let mut state = self.mutable_state.borrow_mut();

        if action == Action::Release {
            match glfw_key_to_key_input(key, modifiers) {
                Some(key_input) => {
                    if let Some(action) = self
                        .mode_stack
                        .dispatch(&mut self.config.bindings, key_input)
                    {
                        match action {
                            AppBindableMessage::PopMode => {
                                self.mode_stack.pop();
                            }
                            AppBindableMessage::ToggleSettings => {
                                self.settings_open = !self.settings_open;
                            }
                            AppBindableMessage::ToggleProjection => {
                                for area in self.area_manager.area_map.values_mut() {
                                    if let crate::ui::area::AreaData::Viewport(ref mut vp_data) =
                                        area.area_data
                                    {
                                        vp_data.projection_mode = match vp_data.projection_mode {
                                            viewport::ProjectionMode::Perspective => {
                                                viewport::ProjectionMode::Orthographic
                                            }
                                            viewport::ProjectionMode::Orthographic => {
                                                viewport::ProjectionMode::Perspective
                                            }
                                        };
                                    }
                                }
                            }
                            AppBindableMessage::ToggleDebugDraw => {
                                self.debug_draw = !self.debug_draw;
                            }
                            AppBindableMessage::DumpDebugPick => {
                                self.debug_picker = !self.debug_picker;
                                let filename = format!(
                                    "picker_dump_{}.png",
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs()
                                );
                                if let Err(e) = self.sketch_picker.picker.dump_to_png(
                                    self.sketch_picker.window_width,
                                    self.sketch_picker.window_height,
                                    &filename,
                                ) {
                                    error!("Failed to dump picker framebuffer: {}", e);
                                } else {
                                    info!("Dumped picker framebuffer to {}", filename);
                                }
                            }
                            AppBindableMessage::TogglePerformanceOverlay => {
                                self.perf_overlay.visible = !self.perf_overlay.visible;
                            }
                            AppBindableMessage::SplitAreaHorizontally => {
                                self.area_manager
                                    .split_area(self.mouse_pos, BoundaryOrientation::Horizontal);
                            }
                            AppBindableMessage::SplitAreaVertically => {
                                self.area_manager
                                    .split_area(self.mouse_pos, BoundaryOrientation::Vertical);
                            }
                            AppBindableMessage::CollapseBoundary => {
                                self.area_manager.collapse_boundary(self.mouse_pos);
                            }
                            AppBindableMessage::ActivatePointMode => {
                                self.mode_stack.push(AppMode::Point);
                            }
                            AppBindableMessage::Confirm => {
                                debug!("Confirm!");
                                match self.mode_stack.outermost().unwrap() {
                                    AppMode::Line => {
                                        let sid = state.sketch_mode_data.sketch_id;
                                        let points =
                                            std::mem::take(&mut state.line_mode_data.points);
                                        let sketch = state
                                            .scene
                                            .sketches
                                            .iter_mut()
                                            .find(|s| s.id == sid)
                                            .unwrap();
                                        sketch.sketch.insert_capped_lines(&points);
                                        self.mode_stack.pop_until(&AppMode::Sketch);
                                    }
                                    AppMode::Circle => todo!(),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                None => {
                    error!(
                        "Couldn't convert GLFW key {:?} {:?} {:?} to keybinds-key",
                        key, modifiers, action
                    );
                }
            }
        }

        for area in self.area_manager.area_map.values_mut() {
            area.handle_key(&mut state, key, scancode, action, modifiers);
        }
    }

    fn handle_mouse_position(&mut self, position: Vector<f32>, delta: Vector<f32>) {
        self.mouse_pos = position;
        if let Some(bid) = self.dragging_boundary {
            self.area_manager.move_boundary(self.mouse_pos, bid);
        }
        let mut state = self.mutable_state.borrow_mut();
        for area in self.area_manager.area_map.values_mut() {
            area.handle_mouse_position(&mut state, &self.mode_stack, position, delta);
        }
    }

    fn handle_mouse_button(
        &mut self,
        _button: glfw::MouseButton,
        _action: Action,
        _modifiers: Modifiers,
    ) {
    }

    fn handle_mouse_scroll(&mut self, _scroll_delta: Vector<f32>) {}
}
