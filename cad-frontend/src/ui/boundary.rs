use cad::registry::RegId;
use serde::{Deserialize, Serialize};

use crate::ui::area::AreaId;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum BoundaryOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize)]
pub struct BoundaryId(pub i64);

impl RegId for BoundaryId {
    fn new() -> Self {
        Self(0)
    }

    fn increment(self) -> Self {
        let BoundaryId(id) = self;
        Self(id + 1)
    }
}

impl Default for BoundaryId {
    fn default() -> Self {
        BoundaryId(-1)
    }
}

#[derive(Debug)]
pub struct Boundary {
    pub id: BoundaryId,
    pub orientation: BoundaryOrientation,
    pub active: bool,
    pub thickness: i32,
    pub hovered_thickness: i32,
    pub hovered: bool,
    // Left / Up
    pub side1: Vec<AreaId>,
    // Down / Right
    pub side2: Vec<AreaId>,
}

impl Boundary {
    pub fn new(id: BoundaryId, orientation: BoundaryOrientation) -> Self {
        Self {
            id,
            orientation,
            active: false,
            thickness: 3,
            hovered_thickness: 6,
            side1: vec![],
            side2: vec![],
            hovered: false,
        }
    }
}
