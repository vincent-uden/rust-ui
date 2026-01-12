use std::{cell::RefCell, fmt, marker::PhantomData, rc::Weak};

use rust_ui::{
    geometry::Vector,
    render::{
        renderer::{AppState, flags},
        widgets::{DefaultAtom, UiBuilder, UiData},
    },
    style::parse_style,
};
use taffy::NodeId;

#[derive(Clone)]
pub struct GraphWidgetData<T>
where
    T: AppState,
{
    phantom: PhantomData<T>,
}
impl<T> fmt::Debug for GraphWidgetData<T>
where
    T: AppState,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphWidgetData")
            .field("phantom", &self.phantom)
            .finish()
    }
}
impl<T> Default for GraphWidgetData<T>
where
    T: AppState,
{
    fn default() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

impl<T> GraphWidgetData<T> where T: AppState {}

impl<T> UiData<T> for GraphWidgetData<T> where T: AppState + 'static {}

pub trait GraphWidgetBuilder<T>
where
    T: AppState,
{
    fn graph_time_series(
        &self,
        style: &str,
        id: DefaultAtom,
        data: Weak<RefCell<Vec<Vec<Vector<f32>>>>>,
    ) -> NodeId;
}

impl<T> GraphWidgetBuilder<T> for UiBuilder<T>
where
    T: AppState,
{
    fn graph_time_series(
        &self,
        style: &str,
        id: DefaultAtom,
        data: Weak<RefCell<Vec<Vec<Vector<f32>>>>>,
    ) -> NodeId {
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), GraphWidgetData::<T>::default()),
        };
        let mut guard = binding.data.lock().unwrap();
        let _state: &mut GraphWidgetData<T> = guard.downcast_mut().unwrap();

        let (style, mut context) = parse_style::<T>(style);
        context.flags |= flags::GRAPH;
        context.graph_data = data;

        self.tree
            .borrow_mut()
            .new_leaf_with_context(style, context)
            .unwrap()
    }
}
