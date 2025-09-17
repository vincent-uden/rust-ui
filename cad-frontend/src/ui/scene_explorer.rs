use std::{path::PathBuf, str::FromStr as _, sync::Arc};

use cad::Scene;
use rust_ui::render::{
    COLOR_LIGHT, NORD3, Text,
    renderer::{NodeContext, flags},
};
use taffy::{AlignItems, FlexDirection, NodeId, Rect, Size, Style, TaffyTree, prelude::length};

use crate::app::App;

#[derive(Debug, Clone, Copy)]
pub struct SceneExplorer {}

impl SceneExplorer {
    pub fn generate_layout(tree: &mut TaffyTree<NodeContext<App>>, parent: NodeId, scene: &Scene) {
        let header = tree
            .new_leaf_with_context(
                Style {
                    margin: Rect {
                        left: length(0.0),
                        right: length(0.0),
                        top: length(0.0),
                        bottom: length(12.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: scene
                            .path
                            .as_ref()
                            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                            .unwrap_or_else(|| "Untitled".into()),
                        font_size: 18,
                        color: COLOR_LIGHT,
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        let container = tree
            .new_with_children(
                Style {
                    padding: Rect {
                        left: length(8.0),
                        right: length(8.0),
                        top: length(30.0),
                        bottom: length(8.0),
                    },
                    flex_direction: taffy::FlexDirection::Column,
                    gap: length(4.0),
                    ..Default::default()
                },
                &[header],
            )
            .unwrap();
        for (i, sketch) in scene.sketches.iter().enumerate() {
            let row = tree
                .new_leaf(Style {
                    flex_direction: FlexDirection::Row,
                    gap: length(8.0),
                    align_items: Some(AlignItems::Center),
                    ..Default::default()
                })
                .unwrap();
            let s = tree
                .new_leaf_with_context(
                    Style::default(),
                    NodeContext {
                        flags: flags::TEXT,
                        text: Text {
                            text: sketch.name.clone(),
                            font_size: 14,
                            color: if sketch.visible { COLOR_LIGHT } else { NORD3 },
                        },
                        on_mouse_up: Some(Arc::new(move |state| {
                            for (j, s) in state.app_state.scene.sketches.iter_mut().enumerate() {
                                if j == i {
                                    s.visible = !s.visible;
                                }
                            }
                        })),
                        ..Default::default()
                    },
                )
                .unwrap();
            let visibility = tree
                .new_leaf_with_context(
                    Style {
                        size: Size::length(30.0),
                        ..Default::default()
                    },
                    NodeContext {
                        flags: flags::SPRITE,
                        sprite_key: if sketch.visible {
                            "Visible".into()
                        } else {
                            "Invisible".into()
                        },
                        ..Default::default()
                    },
                )
                .unwrap();
            tree.add_child(row, s).unwrap();
            tree.add_child(row, visibility).unwrap();
            tree.add_child(container, row).unwrap();
        }
        tree.add_child(parent, container).unwrap();
    }
}
