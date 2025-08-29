use std::{collections::HashMap, path::PathBuf, sync::Arc};

use glfw::{Action, Key, Modifiers, Scancode};

use crate::{
    geometry::Vector,
    render::{
        Border, Color, Text,
        line::LineRenderer,
        mesh::MeshRenderer,
        rect::RectRenderer,
        text::{TextRenderer, total_size},
    },
    shader::Shader,
};
use taffy::prelude::*;

type Flag = u8;

pub mod flags {
    use super::Flag;
    /// Enables text drawing in a node
    pub const TEXT: Flag = 0b00000001;
}

pub type EventListener<T> = Arc<dyn Fn(&mut Renderer<T>)>;

/// Contains relevant information for a UI node in addition to the sizing and position information
/// stored in [taffy::TaffyTree].
#[derive(Default)]
pub struct NodeContext<T>
where
    T: AppState + std::default::Default,
{
    pub flags: Flag,
    // Colors
    pub bg_color: Color,
    // Border
    pub border: Border,
    pub text: Text,
    // Event listeners
    pub on_mouse_enter: Option<EventListener<T>>,
    pub on_mouse_exit: Option<EventListener<T>>,
    pub on_mouse_down: Option<EventListener<T>>,
    pub on_mouse_up: Option<EventListener<T>>,
}

/// Renders a [taffy::TaffyTree] and handles event listeners associated with UI nodes.
pub struct Renderer<T>
where
    T: AppState + std::default::Default,
{
    /// The window width
    pub width: u32,
    /// The window height
    pub height: u32,
    /// Is the mouse left mouse button currently pressed down
    pub mouse_left_down: bool,
    /// Was the mouse left mouse button pressed down last frame
    pub mouse_left_was_down: bool,
    /// The current mouse position
    pub mouse_pos: Vector<f32>,
    /// The mouse position last frame
    pub last_mouse_pos: Vector<f32>,
    pub rect_r: RectRenderer,
    pub text_r: TextRenderer,
    pub line_r: LineRenderer,
    /// Event listeners which have been triggered and are waiting to be called
    pending_event_listeners: Vec<EventListener<T>>,
    hover_states: HashMap<NodeId, bool>,
    /// The application state to be provided by a consumer of the library
    pub app_state: T,
}

impl<T> Renderer<T>
where
    T: AppState + std::default::Default,
{
    pub fn new(
        rect_shader: Shader,
        text_shader: Shader,
        line_shader: Shader,
        initial_state: T,
    ) -> Self {
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
            line_r: LineRenderer::new(line_shader),
            pending_event_listeners: vec![],
            hover_states: HashMap::new(),
            app_state: initial_state,
        }
    }

    /// Runs all the triggered but not yet called event listeners
    fn run_event_listeners(&mut self) {
        while let Some(el) = self.pending_event_listeners.pop() {
            (*el)(self)
        }
    }

    /// Should be called on every frame, before the application states update method *(if it has
    /// any)*
    pub fn update(&mut self) {
        self.run_event_listeners();
    }

    /// Passes key presses to the application state
    pub fn handle_key(
        &mut self,
        key: Key,
        scancode: Scancode,
        action: Action,
        modifiers: Modifiers,
    ) {
        self.app_state.handle_key(key, scancode, action, modifiers);
    }

    /// Fetches a layout tree for each layer from the application state, draws them to the screen
    /// and checks if any event listeners should run (calls [Renderer::render_tree]).
    pub fn compute_layout_and_render(&mut self) {
        let window_size = Vector::new(self.width as f32, self.height as f32);
        let mut layers = self.app_state.generate_layout(window_size);

        for layer in layers.iter_mut() {
            layer
                .tree
                .compute_layout_with_measure(
                    layer.root,
                    layer.desired_size,
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
            let size: Vector<f32> = layer.tree.layout(layer.root).unwrap().size.into();
            let pos = match layer.anchor {
                Anchor::TopLeft => layer.root_pos,
                Anchor::TopRight => {
                    Vector::new(window_size.x - layer.root_pos.x - size.x, layer.root_pos.y)
                }
                Anchor::BottomLeft => {
                    Vector::new(layer.root_pos.x, window_size.y - layer.root_pos.y - size.y)
                }
                Anchor::BottomRight => window_size - layer.root_pos - size,
                Anchor::Center => (window_size - size).scaled(0.5) + layer.root_pos,
            };
            self.render_tree(&layer.tree, layer.root, pos);
        }
    }

    /// Draws a populated layout tree to the screen and queues up event listeners for the drawn
    /// nodes.
    fn render_tree(
        &mut self,
        tree: &TaffyTree<NodeContext<T>>,
        root_node: NodeId,
        position: Vector<f32>,
    ) {
        let mut stack: Vec<(NodeId, taffy::Point<f32>)> = vec![(root_node, position.into())];
        while let Some((id, parent_pos)) = stack.pop() {
            let layout = tree.layout(id).unwrap();
            let default_ctx = &NodeContext::default();
            let ctx = tree.get_node_context(id).unwrap_or(default_ctx);

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

/// Helps taffy decide how big nodes containing text need to be.
pub fn measure_function<T>(
    known_dimensions: taffy::geometry::Size<Option<f32>>,
    available_space: taffy::geometry::Size<taffy::style::AvailableSpace>,
    node_context: Option<&mut NodeContext<T>>,
    text_renderer: &mut TextRenderer,
) -> Size<f32>
where
    T: AppState + std::default::Default,
{
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    if let Some(ctx) = node_context
        && ctx.flags & flags::TEXT == 1
    {
        let lines =
            text_renderer.layout_text(available_space, ctx.text.text.clone(), ctx.text.font_size);
        total_size(&lines).into()
    } else {
        Size::ZERO
    }
}

/// Chooses a corner of the window or its center as the origin for a layer. Any offset provided
/// from the anchor will be towards the middle of the screen, or towards the bottom right corner if
/// anchored to the center.
#[derive(Default)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

pub struct RenderLayout<T>
where
    T: AppState + Default,
{
    pub tree: TaffyTree<NodeContext<T>>,
    pub root: NodeId,
    pub desired_size: Size<AvailableSpace>,
    pub root_pos: Vector<f32>,
    pub anchor: Anchor,
    pub scissor: bool,
}

impl<T> Default for RenderLayout<T>
where
    T: AppState + Default,
{
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            root: NodeId::new(u64::MAX),
            desired_size: Size::MAX_CONTENT,
            root_pos: Vector::zero(),
            anchor: Anchor::default(),
            scissor: false,
        }
    }
}

pub trait AppState: Default {
    fn generate_layout(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>>;
    fn handle_key(&mut self, key: Key, scancode: Scancode, action: Action, modifiers: Modifiers) {}
}
