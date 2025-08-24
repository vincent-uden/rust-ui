use tracing::info;

use crate::{
    geometry::Vector,
    render::{Border, BorderRadius, Color, rect::RectRenderer},
};
use taffy::{CacheTree, prelude::*, print_tree};

pub struct EventListeners {
    on_hover: Option<Box<dyn Fn(&mut State)>>,
    on_click: Option<Box<dyn Fn(&mut State)>>,
}

pub struct State {
    pub width: u32,
    pub height: u32,
    pub mouse_left_down: bool,
    pub mouse_left_was_down: bool,
    pub rect_r: RectRenderer,
}

impl State {
    pub fn draw_and_render(&mut self) {
        let mut tree: TaffyTree<EventListeners> = TaffyTree::new();

        let header_node = tree
            .new_leaf(Style {
                size: Size {
                    width: length(self.width as f32),
                    height: length(100.0),
                },
                ..Default::default()
            })
            .unwrap();

        let body_node = tree
            .new_leaf(Style {
                size: Size {
                    width: length(self.width as f32),
                    height: auto(),
                },
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();

        let root_node = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(self.width as f32),
                        height: length(self.height as f32),
                    },
                    ..Default::default()
                },
                &[header_node, body_node],
            )
            .unwrap();
        tree.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

        let mut stack: Vec<(NodeId, taffy::Point<f32>)> = vec![(root_node, taffy::Point::zero())];
        while let Some((id, parent_pos)) = stack.pop() {
            let layout = tree.layout(id).unwrap();

            let abs_pos = layout.location + parent_pos;
            self.rect_r.draw(
                crate::geometry::Rect {
                    x0: Vector::new(abs_pos.x, abs_pos.y),
                    x1: Vector::new(
                        abs_pos.x + layout.size.width,
                        abs_pos.y + layout.size.height,
                    ),
                },
                Color::new(0.0, 0.0, 0.0, 0.0),
                Color::new(1.0, 0.0, 0.0, 1.0),
                Border {
                    thickness: 2.0,
                    radius: BorderRadius::all(0.0),
                },
                1.0,
            );

            if let Ok(children) = tree.children(id) {
                for child in children {
                    stack.push((child, layout.location + parent_pos));
                }
            }
        }
    }

    pub fn window_size(&mut self, size: (i32, i32)) {
        self.width = size.0 as u32;
        self.height = size.1 as u32;
    }
}
