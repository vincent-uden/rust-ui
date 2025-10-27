use std::sync::Arc;

use glm::vec3;
use rust_ui::{
    geometry::Vector,
    render::{
        COLOR_LIGHT, NORD1, NORD3, NORD7, Text,
        renderer::{NodeContext, Renderer, UiBuilder, flags},
    },
};
use taffy::{
    AlignItems, Dimension, FlexDirection, NodeId, Rect, Size, Style, TaffyTree,
    prelude::{auto, length},
};

use crate::app::{self, App, AppMutableState, SketchMode};

#[derive(Debug, Clone, Copy)]
pub struct Modes {}

impl Modes {
    pub fn generate_layout(
        tree: &mut TaffyTree<NodeContext<App>>,
        parent: NodeId,
        state: &AppMutableState,
    ) {
        let container = tree
            .new_with_children(
                Style {
                    padding: Rect {
                        left: length(8.0),
                        right: length(8.0),
                        top: length(30.0),
                        bottom: length(8.0),
                    },
                    flex_direction: taffy::FlexDirection::Row,
                    gap: length(8.0),
                    align_items: Some(AlignItems::Stretch),
                    size: Size {
                        width: Dimension::percent(1.0),
                        height: auto(),
                    },
                    ..Default::default()
                },
                &[],
            )
            .unwrap();
        match &state.mode {
            app::Mode::EditSketch(i, sketch_mode) => {
                let i = *i;
                let buttons: Vec<(&str, Arc<dyn Fn(&mut Renderer<App>)>)> = vec![
                    (
                        "Point",
                        Arc::new(move |state| {
                            state.app_state.mutable_state.borrow_mut().mode =
                                app::Mode::EditSketch(i, SketchMode::Point);
                        }),
                    ),
                    (
                        "Finish Sketch",
                        Arc::new(|state| {
                            state.app_state.mutable_state.borrow_mut().mode = app::Mode::None;
                        }),
                    ),
                ];
                for b in buttons {
                    mode_button(tree, container, b);
                }
            }
            app::Mode::None => {
                let buttons: Vec<(&str, Arc<dyn Fn(&mut Renderer<App>)>)> = vec![(
                    "New sketch",
                    Arc::new(|state| {
                        let mut mut_state = state.app_state.mutable_state.borrow_mut();
                        // TODO: Pick the plane
                        mut_state.scene.add_sketch(cad::Plane {
                            x: vec3(1.0, 0.0, 0.0),
                            y: vec3(0.0, 1.0, 0.0),
                        });
                    }),
                )];
                for b in buttons {
                    mode_button(tree, container, b);
                }
            }
        }
        tree.add_child(parent, container).unwrap();
    }
}

fn mode_button(
    tree: &mut TaffyTree<NodeContext<App>>,
    parent: NodeId,
    (label, on_click): (&str, Arc<dyn Fn(&mut Renderer<App>)>),
) {
    let square = tree
        .new_leaf_with_context(
            Style {
                padding: Rect::length(4.0),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            },
            NodeContext {
                flags: flags::HOVER_BG,
                bg_color: NORD1,
                bg_color_hover: NORD3,
                on_left_mouse_up: Some(on_click),
                ..Default::default()
            },
        )
        .unwrap();

    let text = tree
        .new_leaf_with_context(
            Style {
                ..Default::default()
            },
            NodeContext {
                flags: flags::TEXT,
                text: Text {
                    text: label.into(),
                    font_size: 14,
                    color: COLOR_LIGHT,
                },
                ..Default::default()
            },
        )
        .unwrap();

    tree.add_child(square, text).unwrap();
    tree.add_child(parent, square).unwrap();
}
