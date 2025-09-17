use core::f32;
use std::{f64::consts::PI, time::Instant};

use cad::{
    Plane, Scene, SketchInfo,
    entity::{FundamentalEntity, GuidedEntity, Line, Point},
    registry::Registry,
};
use glfw::{Action, Key, Modifiers, Scancode};
use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        Color,
        line::LineRenderer,
        renderer::{AppState, RenderLayout},
    },
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{
    sketch_renderer::SketchRenderer,
    ui::{
        area::{Area, AreaData, AreaId, AreaType},
        boundary::{Boundary, BoundaryId, BoundaryOrientation},
        perf_overlay::PerformanceOverlay,
        settings::Settings,
        viewport::{self, ViewportData},
    },
};

#[derive(Serialize, Deserialize)]
struct AreaSerializer {
    pub area_map: Registry<AreaId, Area>,
    pub bdry_map: Registry<BoundaryId, Boundary>,
}

const BDRY_TOLERANCE: f32 = 5.0;

pub struct App {
    pub perf_overlay: PerformanceOverlay,
    pub dragging_boundary: Option<BoundaryId>,
    pub mouse_pos: Vector<f32>,
    pub debug_draw: bool, // Eventually turn this into a menu
    pub original_window_size: Vector<f32>,
    pub area_map: Registry<AreaId, Area>,
    pub bdry_map: Registry<BoundaryId, Boundary>,
    pub settings: Settings,
    pub settings_open: bool,
    pub sketch_renderer: SketchRenderer,
    pub scene: Scene,
}

impl App {
    fn base_layer(&mut self, _window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        let mut out = vec![];
        // Areas can't be calculated using taffy since they're a directed graph, not a tree.
        // Return one RenderLayout per area. They will technically be on different layers, but that
        // doesn' matter as they'll all be scissored.

        for area in self.area_map.values_mut() {
            out.push(area.generate_layout(&self.scene));
        }
        for area in self.area_map.values_mut() {
            out.push(area.area_kind_picker_layout());
        }

        out
    }

    fn split_area(&mut self, pos: Vector<f32>, orientation: BoundaryOrientation) {
        let next_aid = self.area_map.next_id();
        let to_split_aid = self.find_area(pos).unwrap();
        if let Some(to_split) = self.area_map.get_mut(&to_split_aid) {
            // Despite the confusing name, this is correct. If the boundary is horizontal, the
            // areas should be above and below each other, thus splitting the area in half
            // vertically
            let (old, new) = match orientation {
                BoundaryOrientation::Horizontal => to_split.bbox.split_vertically(),
                BoundaryOrientation::Vertical => to_split.bbox.split_horizontally(),
            };
            to_split.bbox = old;
            let new_area = Area::new(next_aid, AreaType::Green, new);
            self.area_map.insert(new_area);
        }

        let next_bid = self.bdry_map.next_id();
        let mut bdry = Boundary::new(next_bid, orientation);
        bdry.side1.push(to_split_aid);
        bdry.side2.push(next_aid);

        for id in self.further_down_bdry_tree(&to_split_aid) {
            if let Some(existing_bdry) = self.bdry_map.get_mut(&id) {
                if existing_bdry.orientation == bdry.orientation {
                    existing_bdry.side1.retain(|id| *id != to_split_aid);
                }
                existing_bdry.side1.push(next_aid);
            }
        }
        for id in self.further_up_bdry_tree(&to_split_aid) {
            if let Some(existing_bdry) = self.bdry_map.get_mut(&id) {
                if existing_bdry.orientation != bdry.orientation {
                    existing_bdry.side2.push(next_aid);
                }
            }
        }
        self.bdry_map.insert(bdry);
    }

    fn collapse_boundary(&mut self, pos: Vector<f32>) {
        if let Some(hovered) = self.find_boundary(pos) {
            if self.bdry_map[hovered].can_collapse() {
                let bdry = self.bdry_map.remove(&hovered).unwrap();
                let deleted_dims = self.area_map[bdry.side2[0]].bbox;
                let remaining_area = &mut self.area_map[bdry.side1[0]];
                match bdry.orientation {
                    BoundaryOrientation::Horizontal => {
                        remaining_area.bbox.x1.y += deleted_dims.height();
                    }
                    BoundaryOrientation::Vertical => {
                        remaining_area.bbox.x1.x += deleted_dims.width();
                    }
                }
                let to_delete = &self.area_map[bdry.side2[0]];
                for bid in self.further_down_bdry_tree(&to_delete.id) {
                    let b = &mut self.bdry_map[bid];
                    if !b.side1.contains(&bdry.side1[0]) {
                        b.side1.push(bdry.side1[0]);
                    }
                }
                for b in self.bdry_map.values_mut() {
                    b.side1.retain(|x| *x != to_delete.id);
                    b.side2.retain(|x| *x != to_delete.id);
                }
                self.area_map.remove(&bdry.side2[0]);
            }
        }
    }

    fn find_area(&self, pos: Vector<f32>) -> Option<AreaId> {
        self.area_map
            .iter()
            .find(|(_, area)| area.bbox.contains(pos))
            .map(|(id, _)| *id)
    }

    fn find_boundary(&self, pos: Vector<f32>) -> Option<BoundaryId> {
        let mut out = None;
        let mut closest_dist = f32::INFINITY;
        for (id, bdry) in self.bdry_map.iter() {
            let dist = self.distance_to_point(bdry, pos);
            if dist < closest_dist {
                out = Some(*id);
                closest_dist = dist;
            }
        }
        if closest_dist < BDRY_TOLERANCE {
            out
        } else {
            None
        }
    }

    fn further_down_bdry_tree(&self, id: &AreaId) -> Vec<BoundaryId> {
        let mut out = vec![];
        for (bid, bdry) in self.bdry_map.iter() {
            if bdry.side1.contains(id) {
                out.push(*bid);
            }
        }
        out
    }

    fn further_up_bdry_tree(&self, id: &AreaId) -> Vec<BoundaryId> {
        let mut out = vec![];
        for (bid, bdry) in self.bdry_map.iter() {
            if bdry.side2.contains(id) {
                out.push(*bid);
            }
        }
        out
    }

    fn distance_to_point(&self, bdry: &Boundary, pos: Vector<f32>) -> f32 {
        let area = &self.area_map[bdry.side2[0]];
        match bdry.orientation {
            BoundaryOrientation::Horizontal => {
                if pos.x > area.bbox.x0.x && pos.x < area.bbox.x0.x + self.extent(bdry) {
                    return (area.bbox.x0.y - pos.y).abs();
                }
            }
            BoundaryOrientation::Vertical => {
                if pos.y > area.bbox.x0.y && pos.y < area.bbox.x0.y + self.extent(bdry) {
                    return (area.bbox.x0.x - pos.x).abs();
                }
            }
        }
        f32::INFINITY
    }

    fn extent(&self, bdry: &Boundary) -> f32 {
        let total;
        match bdry.orientation {
            BoundaryOrientation::Horizontal => {
                let mut total1 = 0.0;
                let mut total2 = 0.0;
                for area_id in &bdry.side1 {
                    let area = &self.area_map[*area_id];
                    total1 += area.bbox.width();
                }
                for area_id in &bdry.side2 {
                    let area = &self.area_map[*area_id];
                    total2 += area.bbox.width();
                }
                total = total1.max(total2);
            }
            BoundaryOrientation::Vertical => {
                let mut total1 = 0.0;
                let mut total2 = 0.0;
                for area_id in &bdry.side1 {
                    let area = &self.area_map[*area_id];
                    total1 += area.bbox.height();
                }
                for area_id in &bdry.side2 {
                    let area = &self.area_map[*area_id];
                    total2 += area.bbox.height();
                }
                total = total1.max(total2);
            }
        }
        total
    }

    fn move_boundary(&mut self, end_pos: Vector<f32>, bid: BoundaryId) {
        let bdry = &self.bdry_map[bid];
        match bdry.orientation {
            BoundaryOrientation::Horizontal => {
                for aid in &bdry.side1 {
                    self.area_map[*aid].bbox.x1.y = end_pos.y;
                }
                for aid in &bdry.side2 {
                    self.area_map[*aid].bbox.x0.y = end_pos.y;
                }
            }
            BoundaryOrientation::Vertical => {
                for aid in &bdry.side1 {
                    self.area_map[*aid].bbox.x1.x = end_pos.x;
                }
                for aid in &bdry.side2 {
                    self.area_map[*aid].bbox.x0.x = end_pos.x;
                }
            }
        }
    }

    pub fn resize_areas(&mut self, new_window_size: Vector<f32>) {
        let scale_x = new_window_size.x / self.original_window_size.x;
        let scale_y = new_window_size.y / self.original_window_size.y;

        for area in self.area_map.values_mut() {
            area.bbox.x0.x *= scale_x;
            area.bbox.x0.y *= scale_y;
            area.bbox.x1.x *= scale_x;
            area.bbox.x1.y *= scale_y;
        }

        self.original_window_size = new_window_size;
    }

    pub fn debug_draw(&mut self, line_renderer: &LineRenderer, window_size: Vector<f32>) {
        for bdry in self.bdry_map.values() {
            for aid1 in &bdry.side1 {
                let a1 = &self.area_map[*aid1];
                for aid2 in &bdry.side2 {
                    let a2 = &self.area_map[*aid2];
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
        let out = AreaSerializer {
            area_map: self.area_map.clone(),
            bdry_map: self.bdry_map.clone(),
        };
        let json = serde_json::to_string_pretty(&out).expect("Failed to serialize layout");
        std::fs::write("layout.json", json).expect("Failed to write layout file");
        info!("Layout saved to layout.json");
    }

    pub fn load_layout(&mut self) {
        match std::fs::read_to_string("layout.json") {
            Ok(json) => match serde_json::from_str::<AreaSerializer>(&json) {
                Ok(serializer) => {
                    self.area_map = serializer.area_map;
                    self.bdry_map = serializer.bdry_map;
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
        for area in self.area_map.values_mut() {
            area.update();
        }
    }

    /// Some areas contain stuff that isn't part of the regular UI tree such as the viewport that
    /// renders 3D scenes. Those are rendered here, before the UI pass.
    pub fn draw_special_areas(&mut self) {
        for area in self.area_map.values_mut() {
            match area.area_type {
                AreaType::Viewport => {
                    let data: &mut ViewportData = (&mut area.area_data).try_into().unwrap();
                    data.size = self.original_window_size;
                    self.sketch_renderer.draw_axes(data);
                    for si in &self.scene.sketches {
                        if si.visible {
                            self.sketch_renderer.draw(
                                &si.sketch,
                                data,
                                si.plane.x.cast(),
                                si.plane.y.cast(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn create_default_layout(
        original_size: Vector<f32>,
    ) -> (Registry<AreaId, Area>, Registry<BoundaryId, Boundary>) {
        let mut area_map = Registry::new();
        let id = area_map.next_id();
        area_map.insert(Area::new(
            id,
            AreaType::Red,
            Rect {
                x0: Vector::new(0.0, 0.0),
                x1: original_size,
            },
        ));
        (area_map, Registry::new())
    }

    pub fn edit_sketch(&mut self, id: i32) {
        // Move camera
        if let Some(sketch) = self.scene.sketches.iter().find(|s| s.id == id) {
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
            for area in self.area_map.values_mut() {
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
            // TODO: Open some sort of edit mode
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let original_size = Vector::new(1000.0, 800.0);

        // Try to load saved layout first
        let (area_map, bdry_map) = match std::fs::read_to_string("layout.json") {
            Ok(json) => match serde_json::from_str::<AreaSerializer>(&json) {
                Ok(serializer) => {
                    info!("Loaded layout from layout.json on startup");
                    (serializer.area_map, serializer.bdry_map)
                }
                Err(e) => {
                    error!("Failed to deserialize layout on startup: {}", e);
                    Self::create_default_layout(original_size)
                }
            },
            Err(_) => {
                // File doesn't exist, create default layout
                error!("No saved layout found, creating default layout");
                Self::create_default_layout(original_size)
            }
        };

        let mut sketch = cad::sketch::Sketch::new("Test sketch".into());
        let p1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point(Point {
                pos: glm::vec2(0.0, 0.0),
            }));
        let p2 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point(Point {
                pos: glm::vec2(1.0, 0.0),
            }));
        let p3 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point(Point {
                pos: glm::vec2(0.0, 1.0),
            }));
        // Doesnt matter for rendering atm
        let l1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Line(Line {
                offset: glm::vec2(0.0, 0.0),
                direction: glm::vec2(0.0, 0.0),
            }));
        let l2 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Line(Line {
                offset: glm::vec2(0.0, 0.0),
                direction: glm::vec2(0.0, 0.0),
            }));
        let l3 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Line(Line {
                offset: glm::vec2(0.0, 0.0),
                direction: glm::vec2(0.0, 0.0),
            }));
        sketch.guided_entities.insert(GuidedEntity::CappedLine {
            start: p1,
            end: p2,
            line: l1,
        });
        sketch.guided_entities.insert(GuidedEntity::CappedLine {
            start: p1,
            end: p3,
            line: l2,
        });
        sketch.guided_entities.insert(GuidedEntity::CappedLine {
            start: p2,
            end: p3,
            line: l3,
        });

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
        };

        Self {
            perf_overlay: PerformanceOverlay::default(),
            dragging_boundary: None,
            mouse_pos: Vector::zero(),
            original_window_size: original_size,
            area_map,
            bdry_map,
            debug_draw: false,
            settings: Settings {},
            settings_open: false,
            sketch_renderer: SketchRenderer::new(),
            scene,
        }
    }
}

impl AppState for App {
    fn generate_layout(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
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
        match action {
            Action::Release => match key {
                Key::F10 => {
                    self.debug_draw = true;
                }
                Key::F12 => {
                    self.perf_overlay.visible = !self.perf_overlay.visible;
                }
                Key::H => {
                    self.split_area(self.mouse_pos, BoundaryOrientation::Horizontal);
                }
                Key::V => {
                    self.split_area(self.mouse_pos, BoundaryOrientation::Vertical);
                }
                Key::D => {
                    self.collapse_boundary(self.mouse_pos);
                }
                Key::Escape => {
                    self.settings_open = !self.settings_open;
                }
                _ => {}
            },
            _ => {}
        }

        for area in self.area_map.values_mut() {
            area.handle_key(key, scancode, action, modifiers);
        }
    }

    fn handle_mouse_position(&mut self, position: Vector<f32>, delta: Vector<f32>) {
        self.mouse_pos = position;
        if let Some(bid) = self.dragging_boundary {
            self.move_boundary(self.mouse_pos, bid);
        }
        for area in self.area_map.values_mut() {
            area.handle_mouse_position(position, delta);
        }
    }

    fn handle_mouse_button(
        &mut self,
        button: glfw::MouseButton,
        action: Action,
        modifiers: Modifiers,
    ) {
        match action {
            Action::Release => {
                self.dragging_boundary = None;
            }
            Action::Press => match button {
                glfw::MouseButton::Button1 => {
                    for (bid, bdry) in self.bdry_map.iter() {
                        if self.distance_to_point(bdry, self.mouse_pos) < BDRY_TOLERANCE {
                            self.dragging_boundary = Some(*bid);
                        }
                    }
                }
                _ => {}
            },
            Action::Repeat => todo!(),
        }
        if self.dragging_boundary.is_none() {
            for area in self.area_map.values_mut() {
                area.handle_mouse_button(button, action, modifiers);
            }
        }
    }

    fn handle_mouse_scroll(&mut self, scroll_delta: Vector<f32>) {
        for area in self.area_map.values_mut() {
            area.handle_mouse_scroll(scroll_delta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Produces a layout with 3 area next to each other
    pub fn collapse_area_edge_case_reconnection() {
        let mut app = App::default();
        app.split_area(Vector::new(275.0, 385.0), BoundaryOrientation::Vertical);
        app.split_area(Vector::new(718.0, 391.0), BoundaryOrientation::Vertical);
        app.collapse_boundary(Vector::new(500.0, 442.0));

        assert!(app.bdry_map.len() == 1, "There should be 1 boundary left");
        let bdry = &app.bdry_map[BoundaryId(1)];
        assert!(
            bdry.side1 == vec![AreaId(0)],
            "The root area should be on the left of the boundary"
        );
        assert!(
            bdry.side2 == vec![AreaId(2)],
            "The leaf area should be on the right of the boundary"
        );
    }
}
