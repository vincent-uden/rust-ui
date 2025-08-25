use std::{collections::HashMap, path::PathBuf, sync::Arc};

use glfw::{Action, Key, Modifiers, Scancode};
use tracing::{debug, info};

use crate::{
    FRAME_TIME,
    geometry::Vector,
    render::{
        Border, BorderRadius, COLOR_DANGER, COLOR_LIGHT, COLOR_PRIMARY, COLOR_SUCCESS, Color, Text,
        rect::RectRenderer,
        text::{TextRenderer, total_size},
    },
    shader::Shader,
};
use taffy::{prelude::*, print_tree};

type Flag = u8;

mod flags {
    use crate::state::Flag;

    pub const TEXT: Flag = 0b00000001;
}

#[derive(Default)]
pub struct NodeContext {
    flags: Flag,
    // Colors
    bg_color: Color,
    // Border
    border: Border,
    // Text
    text: Text,
    // Event listeners
    on_mouse_enter: Option<Arc<dyn Fn(&mut State)>>,
    on_mouse_exit: Option<Arc<dyn Fn(&mut State)>>,
    on_mouse_down: Option<Arc<dyn Fn(&mut State)>>,
    on_mouse_up: Option<Arc<dyn Fn(&mut State)>>,
}

#[derive(Default)]
struct PerfStats {
    visible: bool,
    avg_sleep_ms: f64,
    ram_usage: u64,
}

pub struct State {
    pub width: u32,
    pub height: u32,
    pub mouse_left_down: bool,
    pub mouse_left_was_down: bool,
    pub mouse_pos: Vector<f32>,
    pub last_mouse_pos: Vector<f32>,
    pub rect_r: RectRenderer,
    pub text_r: TextRenderer,
    pending_event_listeners: Vec<Arc<dyn Fn(&mut State)>>,
    header_bg: Color,
    hover_states: HashMap<NodeId, bool>,
    perf_stats: PerfStats,
}

impl State {
    pub fn new(rect_shader: Shader, text_shader: Shader) -> Self {
        Self {
            width: 1000,
            height: 800,
            mouse_left_down: false,
            mouse_left_was_down: false,
            mouse_pos: Vector::zero(),
            last_mouse_pos: Vector::zero(),
            rect_r: RectRenderer::new(rect_shader),
            text_r: TextRenderer::new(
                text_shader,
                &PathBuf::from("./assets/fonts/LiberationMono.ttf"),
            )
            .unwrap(),
            pending_event_listeners: vec![],
            header_bg: COLOR_LIGHT,
            hover_states: HashMap::new(),
            perf_stats: PerfStats::default(),
        }
    }

    fn run_event_listeners(&mut self) {
        while let Some(el) = self.pending_event_listeners.pop() {
            (*el)(self)
        }
    }

    pub fn update(&mut self, avg_sleep_ms: f64, ram_usage: u64) {
        self.perf_stats.avg_sleep_ms = avg_sleep_ms;
        self.perf_stats.ram_usage = ram_usage;
        self.run_event_listeners();
    }

    pub fn handle_key(
        &mut self,
        key: Key,
        _scancode: Scancode,
        action: Action,
        _modifiers: Modifiers,
    ) {
        match key {
            Key::F12 => match action {
                Action::Release => {
                    self.perf_stats.visible = !self.perf_stats.visible;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn stats_overlay(
        &mut self,
        size: crate::geometry::Vector<f32>,
    ) -> (TaffyTree<NodeContext>, NodeId) {
        let mut tree = TaffyTree::new();

        let title = tree
            .new_leaf_with_context(
                Style {
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Performance stats".into(),
                        font_size: 18,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let frame_time = tree
            .new_leaf_with_context(
                Style::default(),
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: format!(
                            "Frame time: {:.2} ms",
                            FRAME_TIME.as_millis() as f64 - self.perf_stats.avg_sleep_ms
                        ),
                        font_size: 14,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let ram_usage = tree
            .new_leaf_with_context(
                Style::default(),
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: format!("RAM: {:.2} MB", self.perf_stats.ram_usage / 1_000_000,),
                        font_size: 14,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let root_node = tree
            .new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: Dimension::percent(1.0),
                        height: Dimension::percent(1.0),
                    },
                    gap: Size {
                        width: length(0.0),
                        height: length(8.0),
                    },
                    max_size: size.into(),
                    padding: Rect::length(12.0),
                    ..Default::default()
                },
                NodeContext {
                    bg_color: Color::new(0.0, 0.0, 0.0, 0.5),
                    ..Default::default()
                },
            )
            .unwrap();

        tree.add_child(root_node, title).unwrap();
        tree.add_child(root_node, frame_time).unwrap();
        tree.add_child(root_node, ram_usage).unwrap();

        tree.compute_layout_with_measure(
            root_node,
            Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MinContent,
            },
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
        (tree, root_node)
    }

    fn generate_layout(&mut self) -> (TaffyTree<NodeContext>, NodeId) {
        let mut tree = TaffyTree::new();
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
                    text: Text {
                        text: "Hello from taffy! gggggg lask jwal aj wkja ljw klaj w".into(),
                        font_size: 18,
                        ..Default::default()
                    },
                    bg_color: self.header_bg,
                    border: Border {
                        radius: BorderRadius {
                            bottom_left: 40.0,
                            bottom_right: 40.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    on_mouse_enter: Some(Arc::new(|state| {
                        info!("Entering");
                        state.header_bg = COLOR_DANGER;
                    })),
                    on_mouse_exit: Some(Arc::new(|state| {
                        info!("Exiting");
                        state.header_bg = COLOR_LIGHT;
                    })),
                    on_mouse_down: Some(Arc::new(|_| {
                        info!("Mouse down");
                    })),
                    on_mouse_up: Some(Arc::new(|_| {
                        info!("Mouse up");
                    })),
                    ..Default::default()
                },
            )
            .unwrap();

        let body_node = tree
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: length(self.width as f32),
                        height: auto(),
                    },
                    border: Rect {
                        left: length(40.0),
                        right: length(40.0),
                        top: length(40.0),
                        bottom: length(40.0),
                    },
                    flex_grow: 1.0,
                    ..Default::default()
                },
                NodeContext {
                    bg_color: COLOR_SUCCESS,
                    border: Border {
                        thickness: 20.0,
                        radius: BorderRadius::all(40.0),
                        color: COLOR_LIGHT,
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let root_node = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(self.width as f32),
                        height: length(self.height as f32),
                    },
                    gap: Size {
                        width: length(16.0),
                        height: length(16.0),
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
        (tree, root_node)
    }

    pub fn draw_and_render(&mut self) {
        let (tree, root_node) = self.generate_layout();
        self.render_tree(&tree, root_node, Vector::zero());
        if self.perf_stats.visible {
            let (tree, root_node) = self.stats_overlay(Vector::new(400.0, 400.0));
            let stats_layout = tree.layout(root_node).unwrap();
            self.render_tree(
                &tree,
                root_node,
                Vector::new(
                    self.width as f32 - stats_layout.size.width,
                    self.height as f32 - stats_layout.size.height,
                ),
            );
        }
    }

    /// Draws a populated layout tree to the screen and queues up event listeners for the drawn
    /// nodes.
    fn render_tree(
        &mut self,
        tree: &TaffyTree<NodeContext>,
        root_node: NodeId,
        position: Vector<f32>,
    ) {
        let mut stack: Vec<(NodeId, taffy::Point<f32>)> = vec![(root_node, position.into())];
        while let Some((id, parent_pos)) = stack.pop() {
            let layout = tree.layout(id).unwrap();
            let default_ctx = &NodeContext::default();
            let ctx = tree.get_node_context(id).unwrap_or(&default_ctx);

            // Drawing
            let abs_pos = layout.location + parent_pos;
            self.rect_r.draw(
                crate::geometry::Rect {
                    x0: Vector::new(abs_pos.x, abs_pos.y),
                    x1: Vector::new(
                        abs_pos.x + layout.size.width,
                        abs_pos.y + layout.size.height,
                    ),
                },
                ctx.bg_color,
                ctx.border,
                1.0,
            );

            if ctx.flags & flags::TEXT == 1 {
                self.text_r.draw_in_box(
                    ctx.text.clone(),
                    Vector::new(abs_pos.x, abs_pos.y),
                    layout.size,
                );
            }

            // Event listeners
            let abs_bbox = crate::geometry::Rect {
                x0: abs_pos.into(),
                x1: Into::<Vector<f32>>::into(abs_pos) + layout.size.into(),
            };
            if let Some(on_mouse_enter) = &ctx.on_mouse_enter
                && abs_bbox.contains(self.mouse_pos)
                && !*self.hover_states.get(&id).unwrap_or(&false)
            {
                self.pending_event_listeners.push(on_mouse_enter.clone());
            }
            if let Some(on_mouse_exit) = &ctx.on_mouse_exit
                && !abs_bbox.contains(self.mouse_pos)
                && *self.hover_states.get(&id).unwrap_or(&false)
            {
                self.pending_event_listeners.push(on_mouse_exit.clone());
            }
            if let Some(on_mouse_down) = &ctx.on_mouse_down
                && abs_bbox.contains(self.mouse_pos)
                && self.mouse_left_down
                && !self.mouse_left_was_down
            {
                self.pending_event_listeners.push(on_mouse_down.clone());
            }
            if let Some(on_mouse_up) = &ctx.on_mouse_up
                && abs_bbox.contains(self.mouse_pos)
                && !self.mouse_left_down
                && self.mouse_left_was_down
            {
                self.pending_event_listeners.push(on_mouse_up.clone());
            }
            if ctx.on_mouse_enter.is_some() || ctx.on_mouse_exit.is_some() {
                if abs_bbox.contains(self.mouse_pos) {
                    self.hover_states.insert(id, true);
                } else {
                    self.hover_states.insert(id, false);
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
            let lines = text_renderer.layout_text(
                available_space,
                ctx.text.text.clone(),
                ctx.text.font_size,
            );
            let size = total_size(&lines);
            return Size {
                width: size.x,
                height: size.y,
            };
        }
    }

    return Size::ZERO;
}
