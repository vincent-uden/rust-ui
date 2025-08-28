use glfw::{Action, Key, Modifiers, Scancode};
use rust_ui::{
    geometry::Vector,
    render::{
        Border, BorderRadius, COLOR_LIGHT, Text,
        renderer::{Anchor, AppState, NodeContext, RenderLayout, flags},
    },
};
use taffy::{
    FlexDirection, Rect, Size, Style, TaffyTree,
    prelude::{TaffyMaxContent, length},
};

use crate::ui::perf_overlay::PerformanceOverlay;

#[derive(Default)]
pub struct App {
    pub perf_overlay: PerformanceOverlay,
}

impl App {
    pub fn update(&mut self) {}

    fn base_layer(&mut self, window_size: Vector<f32>) -> RenderLayout<Self> {
        let mut tree = TaffyTree::new();
        let header_node = tree
            .new_leaf_with_context(
                Style {
                    padding: Rect::length(20.0),
                    size: Size {
                        width: length(window_size.x),
                        height: length(100.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    bg_color: COLOR_LIGHT,
                    border: Border {
                        radius: BorderRadius {
                            bottom_left: 40.0,
                            bottom_right: 40.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let header_text = tree
            .new_leaf_with_context(
                Style {
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Flygande bäckasiner söka hwila på mjuka tuvor".into(),
                        font_size: 18,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        tree.add_child(header_node, header_text).unwrap();

        let root = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(window_size.x),
                        height: length(window_size.y),
                    },
                    gap: Size {
                        width: length(16.0),
                        height: length(16.0),
                    },
                    ..Default::default()
                },
                &[header_node],
            )
            .unwrap();
        RenderLayout {
            tree,
            root,
            desired_size: Size::MAX_CONTENT,
            root_pos: Vector::zero(),
            anchor: Anchor::TopLeft,
        }
    }
}

impl AppState for App {
    fn generate_layout(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        if self.perf_overlay.visible {
            vec![
                self.base_layer(window_size),
                self.perf_overlay.generate_layout(window_size),
            ]
        } else {
            vec![self.base_layer(window_size)]
        }
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
