use cad::registry::RegId;
use rust_ui::{
    geometry::Vector,
    render::{
        Color, NORD9, NORD11, NORD14,
        renderer::{Anchor, NodeContext, RenderLayout},
    },
};
use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, Dimension, FlexDirection, Size, Style, TaffyTree, prelude::length};

use crate::app::App;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum AreaType {
    Red,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize)]
pub struct AreaId(pub i64);

impl RegId for AreaId {
    fn new() -> Self {
        Self(0)
    }

    fn increment(self) -> Self {
        let AreaId(id) = self;
        Self(id + 1)
    }
}

impl Default for AreaId {
    fn default() -> Self {
        AreaId(-1)
    }
}

#[derive(Serialize)]
pub struct Area {
    pub id: AreaId,
    pub area_type: AreaType,
}

impl Area {
    pub fn new(id: AreaId, area_type: AreaType) -> Self {
        Self { id, area_type }
    }

    pub fn generate_layout(&mut self, size: rust_ui::geometry::Vector<f32>) -> RenderLayout<App> {
        let mut tree = TaffyTree::new();

        // TODO:
        // Area type picker

        let root = tree
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: Dimension::percent(1.0),
                        height: Dimension::percent(1.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    bg_color: match self.area_type {
                        AreaType::Red => NORD11,
                        AreaType::Green => NORD14,
                        AreaType::Blue => NORD9,
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        RenderLayout {
            tree,
            root,
            desired_size: Size {
                width: AvailableSpace::Definite(size.x),
                height: AvailableSpace::Definite(size.y),
            },
            root_pos: Vector::zero(),
            anchor: Anchor::TopLeft,
            scissor: true,
        }
    }
}
