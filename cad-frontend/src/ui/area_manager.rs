use cad::registry::Registry;
use rust_ui::geometry::{Rect, Vector};
use serde::{Deserialize, Serialize};

use super::{
    area::{Area, AreaId, AreaType},
    boundary::{Boundary, BoundaryId, BoundaryOrientation},
};

const DEFAULT_BDRY_TOLERANCE: f32 = 5.0;

fn default_bdry_tolerance() -> f32 {
    DEFAULT_BDRY_TOLERANCE
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaManager {
    pub area_map: Registry<AreaId, Area>,
    pub bdry_map: Registry<BoundaryId, Boundary>,
    #[serde(default = "default_bdry_tolerance")]
    pub bdry_tolerance: f32,
}

impl AreaManager {
    pub fn new(original_size: Vector<f32>) -> Self {
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
        Self {
            area_map,
            bdry_map: Registry::new(),
            bdry_tolerance: DEFAULT_BDRY_TOLERANCE,
        }
    }

    pub fn split_area(&mut self, pos: Vector<f32>, orientation: BoundaryOrientation) {
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
            if let Some(existing_bdry) = self.bdry_map.get_mut(&id)
                && existing_bdry.orientation != bdry.orientation
            {
                existing_bdry.side2.push(next_aid);
            }
        }
        self.bdry_map.insert(bdry);
    }

    pub fn collapse_boundary(&mut self, pos: Vector<f32>) {
        if let Some(hovered) = self.find_boundary(pos)
            && self.bdry_map[hovered].can_collapse()
        {
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

    pub fn find_area(&self, pos: Vector<f32>) -> Option<AreaId> {
        self.area_map
            .iter()
            .find(|(_, area)| area.bbox.contains(pos))
            .map(|(id, _)| *id)
    }

    pub fn find_boundary(&self, pos: Vector<f32>) -> Option<BoundaryId> {
        let mut out = None;
        let mut closest_dist = f32::INFINITY;
        for (id, bdry) in self.bdry_map.iter() {
            let dist = self.distance_to_point(bdry, pos);
            if dist < closest_dist {
                out = Some(*id);
                closest_dist = dist;
            }
        }
        if closest_dist < self.bdry_tolerance {
            out
        } else {
            None
        }
    }

    pub fn further_down_bdry_tree(&self, id: &AreaId) -> Vec<BoundaryId> {
        let mut out = vec![];
        for (bid, bdry) in self.bdry_map.iter() {
            if bdry.side1.contains(id) {
                out.push(*bid);
            }
        }
        out
    }

    pub fn further_up_bdry_tree(&self, id: &AreaId) -> Vec<BoundaryId> {
        let mut out = vec![];
        for (bid, bdry) in self.bdry_map.iter() {
            if bdry.side2.contains(id) {
                out.push(*bid);
            }
        }
        out
    }

    pub fn distance_to_point(&self, bdry: &Boundary, pos: Vector<f32>) -> f32 {
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

    pub fn extent(&self, bdry: &Boundary) -> f32 {
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
                total1.max(total2)
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
                total1.max(total2)
            }
        }
    }

    pub fn move_boundary(&mut self, end_pos: Vector<f32>, bid: BoundaryId) {
        let bdry = &self.bdry_map[bid];
        match bdry.orientation {
            BoundaryOrientation::Horizontal => {
                for aid in &bdry.side1.clone() {
                    self.area_map[*aid].bbox.x1.y = end_pos.y;
                }
                for aid in &bdry.side2.clone() {
                    self.area_map[*aid].bbox.x0.y = end_pos.y;
                }
            }
            BoundaryOrientation::Vertical => {
                for aid in &bdry.side1.clone() {
                    self.area_map[*aid].bbox.x1.x = end_pos.x;
                }
                for aid in &bdry.side2.clone() {
                    self.area_map[*aid].bbox.x0.x = end_pos.x;
                }
            }
        }
    }

    pub fn resize_areas(&mut self, old_window_size: Vector<f32>, new_window_size: Vector<f32>) {
        let scale_x = new_window_size.x / old_window_size.x;
        let scale_y = new_window_size.y / old_window_size.y;

        for area in self.area_map.values_mut() {
            area.bbox.x0.x *= scale_x;
            area.bbox.x0.y *= scale_y;
            area.bbox.x1.x *= scale_x;
            area.bbox.x1.y *= scale_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Produces a layout with 3 areas next to each other
    fn collapse_area_edge_case_reconnection() {
        let mut manager = AreaManager::new(Vector::new(1000.0, 800.0));
        manager.split_area(Vector::new(275.0, 385.0), BoundaryOrientation::Vertical);
        manager.split_area(Vector::new(718.0, 391.0), BoundaryOrientation::Vertical);
        manager.collapse_boundary(Vector::new(500.0, 442.0));

        assert!(
            manager.bdry_map.len() == 1,
            "There should be 1 boundary left"
        );
        let bdry = &manager.bdry_map[BoundaryId(1)];
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
