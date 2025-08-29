use std::sync::Arc;

use cad::registry::RegId;
use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        COLOR_BLACK, COLOR_LIGHT, Color, NORD3, NORD9, NORD11, NORD14, Text,
        renderer::{Anchor, NodeContext, RenderLayout, Renderer, flags},
    },
};
use serde::{Deserialize, Serialize};
use taffy::{
    AvailableSpace, Dimension, FlexDirection, Size, Style, TaffyTree,
    prelude::{TaffyMinContent, auto, length},
};
use tracing::debug;

use crate::app::App;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum AreaType {
    Red,
    Green,
    Blue,
}

impl AreaType {
    pub fn all() -> [AreaType; 3] {
        [AreaType::Red, AreaType::Green, AreaType::Blue]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AreaType::Red => "Red",
            AreaType::Green => "Green",
            AreaType::Blue => "Blue",
        }
    }
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
    pub hovered: Option<usize>,
    pub expanded: Option<usize>,
    pub expand_hovered: Option<usize>,
}

impl Area {
    pub fn new(id: AreaId, area_type: AreaType, bbox: Rect<f32>) -> Self {
        Self {
            id,
            area_type,
            bbox,
            hovered: None,
            expanded: None,
            expand_hovered: None,
        }
    }

    pub fn generate_layout(&mut self) -> RenderLayout<App> {
        let mut tree = TaffyTree::new();
        let id = self.id;

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
                    on_mouse_exit: Some(Arc::new(move |state: &mut Renderer<App>| {
                        let area = &mut state.app_state.area_map[id];
                        area.expanded = None;
                        area.expand_hovered = None;
                        area.hovered = None;
                    })),
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

    pub fn area_kind_picker_layout(&mut self) -> RenderLayout<App> {
        let id = self.id;
        let mut tree = TaffyTree::new();
        let root = tree
            .new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                NodeContext {
                    bg_color: Color::default(),
                    ..Default::default()
                },
            )
            .unwrap();

        let bar = tree
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: Dimension::length(self.bbox.width()),
                        height: Dimension::auto(),
                    },
                    padding: taffy::Rect {
                        left: length(4.0),
                        right: length(4.0),
                        top: length(4.0),
                        bottom: length(4.0),
                    },
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    // Hack to get make the bar text height
                    text: Text {
                        text: "M".into(),
                        font_size: 18,
                        color: Color::default(),
                    },
                    bg_color: COLOR_BLACK,
                    ..Default::default()
                },
            )
            .unwrap();

        let kind_tab = tree
            .new_leaf_with_context(
                Style {
                    padding: taffy::Rect {
                        left: length(4.0),
                        right: length(4.0),
                        top: length(4.0),
                        bottom: length(4.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Area type".into(),
                        font_size: 18,
                        color: COLOR_LIGHT,
                    },
                    bg_color: if self.expand_hovered.is_some() {
                        NORD3
                    } else {
                        COLOR_BLACK
                    },
                    on_mouse_down: Some(Arc::new(move |state: &mut Renderer<App>| {
                        let area = &mut state.app_state.area_map[id];
                        if let Some(_) = area.expanded {
                            area.expanded = None;
                        } else {
                            area.expanded = Some(0);
                        }
                    })),
                    on_mouse_enter: Some(Arc::new(move |state: &mut Renderer<App>| {
                        let area = &mut state.app_state.area_map[id];
                        area.expand_hovered = Some(0);
                    })),
                    on_mouse_exit: Some(Arc::new(move |state: &mut Renderer<App>| {
                        let area = &mut state.app_state.area_map[id];
                        area.expand_hovered = None;
                    })),
                    ..Default::default()
                },
            )
            .unwrap();

        let expanded = tree
            .new_leaf_with_context(
                Style {
                    padding: taffy::Rect {
                        left: length(0.0),
                        right: length(0.0),
                        top: length(-(18.0 + 4.0 + 4.0)),
                        bottom: length(0.0),
                    },
                    flex_direction: FlexDirection::Column,
                    align_self: Some(taffy::AlignItems::Start),
                    ..Default::default()
                },
                NodeContext {
                    bg_color: COLOR_BLACK,
                    ..Default::default()
                },
            )
            .unwrap();

        tree.add_child(expanded, kind_tab).unwrap();

        if self.expanded.is_some() {
            for (i, kind) in AreaType::all().into_iter().enumerate() {
                let id = self.id;
                let node = tree
                    .new_leaf_with_context(
                        Style {
                            size: Size::auto(),
                            padding: taffy::Rect {
                                left: length(4.0),
                                right: length(4.0),
                                top: length(4.0),
                                bottom: length(4.0),
                            },
                            ..Default::default()
                        },
                        NodeContext {
                            flags: flags::TEXT,
                            text: Text {
                                text: kind.name().into(),
                                font_size: 18,
                                color: COLOR_LIGHT,
                            },
                            bg_color: if self.hovered.unwrap_or(usize::MAX) == i {
                                NORD3
                            } else {
                                COLOR_BLACK
                            },
                            on_mouse_up: Some(Arc::new(move |state| {
                                state.app_state.area_map[id].area_type = kind;
                            })),
                            on_mouse_enter: Some(Arc::new(move |state: &mut Renderer<App>| {
                                state.app_state.area_menu_item_enter(id, i);
                            })),
                            on_mouse_exit: Some(Arc::new(move |state: &mut Renderer<App>| {
                                state.app_state.area_menu_item_exit(id, i);
                            })),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                tree.add_child(expanded, node).unwrap();
            }
        }

        tree.add_child(root, bar).unwrap();
        tree.add_child(root, expanded).unwrap();

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
