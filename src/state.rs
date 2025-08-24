use tracing::info;

use crate::{
    geometry::Vector,
    render::{Border, BorderRadius, COLOR_LIGHT, Color, rect::RectRenderer, text::TextRenderer},
};
use taffy::{CacheTree, prelude::*, print_tree};

type Flag = u8;

mod flags {
    use crate::state::Flag;

    pub const TEXT: Flag = 0b00000001;
}

#[derive(Default)]
pub struct NodeContext {
    flags: Flag,
    bg_color: Color,
    text: String,
    font_size: u32,
    on_hover: Option<Box<dyn Fn(&mut State)>>,
    on_click: Option<Box<dyn Fn(&mut State)>>,
}

pub struct State {
    pub width: u32,
    pub height: u32,
    pub mouse_left_down: bool,
    pub mouse_left_was_down: bool,
    pub rect_r: RectRenderer,
    pub text_r: TextRenderer,
}

impl State {
    pub fn draw_and_render(&mut self) {
        let mut tree: TaffyTree<NodeContext> = TaffyTree::new();

        let header_node = tree
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: length(self.width as f32),
                        height: length(100.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: "Hello from taffy!".into(),
                    font_size: 18,
                    bg_color: COLOR_LIGHT,
                    ..Default::default()
                },
            )
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
        tree.compute_layout_with_measure(
            root_node,
            Size::MAX_CONTENT,
            |known_dimensions, available_space, _node_id, node_context, _style| {
                measure_function(
                    known_dimensions,
                    available_space,
                    node_context,
                    &mut self.text_r,
                )
            },
        )
        .unwrap();

        let mut stack: Vec<(NodeId, taffy::Point<f32>)> = vec![(root_node, taffy::Point::zero())];
        while let Some((id, parent_pos)) = stack.pop() {
            let layout = tree.layout(id).unwrap();
            let context = tree.get_node_context(id);

            let abs_pos = layout.location + parent_pos;
            self.rect_r.draw(
                crate::geometry::Rect {
                    x0: Vector::new(abs_pos.x, abs_pos.y),
                    x1: Vector::new(
                        abs_pos.x + layout.size.width,
                        abs_pos.y + layout.size.height,
                    ),
                },
                context
                    .map(|c| c.bg_color)
                    .unwrap_or(Color::new(0.0, 0.0, 0.0, 0.0)),
                Color::new(1.0, 0.0, 0.0, 1.0),
                Border {
                    thickness: 2.0,
                    radius: BorderRadius::all(0.0),
                },
                1.0,
            );

            if let Some(ctx) = context {
                if ctx.flags & flags::TEXT == 1 {
                    self.text_r.draw_text(
                        &ctx.text,
                        Vector::new(abs_pos.x, abs_pos.y),
                        ctx.font_size,
                        1.0,
                        Color::new(0.0, 0.0, 0.0, 1.0),
                    );
                }
            }

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

fn measure_function(
    known_dimensions: taffy::geometry::Size<Option<f32>>,
    available_space: taffy::geometry::Size<taffy::style::AvailableSpace>,
    node_context: Option<&mut NodeContext>,
    text_renderer: &mut TextRenderer,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    if let Some(ctx) = node_context {
        if ctx.flags & flags::TEXT == 1 {
            let size = text_renderer.measure_text_size(&ctx.text, ctx.font_size);
            return Size {
                width: size.x,
                height: size.y,
            };
        }
    }

    return Size::ZERO;
}
