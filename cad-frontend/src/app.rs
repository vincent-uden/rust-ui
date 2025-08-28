use cad::registry::Registry;
use glfw::{Action, Key, Modifiers, Scancode};
use rust_ui::{
    geometry::{Rect, Vector},
    render::renderer::{AppState, RenderLayout},
};
use tracing::{debug, error};

use crate::ui::{
    area::{Area, AreaId, AreaType},
    boundary::{Boundary, BoundaryId, BoundaryOrientation},
    perf_overlay::PerformanceOverlay,
};

pub struct App {
    pub perf_overlay: PerformanceOverlay,
    pub mouse_pos: Vector<f32>,
    area_map: Registry<AreaId, Area>,
    bdry_map: Registry<BoundaryId, Boundary>,
}

impl App {
    pub fn update(&mut self) {}

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
                if existing_bdry.orientation != existing_bdry.orientation {
                    existing_bdry.side2.push(next_aid);
                }
            }
        }
        self.bdry_map.insert(bdry);
    }

    fn find_area(&self, pos: Vector<f32>) -> Option<AreaId> {
        self.area_map
            .iter()
            .find(|(_, area)| area.bbox.contains(pos))
            .map(|(id, _)| *id)
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
                Key::F12 => {
                    self.perf_overlay.visible = !self.perf_overlay.visible;
                }
                Key::H => {
                    self.split_area(self.mouse_pos, BoundaryOrientation::Horizontal);
                }
                Key::V => {
                    self.split_area(self.mouse_pos, BoundaryOrientation::Vertical);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
