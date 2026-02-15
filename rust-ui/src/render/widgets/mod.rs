pub use string_cache::DefaultAtom;
use tracing::error;

use std::any::Any;
use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use taffy::{Layout, NodeId, Style, TaffyTree};

use crate::geometry::Rect;
use crate::render::renderer::{
    AppState, DelayedRender, EventListener, Listeners, NodeContext, Renderer, flags,
};
use crate::render::{COLOR_LIGHT, Text};
use crate::style::parse_style;

pub mod scrollable;
pub mod text_field;

#[macro_export]
macro_rules! id {
    ($($arg:tt)*) => {
        $crate::render::widgets::DefaultAtom::from(format!($($arg)*))
    };
}

pub trait UiData<T>: fmt::Debug + Any
where
    T: AppState,
{
    fn run_event_listener(&mut self, name: &str, renderer: &mut T) {}
    /// Some widgets need really fancy custom rendering. Those special widgets that do can do that here
    fn custom_render(
        &self,
        id: &NodeId,
        ctx: &NodeContext<T>,
        layout: &Layout,
        renderer: &mut Renderer<T>,
        bbox: Rect<f32>,
    ) {
    }
}

impl<T> dyn UiData<T>
where
    T: AppState,
{
    pub fn downcast_ref<U>(&self) -> Option<&U>
    where
        U: 'static,
    {
        (self as &dyn Any).downcast_ref::<U>()
    }

    pub fn downcast_mut<U>(&mut self) -> Option<&mut U>
    where
        U: 'static,
    {
        (self as &mut dyn Any).downcast_mut::<U>()
    }
}

#[derive(Debug)]
pub struct UiState<T>
where
    T: AppState,
{
    /// The frame number on which this piece of state was last accessed
    pub last_touched: usize,
    pub data: Arc<Mutex<Box<dyn UiData<T>>>>,
}
impl<T> Clone for UiState<T>
where
    T: AppState,
{
    fn clone(&self) -> Self {
        Self {
            last_touched: self.last_touched.clone(),
            data: self.data.clone(),
        }
    }
}

pub struct UiBuilder<T>
where
    T: AppState,
{
    /// The current frame number. Must be updated by the program running the [UiBuilder]
    frame: usize,
    /// Do **NOT** push to tree directly. Use [Uibuilder::new_leaf_with_context] instead to
    /// ensure tracking of popup widgets.
    tree: RefCell<TaffyTree<NodeContext<T>>>,
    pub state: RefCell<HashMap<DefaultAtom, UiState<T>>>,
    pub delayed_ids: RefCell<Vec<NodeId>>,
}

impl<T> UiBuilder<T>
where
    T: AppState,
{
    pub fn new() -> Self {
        Self {
            frame: 0,
            tree: TaffyTree::new().into(),
            state: HashMap::new().into(),
            delayed_ids: vec![].into(),
        }
    }

    /// Wrapper for [[taffy::TaffyTree::new_leaf_with_context]] that also tracks possible popup behaviour.
    pub fn new_leaf_with_context(
        &self,
        tree: &mut TaffyTree<NodeContext<T>>,
        style: Style,
        context: NodeContext<T>,
    ) -> NodeId {
        let id = tree.new_leaf(style).unwrap();
        if let Some(_) = &context.z_index {
            let mut delayed = self.delayed_ids.borrow_mut();
            delayed.push(id);
        }
        tree.set_node_context(id, Some(context)).unwrap();
        id
    }

    pub fn ui<I, B>(&self, style: &str, listeners: Listeners<T>, children: I) -> NodeId
    where
        I: IntoIterator<Item = B>,
        B: Borrow<NodeId>,
    {
        let (style, mut context) = parse_style(style);
        context.set_listeners(listeners);
        let mut tree = self.tree.borrow_mut();
        let parent = self.new_leaf_with_context(&mut tree, style, context);
        for child in children {
            tree.add_child(parent, *child.borrow()).unwrap();
        }
        return parent;
    }

    pub fn div<I, B>(&self, style: &str, children: I) -> NodeId
    where
        I: IntoIterator<Item = B>,
        B: Borrow<NodeId>,
    {
        let (style, context) = parse_style(style);
        let mut tree = self.tree.borrow_mut();
        let parent = self.new_leaf_with_context(&mut tree, style, context);
        for child in children {
            tree.add_child(parent, *child.borrow()).unwrap();
        }
        return parent;
    }

    pub fn text(&self, style: &str, text: Text) -> NodeId {
        let (style, mut context) = parse_style(style);
        context.text = text;
        context.flags |= flags::TEXT;
        let mut tree = self.tree.borrow_mut();
        let parent = self.new_leaf_with_context(&mut tree, style, context);
        return parent;
    }

    pub fn text_explicit(&self, style: &str, text: Text) -> NodeId {
        let (style, mut context) = parse_style(style);
        context.text = text;
        context.flags |= flags::TEXT | flags::EXPLICIT_TEXT_LAYOUT;
        let mut tree = self.tree.borrow_mut();
        let parent = self.new_leaf_with_context(&mut tree, style, context);
        return parent;
    }

    pub fn text_button(&self, style: &str, text: Text, listeners: Listeners<T>) -> NodeId {
        let mut tree = self.tree.borrow_mut();

        let mut inner_ctx = NodeContext::default();
        inner_ctx.text = text;
        inner_ctx.flags |= flags::TEXT;
        let inner = self.new_leaf_with_context(&mut tree, Style::DEFAULT, inner_ctx);

        let (style, mut outer_ctx) = parse_style(style);
        outer_ctx.set_listeners(listeners);
        let outer = self.new_leaf_with_context(&mut tree, style, outer_ctx);
        tree.add_child(outer, inner).unwrap();
        return outer;
    }

    pub fn sprite(&self, style: &str, sprite_key: &str, listeners: Listeners<T>) -> NodeId {
        let (style, mut context) = parse_style(style);
        context.flags |= flags::SPRITE;
        context.sprite_key = sprite_key.into();
        context.set_listeners(listeners);
        let mut tree = self.tree.borrow_mut();
        let parent = self.new_leaf_with_context(&mut tree, style, context);
        return parent;
    }

    pub fn accessing_state(&self, id: &DefaultAtom) -> Option<UiState<T>> {
        let mut state = self.state.borrow_mut();
        if let Some(state) = state.get_mut(id) {
            state.last_touched = self.frame;
        }
        state.borrow().get(id).cloned()
    }

    pub fn mutate_state<F, R>(&self, id: &DefaultAtom, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn UiData<T>) -> R,
    {
        let mut state = self.state.borrow_mut();
        match state.get_mut(id) {
            Some(state) => {
                let mut guard = state.data.lock().unwrap();
                Some(f(guard.as_mut()))
            }
            None => {
                error!("{} not found", id);
                None
            }
        }
    }

    pub fn insert_state(&self, id: DefaultAtom, ui_state: impl UiData<T>) -> UiState<T> {
        let mut state = self.state.borrow_mut();
        state.insert(
            id.clone(),
            UiState {
                last_touched: self.frame,
                data: Arc::new(Mutex::new(Box::new(ui_state))),
            },
        );
        state[&id].clone()
    }

    pub fn borrow_tree<'a>(&'a self) -> RefMut<'a, TaffyTree<NodeContext<T>>> {
        self.tree.borrow_mut()
    }

    /// **WARNING** this erases the tree from [Self].
    pub fn tree(&self) -> taffy::TaffyTree<NodeContext<T>> {
        self.tree.replace(TaffyTree::new())
    }

    pub fn update(&mut self, frame: usize) {
        self.frame = frame;
        self.prune_state_map();
        let mut delayed = self.delayed_ids.borrow_mut();
        delayed.clear();
    }

    fn prune_state_map(&self) {
        let mut state = self.state.borrow_mut();
        state.retain(|_, v| v.last_touched >= (self.frame - 1));
    }

    pub fn mutate_context<F>(&self, id: NodeId, f: F)
    where
        F: FnOnce(&mut NodeContext<T>),
    {
        let mut tree = self.tree.borrow_mut();
        tree.get_node_context_mut(id).map(f);
    }
}
