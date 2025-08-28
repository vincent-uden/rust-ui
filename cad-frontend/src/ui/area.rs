use cad::registry::RegId;
use rust_ui::{
    geometry::{Rect, Vector},
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

#[derive(Debug, Serialize)]
pub struct Area {
    pub id: AreaId,
    pub area_type: AreaType,
    pub bbox: Rect<f32>,
}

impl Area {
    pub fn new(id: AreaId, area_type: AreaType, bbox: Rect<f32>) -> Self {
        Self {
            id,
            area_type,
            bbox,
        }
    }

    pub fn generate_layout(&mut self) -> RenderLayout<App> {
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
                width: AvailableSpace::Definite(self.bbox.width()),
                height: AvailableSpace::Definite(self.bbox.height()),
            },
            root_pos: self.bbox.x0,
            anchor: Anchor::TopLeft,
            scissor: true,
        }
    }
}
