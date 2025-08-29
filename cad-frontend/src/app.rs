use core::f32;

use cad::registry::Registry;
use glfw::{Action, Key, Modifiers, Scancode};
use glm::orientation;
use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        Color,
        line::LineRenderer,
        renderer::{AppState, RenderLayout},
    },
};
use tracing::{debug, error};

use crate::ui::{
    area::{Area, AreaId, AreaType},
    boundary::{Boundary, BoundaryId, BoundaryOrientation},
    perf_overlay::PerformanceOverlay,
};

const BDRY_TOLERANCE: f32 = 5.0;

pub struct App {
    pub perf_overlay: PerformanceOverlay,
    pub mouse_pos: Vector<f32>,
    pub debug_draw: bool, // Eventually turn this into a menu
    area_map: Registry<AreaId, Area>,
    bdry_map: Registry<BoundaryId, Boundary>,
}

impl App {
    fn base_layer(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        let mut out = vec![];
        // Areas can't be calculated using taffy since they're a directed graph, not a tree.
        // Return one RenderLayout per area. They will technically be on different layers, but that
        // doesn' matter as they'll all be scissored.
        let root_area = self.area_map.get(&AreaId(0));

        for area in self.area_map.values_mut() {
            // TODO: Replace with the areas actual size
            out.push(area.generate_layout());
        }

        out
    }

    // TODO: (Next) Horizontal -> Vertical split should result in a vertical split that isnt
    // collapsable. Currently it is. Which is wrong
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
            let new_area = Area::new(
                next_aid,
                match to_split.area_type {
                    AreaType::Red => AreaType::Blue,
                    AreaType::Green => AreaType::Red,
                    AreaType::Blue => AreaType::Green,
                },
                new,
            );
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
}

impl Default for App {
    fn default() -> Self {
        let mut area_map = Registry::new();
        let id = area_map.next_id();
        area_map.insert(Area::new(
            id,
            AreaType::Red,
            Rect {
                x0: Vector::new(0.0, 0.0),
                x1: Vector::new(1000.0, 800.0),
            },
        ));
        Self {
            perf_overlay: PerformanceOverlay::default(),
            mouse_pos: Vector::zero(),
            area_map,
            bdry_map: Registry::new(),
            debug_draw: false,
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
        out
    }

    fn handle_key(&mut self, key: Key, _scancode: Scancode, action: Action, _modifiers: Modifiers) {
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
                _ => {}
            },
            _ => {}
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
