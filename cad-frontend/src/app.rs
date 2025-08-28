use cad::registry::Registry;
use glfw::{Action, Key, Modifiers, Scancode};
use rust_ui::{
    geometry::Vector,
    render::{
        Border, BorderRadius, COLOR_LIGHT, Text,
        renderer::{Anchor, AppState, NodeContext, RenderLayout, flags},
    },
};
use taffy::{
    FlexDirection, NodeId, Rect, Size, Style, TaffyTree,
    prelude::{TaffyMaxContent, length},
};

use crate::ui::{
    area::{Area, AreaId, AreaType},
    boundary::{Boundary, BoundaryId},
    perf_overlay::PerformanceOverlay,
};

pub struct App {
    pub perf_overlay: PerformanceOverlay,
    area_map: Registry<AreaId, Area>,
    bdry_map: Registry<BoundaryId, Boundary>,
}

impl App {
    pub fn update(&mut self) {}

    fn base_layer(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        let mut out = vec![];
        // Areas can't be calculated using taffy since they're a graph, not a tree.
        // Return one RenderLayout per area. They will technically be on different layers, but that
        // doesn' matter as they'll all be scissored.
        let root_area = self.area_map.get(&AreaId(0));

        for area in self.area_map.values_mut() {
            // TODO: Replace with the areas actual size
            out.push(area.generate_layout(window_size));
        }

        out
    }

    fn further_down_bdr_tree(self, id: &AreaId) -> Vec<BoundaryId> {
        let mut out = vec![];
        for (bid, bdry) in self.bdry_map.iter() {
            if bdry.side1.contains(id) {
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
        area_map.insert(Area::new(id, AreaType::Red));
        Self {
            perf_overlay: PerformanceOverlay::default(),
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
        match key {
            Key::F12 => match action {
                Action::Release => {
                    self.perf_overlay.visible = !self.perf_overlay.visible;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
