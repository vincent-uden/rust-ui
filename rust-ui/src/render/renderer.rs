use std::{
    cell::{LazyCell, RefCell},
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use dashmap::DashMap;
use glfw::{Action, Key, Modifiers, MouseButton, Scancode};
use tracing::{debug, error, info};

use crate::{
    geometry::Vector,
    render::{
        Border, COLOR_LIGHT, Color, Text,
        line::LineRenderer,
        rect::RectRenderer,
        sprite::{SpriteKey, SpriteRenderer},
        text::{TextRenderer, total_size},
    },
    style::parse_style,
};
use taffy::{prelude::*, print_tree};

type Flag = u8;

#[cfg_attr(any(), rustfmt::skip)]
pub mod flags {
    use super::Flag;
    /// Enables text drawing in a node
    pub const TEXT: Flag                 = 0b00000001;
    pub const HOVER_BG: Flag             = 0b00000010;
    pub const EXPLICIT_TEXT_LAYOUT: Flag = 0b00000100;
    pub const SPRITE: Flag               = 0b00001000;
    /// Changes how the limits of offsets work to act as a scoll bar
    pub const SCROLL_BAR: Flag           = 0b00010000;
    /// Changes how the limits of offsets work to act as content being scrolled
    pub const SCROLL_CONTENT: Flag       = 0b00100000;
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
    pub bg_color_hover: Color,
    // Border
    pub border: Border,
    pub text: Text,
    pub sprite_key: String,
    pub offset: Vector<f32>,
    // Event listeners
    pub on_mouse_enter: Option<EventListener<T>>,
    pub on_mouse_exit: Option<EventListener<T>>,
    pub on_mouse_down: Option<EventListener<T>>,
    pub on_mouse_up: Option<EventListener<T>>,
    // Clipping
    pub scissor: bool,
}

#[derive(Default)]
pub struct Listeners<T>
where
    T: AppState + std::default::Default,
{
    pub on_mouse_enter: Option<EventListener<T>>,
    pub on_mouse_exit: Option<EventListener<T>>,
    pub on_mouse_down: Option<EventListener<T>>,
    pub on_mouse_up: Option<EventListener<T>>,
}

impl SpriteKey for String {}

static DEBUG_MAP: std::sync::LazyLock<DashMap<String, String>> =
    std::sync::LazyLock::new(|| DashMap::new());

/// Inserts a logging message to be rendered in the visual debugging overlay
pub fn visual_log(key: String, message: String) {
    DEBUG_MAP.insert(key, message);
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
    pub sprite_r: SpriteRenderer<String>,
    /// Event listeners which have been triggered and are waiting to be called
    pending_event_listeners: Vec<EventListener<T>>,
    hover_states: HashMap<NodeId, bool>,
    /// The application state to be provided by a consumer of the library
    pub app_state: T,
    /// Wether or not the debug layer should be shown. This is drawn on top of everything drawn by
    /// the app layer
    pub show_debug_layer: bool,
    pub debug_position: Vector<f32>,
    pub debug_size: Vector<f32>,
    pub debug_scroll: f32,
}

impl<T> Renderer<T>
where
    T: AppState + std::default::Default,
{
    pub fn new(
        rect_renderer: RectRenderer,
        text_renderer: TextRenderer,
        line_renderer: LineRenderer,
        sprite_renderer: SpriteRenderer<String>,
        initial_state: T,
    ) -> Self {
        Self {
            width: 1000,
            height: 800,
            mouse_left_down: false,
            mouse_left_was_down: false,
            mouse_pos: Vector::zero(),
            last_mouse_pos: Vector::zero(),
            rect_r: rect_renderer,
            text_r: text_renderer,
            line_r: line_renderer,
            sprite_r: sprite_renderer,
            pending_event_listeners: vec![],
            hover_states: HashMap::new(),
            app_state: initial_state,
            show_debug_layer: true,
            debug_position: Vector::zero(),
            debug_size: Vector::new(100.0, 200.0),
            debug_scroll: 0.0,
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

    /// Passes mouse button events to the application state
    pub fn handle_mouse_button(
        &mut self,
        button: MouseButton,
        action: Action,
        modifiers: Modifiers,
    ) {
        self.mouse_left_down = action == glfw::Action::Press || action == glfw::Action::Repeat;
        self.app_state
            .handle_mouse_button(button, action, modifiers);
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
        self.app_state.handle_mouse_scroll(scroll_delta);
    }

    /// Fetches a layout tree for each layer from the application state, draws them to the screen
    /// and checks if any event listeners should run (calls [Renderer::render_tree]).
    pub fn compute_layout_and_render(&mut self) {
        let window_size = Vector::new(self.width as f32, self.height as f32);
        let mut layers = self.app_state.generate_layout(window_size);

        if self.show_debug_layer {
            layers.push(self.debug_layer());
        }

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

            if layer.scissor {
                self.enable_scissor_for_layer(pos, size);
            }

            let _ = self.dfs(&layer.tree, layer.root, pos);

            if layer.scissor {
                self.disable_scissor();
            }
        }
    }

    fn dfs(
        &mut self, 
        tree: &TaffyTree<NodeContext<T>>, 
        root_node: NodeId, 
        position: Vector<f32>
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
                if let Some((_, scissor)) = trail.iter().rev().find(|(_, scissor)| scissor.is_some()) {
                    let scissor = scissor.unwrap();
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
                // TODO: This isn't quite right
                //       Scroll content needs to be scissored to its parent which is currently not
                //       possible due to having no memory of parent scissor state. It's always
                //       reset
                if let Some(parent_bbox) = tree.parent(id).map(|pid| tree.layout(pid).unwrap()) {
                    if layout.size.height > parent_bbox.size.height {
                        abs_pos.y -= lerp(
                            0.0,
                            layout.size.height - parent_bbox.size.height,
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

            if ctx.flags & flags::TEXT != 0 {
                if ctx.flags & flags::EXPLICIT_TEXT_LAYOUT == 0 {
                    self.text_r.draw_in_box(
                        ctx.text.clone(),
                        Vector::new(
                            abs_pos.x + layout.padding.left,
                            abs_pos.y + layout.padding.top,
                        ),
                        Size {
                            width: layout.size.width - layout.padding.left - layout.padding.right,
                            height: layout.size.height - layout.padding.top - layout.padding.bottom,
                        },
                    );
                } else {
                    debug!("{:?} {:?}", ctx.text, trail);
                    self.text_r.draw_in_box_explicit(
                        ctx.text.clone(),
                        Vector::new(
                            abs_pos.x + layout.padding.left,
                            abs_pos.y + layout.padding.top,
                        ),
                        Size {
                            width: layout.size.width - layout.padding.left - layout.padding.right,
                            height: layout.size.height - layout.padding.top - layout.padding.bottom,
                        },
                    );
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
        // Goal:
        // Container (floating position)
        //   Header (flex-row)
        //     Label
        //     Open/Close
        //   Body (flex-col, scrollable)
        //     Block
        //       BlockHeader (flex-row)
        //         Label
        //         Delete
        //         Open/Close
        //       BlockData
        //         Text
        //   Footer
        //     Size-drag
        let tree: taffy::TaffyTree<NodeContext<T>> = taffy::TaffyTree::new();
        let tree = RefCell::new(tree);

        let b = UiBuilder::new(&tree);
        #[cfg_attr(any(), rustfmt::skip)]
        let root = b.div("rounded-8 bg-black opacity-40 w-full h-full p-8 flex-col", &[
            b.ui("flex-row border-2 border-black", Listeners::default(), &[
                b.text("grow",
                    Text::new("Debug".into(), 18, COLOR_LIGHT),
                    &[],
                ),
                b.ui("", Listeners::default(), &[]), // TODO: Icons?
            ]),
            b.scrollable("", self.debug_scroll, Arc::new(|_, _| {}), &[
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
                b.text("", Text::new("Hola".into(), 18, COLOR_LIGHT), &[]),
            ]),
            b.div( "flex-row",
                &[
                b.text("grow",
                    Text::new("Debug".into(), 18, COLOR_LIGHT),
                    &[],
                ),
                    b.ui( "", Listeners::default(), &[])
                ],// TODO: Icons?
            ),
        ]);

        let data = unsafe { std::ptr::read(tree.as_ptr()) };
        std::mem::forget(tree);
        RenderLayout {
            tree: data,
            root,
            desired_size: Size {
                width: AvailableSpace::Definite(self.debug_size.x),
                height: AvailableSpace::Definite(self.debug_size.y),
            },
            root_pos: self.debug_position.into(),
            anchor: Anchor::TopRight,
            scissor: true,
        }
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
        if ctx.flags & flags::EXPLICIT_TEXT_LAYOUT == 0 {
            let lines = text_renderer.layout_text(
                available_space,
                ctx.text.text.clone(),
                ctx.text.font_size,
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
    fn handle_key(
        &mut self,
        _key: Key,
        _scancode: Scancode,
        _action: Action,
        _modifiers: Modifiers,
    ) {
    }
    fn handle_mouse_button(
        &mut self,
        _button: MouseButton,
        _action: Action,
        _modifiers: Modifiers,
    ) {
    }
    fn handle_mouse_position(&mut self, _position: Vector<f32>, _delta: Vector<f32>) {}
    fn handle_mouse_scroll(&mut self, _scroll_delta: Vector<f32>) {}
}

struct UiBuilder<'a, T>
where
    T: AppState + Default,
{
    tree: &'a RefCell<TaffyTree<NodeContext<T>>>,
}

impl<'a, T> UiBuilder<'a, T>
where
    T: AppState + Default,
{
    pub fn new(tree: &'a RefCell<TaffyTree<NodeContext<T>>>) -> Self {
        Self { tree }
    }

    pub fn ui(&self, style: &str, listeners: Listeners<T>, children: &[NodeId]) -> NodeId {
        let (style, mut context) = parse_style(style);
        context.on_mouse_exit = listeners.on_mouse_exit;
        context.on_mouse_enter = listeners.on_mouse_enter;
        context.on_mouse_up = listeners.on_mouse_up;
        context.on_mouse_down = listeners.on_mouse_down;
        let mut tree = self.tree.borrow_mut();
        let parent = tree.new_leaf_with_context(style, context).unwrap();
        for child in children {
            tree.add_child(parent, *child).unwrap();
        }
        return parent;
    }

    fn div(&self, style: &str, children: &[NodeId]) -> NodeId {
        let (style, context) = parse_style(style);
        let mut tree = self.tree.borrow_mut();
        let parent = tree.new_leaf_with_context(style, context).unwrap();
        for child in children {
            tree.add_child(parent, *child).unwrap();
        }
        return parent;
    }

    fn text(&self, style: &str, text: Text, children: &[NodeId]) -> NodeId {
        let (style, mut context) = parse_style(style);
        context.text = text;
        context.flags |= flags::TEXT;
        let mut tree = self.tree.borrow_mut();
        let parent = tree.new_leaf_with_context(style, context).unwrap();
        for child in children {
            tree.add_child(parent, *child).unwrap();
        }
        return parent;
    }

    /// Scroll height is in percent. Thus if items are added, scrolling is preserved
    fn scrollable(
        &self,
        style: &str,
        scroll_height: f32,
        update_scroll: Arc<dyn Fn(&mut Renderer<T>, f32)>,
        children: &[NodeId],
    ) -> NodeId {
        let scrollbar = {
            let mut tree = self.tree.borrow_mut();
            let (stl, mut ctx) = parse_style("w-full bg-red-800 hover:bg-red-900 h-32 scroll-bar");
            ctx.offset.y = scroll_height;
            tree.new_leaf_with_context(stl, ctx).unwrap()
        };
        let scroll_content = {
            let mut tree = self.tree.borrow_mut();
            let (stl, mut ctx) = parse_style("flex-col scroll-content");
            ctx.offset.y = scroll_height;
            let parent = tree.new_leaf_with_context(stl, ctx).unwrap();
            for child in children {
                tree.add_child(parent, *child).unwrap();
            }
            parent
        };

        // I am not 100% sure why the overflow-clip has to be this far out, but it works here
        #[cfg_attr(any(), rustfmt::skip)]
        self.ui(&format!("{} flex-row border-2 border-white overflow-clip", style), Listeners::default(), &[
            self.ui("grow bg-sky-500 border-2 border-red-500", Listeners {
                on_mouse_up: Some(Arc::new(|state| {
                    state.debug_scroll += 0.1;
                    info!(state.debug_scroll);
                })), 
                ..Default::default()
            }, &[scroll_content]),
            self.div("w-8 bg-red-500", &[scrollbar]),
        ])
    }
}

pub fn lerp(start: f32, end: f32, normalized: f32) -> f32 {
    start + normalized * (end - start)
}

#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::render::renderer::{AppState, NodeContext};

    #[derive(Default)]
    struct DummyState {}

    impl AppState for DummyState {
        fn generate_layout(
            &mut self,
            _: crate::geometry::Vector<f32>,
        ) -> Vec<crate::render::renderer::RenderLayout<Self>> {
            todo!()
        }
    }

    #[test]
    pub fn ui_shorthand_doesnt_deadlock() {
        let tree: taffy::TaffyTree<NodeContext<DummyState>> = taffy::TaffyTree::new();
        let tree = RefCell::new(tree);
        let b = UiBuilder::new(&tree);
        b.ui("", Listeners::default(), &[b.div("", &[]), b.div("", &[])]);
    }
}
