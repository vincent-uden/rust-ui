use core::fmt;
use std::{
    any::Any,
    borrow::Borrow,
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Weak,
    sync::{Arc, Mutex},
};

use dashmap::DashMap;
use glfw::{Action, Key, Modifiers, MouseButton, Scancode};
use smol_str::SmolStr;
use string_cache::DefaultAtom;
use tracing::debug;

use crate::{
    geometry::{Rect, Vector},
    render::{
        Border, COLOR_DANGER, COLOR_LIGHT, Color, Text,
        graph::{GraphRenderer, Interpolation},
        line::LineRenderer,
        rect::RectRenderer,
        sprite::{SpriteKey, SpriteRenderer},
        text::{TextRenderer, total_size},
        widgets::{UiBuilder, scrollable::ScrollableBuilder},
    },
    style::parse_style,
};
use taffy::prelude::*;

type Flag = u16;

#[cfg_attr(any(), rustfmt::skip)]
pub mod flags {
    use super::Flag;
    /// Enables text drawing in a node
    pub const TEXT: Flag                 = 1 << 0;
    pub const HOVER_BG: Flag             = 1 << 1;
    pub const EXPLICIT_TEXT_LAYOUT: Flag = 1 << 2;
    pub const SPRITE: Flag               = 1 << 3;
    /// Changes how the limits of offsets work to act as a scoll bar
    pub const SCROLL_BAR: Flag           = 1 << 4;
    /// Changes how the limits of offsets work to act as content being scrolled
    pub const SCROLL_CONTENT: Flag       = 1 << 5;
    /// Should text scroll to keep the cursor position visible
    pub const TEXT_SCROLL: Flag          = 1 << 6;
    /// Force text onto a single line (no wrapping). Not compatible with [EXPLICIT_TEXT_LAYOUT]
    pub const TEXT_SINGLE_LINE: Flag     = 1 << 7;
    /// Should the rectangle be used to render a (line) graph
    pub const GRAPH: Flag                = 1 << 8;
}

// TODO: Investigate if this can be changed to an FnOnce somehow
pub type EventListener<T> = Arc<dyn Fn(&mut Renderer<T>)>;

/// Contains relevant information for a UI node in addition to the sizing and position information
/// stored in [taffy::TaffyTree].
pub struct NodeContext<T>
where
    T: AppState,
{
    pub flags: Flag,
    pub bg_color: Color,
    pub bg_color_hover: Color,
    pub border: Border,
    pub text: Text,
    pub sprite_key: T::SpriteKey,
    pub offset: Vector<f32>,
    // Event listeners
    pub on_scroll: Option<EventListener<T>>,
    pub on_mouse_enter: Option<EventListener<T>>,
    pub on_mouse_exit: Option<EventListener<T>>,
    pub on_left_mouse_down: Option<EventListener<T>>,
    pub on_left_mouse_up: Option<EventListener<T>>,
    pub on_right_mouse_down: Option<EventListener<T>>,
    pub on_right_mouse_up: Option<EventListener<T>>,
    pub on_middle_mouse_down: Option<EventListener<T>>,
    pub on_middle_mouse_up: Option<EventListener<T>>,
    // Clipping
    pub scissor: bool,
    // Persistent state
    pub persistent_id: Option<DefaultAtom>,
    pub cursor_idx: Option<usize>,
    // Move somewhere else? Can we simplify this in any way?
    pub graph_data: Weak<RefCell<Vec<Vec<Vector<f32>>>>>,
}

impl<T> Default for NodeContext<T>
where
    T: AppState,
{
    fn default() -> Self {
        Self {
            flags: Default::default(),
            bg_color: Default::default(),
            bg_color_hover: Default::default(),
            border: Default::default(),
            text: Default::default(),
            sprite_key: Default::default(),
            offset: Default::default(),
            on_scroll: Default::default(),
            on_mouse_enter: Default::default(),
            on_mouse_exit: Default::default(),
            on_left_mouse_down: Default::default(),
            on_left_mouse_up: Default::default(),
            on_right_mouse_down: Default::default(),
            on_right_mouse_up: Default::default(),
            on_middle_mouse_down: Default::default(),
            on_middle_mouse_up: Default::default(),
            scissor: Default::default(),
            persistent_id: Default::default(),
            cursor_idx: Default::default(),
            graph_data: Default::default(),
        }
    }
}

impl<T> NodeContext<T>
where
    T: AppState,
{
    pub fn set_listeners(&mut self, listeners: Listeners<T>) {
        self.on_scroll = listeners.on_scroll;
        self.on_mouse_exit = listeners.on_mouse_exit;
        self.on_mouse_enter = listeners.on_mouse_enter;
        self.on_left_mouse_up = listeners.on_left_mouse_up;
        self.on_left_mouse_down = listeners.on_left_mouse_down;
        self.on_right_mouse_up = listeners.on_right_mouse_up;
        self.on_right_mouse_down = listeners.on_right_mouse_down;
        self.on_middle_mouse_up = listeners.on_middle_mouse_up;
        self.on_middle_mouse_down = listeners.on_middle_mouse_down;
    }
}

pub struct Listeners<T>
where
    T: AppState,
{
    pub on_scroll: Option<EventListener<T>>,
    pub on_mouse_enter: Option<EventListener<T>>,
    pub on_mouse_exit: Option<EventListener<T>>,
    pub on_left_mouse_down: Option<EventListener<T>>,
    pub on_left_mouse_up: Option<EventListener<T>>,
    pub on_right_mouse_down: Option<EventListener<T>>,
    pub on_right_mouse_up: Option<EventListener<T>>,
    pub on_middle_mouse_down: Option<EventListener<T>>,
    pub on_middle_mouse_up: Option<EventListener<T>>,
}

impl<T> Default for Listeners<T>
where
    T: AppState,
{
    fn default() -> Self {
        Self {
            on_scroll: Default::default(),
            on_mouse_enter: Default::default(),
            on_mouse_exit: Default::default(),
            on_left_mouse_down: Default::default(),
            on_left_mouse_up: Default::default(),
            on_right_mouse_down: Default::default(),
            on_right_mouse_up: Default::default(),
            on_middle_mouse_down: Default::default(),
            on_middle_mouse_up: Default::default(),
        }
    }
}

impl SpriteKey for String {}

static DEBUG_MAP: std::sync::LazyLock<DashMap<DefaultAtom, String>> =
    std::sync::LazyLock::new(|| DashMap::new());

/// Inserts a logging message to be rendered in the visual debugging overlay
pub fn visual_log(key: &str, message: String) {
    DEBUG_MAP.insert(DefaultAtom::from(key), message);
}

#[derive(Debug)]
pub enum MouseDragState {
    /// Contains the point at which the mouse was pressed
    Pressed(Vector<f32>),
    Released,
}

/// Renders a [taffy::TaffyTree] and handles event listeners associated with UI nodes.
pub struct Renderer<T>
where
    T: AppState,
{
    /// The current frame number
    pub frame: usize,
    /// The window width
    pub width: u32,
    /// The window height
    pub height: u32,
    /// Is the mouse left mouse button currently pressed down
    pub mouse_left_down: bool,
    /// Is the mouse right mouse button currently pressed down
    pub mouse_right_down: bool,
    /// Is the mouse middle mouse button currently pressed down
    pub mouse_middle_down: bool,
    /// Was the mouse left mouse button pressed down last frame
    pub mouse_left_was_down: bool,
    /// Was the mouse right mouse button pressed down last frame
    pub mouse_right_was_down: bool,
    /// Was the mouse middle mouse button pressed down last frame
    pub mouse_middle_was_down: bool,
    /// The current mouse position
    pub mouse_pos: Vector<f32>,
    /// The current scroll wheel movement since last frame
    pub scroll_delta: Vector<f32>,
    /// The mouse position last frame
    pub last_mouse_pos: Vector<f32>,
    pub rect_r: RectRenderer,
    pub text_r: TextRenderer,
    pub line_r: LineRenderer,
    pub sprite_r: SpriteRenderer<T::SpriteKey>,
    pub graph_r: GraphRenderer,
    /// Event listeners which have been triggered and are waiting to be called
    pending_event_listeners: Vec<EventListener<T>>,
    hover_states: HashMap<NodeId, bool>,
    /// The application state to be provided by a consumer of the library
    pub app_state: T,
    // --- Debug state
    /// Wether or not the debug layer should be shown. This is drawn on top of everything drawn by
    /// the app layer
    pub show_debug_layer: bool,
    pub debug_position: Vector<f32>,
    pub debug_size: Vector<f32>,
    pub debug_cached_size: Vector<f32>,
    pub debug_drag: MouseDragState,
    pub debug_expanded: bool,
    layers: Arc<Vec<RenderLayout<T>>>,
    // --- Event capture
    /// Has the mouse hit any ui elements in a layer? This is the layer in which it happened
    pub mouse_hit_layer: i32,
    /// The UI builder used for constructing layouts and managing widget state
    pub ui_builder: UiBuilder<T>,
}

impl<T> Renderer<T>
where
    T: AppState,
{
    pub fn new(
        rect_renderer: RectRenderer,
        text_renderer: TextRenderer,
        line_renderer: LineRenderer,
        sprite_renderer: SpriteRenderer<T::SpriteKey>,
        graph_renderer: GraphRenderer,
        initial_state: T,
    ) -> Self {
        Self {
            frame: 0,
            width: 1000,
            height: 800,
            mouse_left_down: false,
            mouse_left_was_down: false,
            mouse_right_down: false,
            mouse_right_was_down: false,
            mouse_middle_down: false,
            mouse_middle_was_down: false,
            mouse_pos: Vector::zero(),
            scroll_delta: Vector::zero(),
            last_mouse_pos: Vector::zero(),
            rect_r: rect_renderer,
            text_r: text_renderer,
            line_r: line_renderer,
            sprite_r: sprite_renderer,
            graph_r: graph_renderer,
            pending_event_listeners: vec![],
            hover_states: HashMap::new(),
            app_state: initial_state,
            show_debug_layer: false,
            debug_position: Vector::zero(),
            debug_size: Vector::new(400.0, 200.0),
            debug_cached_size: Vector::new(400.0, 200.0),
            debug_drag: MouseDragState::Released,
            debug_expanded: true,
            layers: Arc::new(vec![]),
            mouse_hit_layer: -1,
            ui_builder: UiBuilder::new(),
        }
    }

    fn enable_scissor_for_layer(&self, root_pos: Vector<f32>, size: Vector<f32>) {
        let opengl_y = self.height as f32 - root_pos.y - size.y;
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(
                root_pos.x as i32,
                opengl_y as i32,
                size.x as i32,
                size.y as i32,
            );
        }
    }

    fn disable_scissor(&self) {
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }

    /// Runs all the triggered but not yet called event listeners
    fn run_event_listeners(&mut self) {
        while let Some(el) = self.pending_event_listeners.pop() {
            (*el)(self)
        }
    }

    /// Should be called on every frame, before input handling
    pub fn pre_update(&mut self) {
        self.mouse_left_was_down = self.mouse_left_down;
        self.mouse_right_was_down = self.mouse_right_down;
        self.mouse_middle_was_down = self.mouse_middle_down;
        self.scroll_delta.x = 0.0;
        self.scroll_delta.y = 0.0;
    }

    /// Should be called on every frame, before the application states update method *(if it has
    /// any)*
    pub fn update(&mut self) {
        let _span = tracy_client::span!("App update");
        self.frame += 1;
        self.ui_builder.update(self.frame);
        self.mouse_hit_layer = -1;

        match self.debug_drag {
            MouseDragState::Pressed(pressed_at) => {
                let delta = pressed_at - self.mouse_pos;
                self.debug_size.x = self.debug_cached_size.x + delta.x;
                self.debug_size.y = self.debug_cached_size.y - delta.y;
            }
            _ => {}
        }

        self.app_state.update();
        self.compute_layout();
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
        self.app_state
            .handle_key(key, scancode, action, modifiers, &self.ui_builder);
    }

    /// Passes character input to application state
    pub fn handle_char(&mut self, unicode: u32) {
        self.app_state.handle_char(unicode, &self.ui_builder);
    }

    /// Passes mouse button events to the application state
    pub fn handle_mouse_button(
        &mut self,
        button: MouseButton,
        action: Action,
        modifiers: Modifiers,
    ) {
        match button {
            MouseButton::Button1 => {
                self.mouse_left_down =
                    action == glfw::Action::Press || action == glfw::Action::Repeat;
                self.app_state
                    .handle_mouse_button(button, action, modifiers, &self.ui_builder);
            }
            MouseButton::Button2 => {
                self.mouse_right_down =
                    action == glfw::Action::Press || action == glfw::Action::Repeat;
                self.app_state
                    .handle_mouse_button(button, action, modifiers, &self.ui_builder);
            }
            MouseButton::Button3 => {
                self.mouse_middle_down =
                    action == glfw::Action::Press || action == glfw::Action::Repeat;
                self.app_state
                    .handle_mouse_button(button, action, modifiers, &self.ui_builder);
            }
            _ => {}
        }
    }

    /// Passes mouse position changes to the application state
    pub fn handle_mouse_position(&mut self, position: Vector<f32>) {
        let delta = position - self.last_mouse_pos;
        self.last_mouse_pos = self.mouse_pos;
        self.mouse_pos = position;
        self.app_state.handle_mouse_position(position, delta);
    }

    /// Passes mouse scroll events to the application state
    pub fn handle_mouse_scroll(&mut self, scroll_delta: Vector<f32>) {
        self.scroll_delta = scroll_delta;
        self.app_state.handle_mouse_scroll(scroll_delta);
    }

    fn compute_layout(&mut self) {
        let window_size = Vector::new(self.width as f32, self.height as f32);
        let mut layers = self
            .app_state
            .generate_layout(window_size, &self.ui_builder);

        if self.show_debug_layer {
            layers.push(self.debug_layer());
        }

        for (i, layer) in layers.iter_mut().enumerate().rev() {
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
            let _ = self.collect_event_listeners(&layer.tree, layer.root, pos, i as i32);
        }
        self.layers = layers.into();
    }

    /// Fetches a layout tree for each layer from the application state, draws them to the screen
    /// and checks if any event listeners should run (calls [Renderer::render]).
    pub fn render(&mut self) {
        let window_size = Vector::new(self.width as f32, self.height as f32);
        let layers = self.layers.clone();
        for layer in layers.iter() {
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

            if layer.scissor {
                self.enable_scissor_for_layer(pos, size);
            }

            let _ = self.render_tree(&layer.tree, layer.root, pos);

            if layer.scissor {
                self.disable_scissor();
            }
        }
    }

    fn collect_event_listeners(
        &mut self,
        tree: &TaffyTree<NodeContext<T>>,
        root_node: NodeId,
        position: Vector<f32>,
        layer_idx: i32,
    ) -> taffy::TaffyResult<()> {
        let mut to_render: Vec<(NodeId, taffy::Point<f32>)> = vec![(root_node, position.into())];
        let mut trail: Vec<(NodeId, Option<crate::geometry::Rect<f32>>)> = vec![];
        let mut current_scissor: Option<crate::geometry::Rect<f32>>;
        while let Some((id, parent_pos)) = to_render.pop() {
            let layout = tree.layout(id)?;
            let default_ctx = &NodeContext::default();
            let ctx = tree.get_node_context(id).unwrap_or(default_ctx);

            let mut abs_pos = layout.location + parent_pos;

            if ctx.flags & flags::SCROLL_BAR != 0 {
                if let Some(parent_bbox) = tree.parent(id).map(|pid| tree.layout(pid).unwrap()) {
                    abs_pos.y += lerp(
                        0.0,
                        parent_bbox.size.height - layout.size.height,
                        ctx.offset.y,
                    );
                }
            } else if ctx.flags & flags::SCROLL_CONTENT != 0 {
                if let Some(Ok(parent_bbox)) = tree.parent(id).map(|pid| tree.layout(pid)) {
                    if layout.content_size.height > parent_bbox.size.height {
                        abs_pos.y -= lerp(
                            0.0,
                            layout.content_size.height - parent_bbox.size.height,
                            ctx.offset.y,
                        );
                    }
                }
            } else {
                abs_pos = abs_pos + ctx.offset.into();
            }

            // Traverse trail to find parent
            while trail.last().is_some() && tree.parent(id) != trail.last().map(|x| x.0) {
                trail.pop();
            }
            // Set current_scissor
            let node_rect = crate::geometry::Rect {
                x0: Into::<Vector<f32>>::into(abs_pos),
                x1: Into::<Vector<f32>>::into(abs_pos) + layout.size.into(),
            };
            if ctx.scissor {
                current_scissor = Some(node_rect);
                trail.push((id, Some(node_rect)));
            } else {
                current_scissor = trail.iter().rev().find_map(|(_, s)| *s);
                trail.push((id, None));
            }

            let mouse_in_scissor = current_scissor.map_or(true, |r| r.contains(self.mouse_pos));
            if mouse_in_scissor {
                let abs_bbox = crate::geometry::Rect {
                    x0: abs_pos.into(),
                    x1: Into::<Vector<f32>>::into(abs_pos) + layout.size.into(),
                };
                if abs_bbox.contains(self.mouse_pos) {
                    if let Some(on_mouse_down) = &ctx.on_left_mouse_down
                        && self.mouse_left_down
                        && !self.mouse_left_was_down
                        && layer_idx >= self.mouse_hit_layer
                    {
                        self.pending_event_listeners.push(on_mouse_down.clone());
                        self.mouse_hit_layer = layer_idx;
                    }
                    if let Some(on_mouse_up) = &ctx.on_left_mouse_up
                        && !self.mouse_left_down
                        && self.mouse_left_was_down
                    {
                        self.pending_event_listeners.push(on_mouse_up.clone());
                    }
                    if let Some(on_mouse_down) = &ctx.on_right_mouse_down
                        && self.mouse_right_down
                        && !self.mouse_right_was_down
                        && layer_idx >= self.mouse_hit_layer
                    {
                        self.pending_event_listeners.push(on_mouse_down.clone());
                        self.mouse_hit_layer = layer_idx;
                    }
                    if let Some(on_mouse_up) = &ctx.on_right_mouse_up
                        && !self.mouse_right_down
                        && self.mouse_right_was_down
                    {
                        self.pending_event_listeners.push(on_mouse_up.clone());
                    }
                    if let Some(on_mouse_down) = &ctx.on_middle_mouse_down
                        && self.mouse_middle_down
                        && !self.mouse_middle_was_down
                        && layer_idx >= self.mouse_hit_layer
                    {
                        self.pending_event_listeners.push(on_mouse_down.clone());
                        self.mouse_hit_layer = layer_idx;
                    }
                    if let Some(on_mouse_up) = &ctx.on_middle_mouse_up
                        && !self.mouse_middle_down
                        && self.mouse_middle_was_down
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
                    if let Some(on_scroll) = &ctx.on_scroll
                        && (self.scroll_delta.x.abs() > 0.01 || self.scroll_delta.y.abs() > 0.01)
                        && abs_bbox.contains(self.mouse_pos)
                        && layer_idx >= self.mouse_hit_layer
                    {
                        self.pending_event_listeners.push(on_scroll.clone());
                        self.mouse_hit_layer = layer_idx;
                    }

                    // Even if no event listener is registered, an element with text, an icon or a
                    // background colour should occlude anything behind it. Eventually all this should
                    // probably be replaced a proper rounded rectangle-drawing picker buffer similar to
                    // how sketch objects are picked.
                    if (ctx.bg_color.a != 0.0
                        || (ctx.flags & (flags::TEXT | flags::SPRITE | flags::HOVER_BG) != 0))
                        && layer_idx >= self.mouse_hit_layer
                    {
                        self.mouse_hit_layer = layer_idx;
                    }
                }

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
            }

            for child in tree.children(id)?.iter().rev() {
                to_render.push((*child, abs_pos));
            }
        }

        Ok(())
    }

    fn render_tree(
        &mut self,
        tree: &TaffyTree<NodeContext<T>>,
        root_node: NodeId,
        position: Vector<f32>,
    ) -> taffy::TaffyResult<()> {
        let mut to_render: Vec<(NodeId, taffy::Point<f32>)> = vec![(root_node, position.into())];
        // The trail acts as a scissor stack. It allows the program to restore an outer scissor
        // state before delving into the rendering of additional descendant nodes
        let mut trail: Vec<(NodeId, Option<(Vector<f32>, Vector<f32>)>)> = vec![];

        while let Some((id, parent_pos)) = to_render.pop() {
            let layout = tree.layout(id)?;
            let mut abs_pos = layout.location + parent_pos;
            let default_ctx = &NodeContext::default();
            let ctx = tree.get_node_context(id).unwrap_or(default_ctx);
            // If the last node of the trail isn't our parent, we should traverse the trail upwards
            // until we find our parent. This ensures that scissoring is applied to all children of
            // a node, while not affecting any other nodes in other places of the tree.
            while trail.last().is_some() && tree.parent(id) != trail.last().map(|x| x.0) {
                trail.pop();
            }
            self.disable_scissor();
            // After ensuring that the last node is my parent, find the nearest scissor region if
            // there are any
            if ctx.scissor {
                trail.push((id, Some((abs_pos.into(), layout.size.into()))));
                self.enable_scissor_for_layer(abs_pos.into(), layout.size.into());
            } else {
                if let Some((_, Some(scissor))) =
                    trail.iter().rev().find(|(_, scissor)| scissor.is_some())
                {
                    self.enable_scissor_for_layer(scissor.0, scissor.1);
                }
                trail.push((id, None));
            }

            if ctx.flags & flags::SCROLL_BAR != 0 {
                if let Some(parent_bbox) = tree.parent(id).map(|pid| tree.layout(pid).unwrap()) {
                    abs_pos.y += lerp(
                        0.0,
                        parent_bbox.size.height - layout.size.height,
                        ctx.offset.y,
                    );
                }
            } else if ctx.flags & flags::SCROLL_CONTENT != 0 {
                if let Some(Ok(parent_bbox)) = tree.parent(id).map(|pid| tree.layout(pid)) {
                    if layout.content_size.height > parent_bbox.size.height {
                        abs_pos.y -= lerp(
                            0.0,
                            layout.content_size.height - parent_bbox.size.height,
                            ctx.offset.y,
                        );
                    }
                }
            } else {
                abs_pos = abs_pos + ctx.offset.into();
            }
            let bbox = crate::geometry::Rect {
                x0: Vector::new(abs_pos.x, abs_pos.y),
                x1: Vector::new(
                    abs_pos.x + layout.size.width,
                    abs_pos.y + layout.size.height,
                ),
            };
            let bg_color = if (ctx.flags & flags::HOVER_BG != 0) && bbox.contains(self.mouse_pos) {
                ctx.bg_color_hover
            } else {
                ctx.bg_color
            };

            if ctx.scissor {
                self.enable_scissor_for_layer(abs_pos.into(), layout.size.into());
            }

            // Drawing
            self.rect_r.draw(bbox, bg_color, ctx.border, 1.0);
            if ctx.flags & flags::GRAPH != 0 {
                if let Some(rc) = ctx.graph_data.upgrade() {
                    let points = (*rc).borrow();
                    // TODO: Loop over traces
                    self.graph_r.bind_graph(
                        &points[0],
                        Rect::from_points(Vector::new(0.0, -1.0), Vector::new(1.0, 1.0)),
                        Interpolation::Linear,
                        layout.size.into(),
                        0,
                    );
                } else {
                    self.graph_r.bind_graph(
                        &[],
                        Rect::from_points(Vector::new(0.0, -1.0), Vector::new(1.0, 1.0)),
                        Interpolation::Linear,
                        layout.size.into(),
                        0,
                    );
                }
                self.graph_r.draw(0, bbox, COLOR_DANGER, COLOR_DANGER, 1.0);
            }

            if ctx.flags & flags::TEXT != 0 {
                let mut text_pos = Vector::new(
                    abs_pos.x + layout.padding.left,
                    abs_pos.y + layout.padding.top,
                );
                let text_size = Size {
                    width: layout.size.width - layout.padding.left - layout.padding.right,
                    height: layout.size.height - layout.padding.top - layout.padding.bottom,
                };
                if ctx.flags & flags::TEXT_SCROLL != 0 {
                    if let Some(Ok(parent_layout)) = tree.parent(id).map(|pid| tree.layout(pid)) {
                        self.enable_scissor_for_layer(parent_pos.into(), parent_layout.size.into());
                        if let Some(cursor_idx) = ctx.cursor_idx {
                            let cursor_pos = self.text_r.cursor_pos(
                                &ctx.text.text,
                                Vector::zero(),
                                ctx.text.font_size,
                                cursor_idx,
                            );
                            if cursor_pos.x > parent_layout.size.width {
                                text_pos.x += parent_layout.size.width - cursor_pos.x - 10.0;
                            }
                        }
                    }
                }
                if ctx.flags & flags::EXPLICIT_TEXT_LAYOUT != 0 {
                    self.text_r.draw_in_box_explicit(
                        ctx.text.clone(),
                        text_pos,
                        text_size,
                        ctx.cursor_idx,
                    );
                } else {
                    if ctx.flags & flags::TEXT_SINGLE_LINE != 0 {
                        self.text_r.draw_on_line(
                            ctx.text.clone(),
                            text_pos,
                            text_size,
                            ctx.cursor_idx,
                        );
                    } else {
                        self.text_r.draw_in_box(
                            ctx.text.clone(),
                            text_pos,
                            text_size,
                            ctx.cursor_idx,
                        );
                    }
                }
            }
            if ctx.flags & flags::SPRITE != 0 {
                self.sprite_r.draw(
                    &ctx.sprite_key,
                    crate::geometry::Rect {
                        x0: Vector::new(
                            abs_pos.x + layout.padding.left,
                            abs_pos.y + layout.padding.top,
                        ),
                        x1: Vector::new(
                            abs_pos.x - layout.padding.left + layout.size.width,
                            abs_pos.y - layout.padding.top + layout.size.height,
                        ),
                    },
                );
            }

            for child in tree.children(id)?.iter().rev() {
                to_render.push((*child, abs_pos));
            }
        }

        self.disable_scissor();

        Ok(())
    }

    pub fn window_size(&mut self, size: (i32, i32)) {
        self.width = size.0 as u32;
        self.height = size.1 as u32;
    }

    fn debug_layer(&self) -> RenderLayout<T> {
        let b = &self.ui_builder;
        let mut entries = vec![];
        for key_value in DEBUG_MAP.iter() {
            let key = key_value.key();
            let value = key_value.value();
            entries.push(b.text("", Text::new(key.as_ref().to_string(), 12, COLOR_LIGHT)));
            entries.push(b.text_explicit("", Text::new(value.clone(), 12, COLOR_LIGHT)));
        }

        let mut resize_listener = Listeners::default();
        resize_listener.on_left_mouse_down = Some(Arc::new(|state: &mut Self| {
            state.debug_drag = MouseDragState::Pressed(state.mouse_pos);
            state.debug_cached_size = state.debug_size;
        }));
        resize_listener.on_left_mouse_up = Some(Arc::new(|state| {
            state.debug_drag = MouseDragState::Released;
        }));
        let mut expand_listener = Listeners::default();
        expand_listener.on_left_mouse_up = Some(Arc::new(|state: &mut Self| {
            state.debug_expanded = !state.debug_expanded;
        }));

        let mut children = vec![
            #[cfg_attr(any(), rustfmt::skip)]
            b.ui("flex-row", Listeners::default(), &[
                b.text("grow", Text::new("Debug", 24, COLOR_LIGHT)),
                b.sprite("w-30 h-30", if self.debug_expanded { "Up" } else { "Down" }, expand_listener),
            ]),
        ];
        if self.debug_expanded {
            #[cfg_attr(any(), rustfmt::skip)]
            children.extend(&[
                b.scrollable(DefaultAtom::from("debug_scroll"), "grow", entries),
                b.div("flex-row",
                    &[
                        b.sprite("w-30 h-30", "HandleLeft", resize_listener),
                        b.div("grow", &[] as &[NodeId]),
                    ],
                ),
            ])
        }

        let root = b.div(
            "rounded-8 bg-black opacity-40 w-full h-full p-8 flex-col",
            &children,
        );

        RenderLayout {
            tree: b.tree(),
            root,
            desired_size: Size {
                width: AvailableSpace::Definite(self.debug_size.x),
                height: if self.debug_expanded {
                    AvailableSpace::Definite(self.debug_size.y)
                } else {
                    AvailableSpace::MaxContent
                },
            },
            root_pos: self.debug_position.into(),
            anchor: Anchor::TopRight,
            scissor: true,
        }
    }

    pub fn set_focus(&mut self, focus: Option<DefaultAtom>) {
        self.app_state.set_focus(focus);
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
    T: AppState,
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
        if ctx.flags & flags::EXPLICIT_TEXT_LAYOUT == 0 {
            let lines = text_renderer.layout_text(
                available_space,
                ctx.text.text.clone(),
                ctx.text.font_size,
                true,
            );
            total_size(&lines).into()
        } else {
            let lines = text_renderer.layout_text_explicit(
                available_space,
                ctx.text.text.clone(),
                ctx.text.font_size,
            );
            total_size(&lines).into()
        }
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
    T: AppState,
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
    T: AppState,
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

pub trait AppState: Sized + 'static {
    type SpriteKey: crate::render::sprite::SpriteKey;

    fn generate_layout(
        &mut self,
        window_size: Vector<f32>,
        ui: &UiBuilder<Self>,
    ) -> Vec<RenderLayout<Self>>;
    fn handle_key(
        &mut self,
        _key: Key,
        _scancode: Scancode,
        _action: Action,
        _modifiers: Modifiers,
        _ui: &UiBuilder<Self>,
    ) {
    }
    fn handle_char(&mut self, _unicode: u32, _ui: &UiBuilder<Self>) {}
    fn handle_mouse_button(
        &mut self,
        _button: MouseButton,
        _action: Action,
        _modifiers: Modifiers,
        _ui: &UiBuilder<Self>,
    ) {
    }
    fn handle_mouse_position(&mut self, _position: Vector<f32>, _delta: Vector<f32>) {}
    fn handle_mouse_scroll(&mut self, _scroll_delta: Vector<f32>) {}
    fn update(&mut self) {}
    fn set_focus(&mut self, _focus: Option<DefaultAtom>) {}
}

pub fn lerp(start: f32, end: f32, normalized: f32) -> f32 {
    start + normalized * (end - start)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::render::{renderer::AppState, widgets::UiBuilder};

    #[derive(Default)]
    struct DummyState {}

    impl AppState for DummyState {
        type SpriteKey = String;

        fn generate_layout(
            &mut self,
            _: crate::geometry::Vector<f32>,
            _ui: &UiBuilder<Self>,
        ) -> Vec<crate::render::renderer::RenderLayout<Self>> {
            todo!()
        }
    }

    #[test]
    pub fn ui_shorthand_doesnt_deadlock() {
        let b = UiBuilder::<DummyState>::new();
        b.ui("", Listeners::default(), &[b.div("", &[]), b.div("", &[])]);
    }
}
