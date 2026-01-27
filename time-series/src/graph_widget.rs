use std::{cell::RefCell, fmt, marker::PhantomData, rc::Weak, sync::Arc};

use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        renderer::{AppState, flags, visual_log},
        widgets::{DefaultAtom, UiBuilder, UiData},
    },
    style::parse_style,
};
use strum::EnumString;
use taffy::NodeId;

#[derive(Debug, Copy, Clone, EnumString, Default)]
pub enum GraphInteraction {
    #[default]
    None,
    Panning {
        pan_start: Vector<f32>,
        mouse_pos: Vector<f32>,
    },
    BoxZooming,
}

#[derive(Clone)]
pub struct GraphWidgetData<T>
where
    T: AppState,
{
    phantom: PhantomData<T>,
    pub interaction: GraphInteraction,
    /// Graph limits, currently in data space
    pub limits: Rect<f32>,
}
impl<T> fmt::Debug for GraphWidgetData<T>
where
    T: AppState,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphWidgetData")
            .field("phantom", &self.phantom)
            .field("interaction", &self.interaction)
            .field("limits", &self.limits)
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
            interaction: Default::default(),
            limits: Default::default(),
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
        visual_log("", format!("{:#?}", _state));

        let (style, mut context) = parse_style::<T>(style);
        context.flags |= flags::GRAPH;
        context.graph_data = data;
        let id1 = id.clone();
        context.on_middle_mouse_down = Some(Arc::new(move |state| {
            state.ui_builder.mutate_state(&id1, |w_state| {
                let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                // TODO: Convert to data domain somehow
                w_state.interaction = GraphInteraction::Panning {
                    pan_start: state.mouse_pos,
                    mouse_pos: state.mouse_pos,
                };
            });
        }));
        let id1 = id.clone();
        context.on_middle_mouse_up = Some(Arc::new(move |state| {
            state.ui_builder.mutate_state(&id1, |w_state| {
                let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                match w_state.interaction {
                    GraphInteraction::None => {}
                    GraphInteraction::Panning {
                        pan_start,
                        mouse_pos,
                    } => {
                        // TODO: Convert to data domain somehow
                        w_state.limits.x0 += mouse_pos - pan_start;
                        w_state.limits.x1 += mouse_pos - pan_start;
                    }
                    GraphInteraction::BoxZooming => todo!("Box zooming is not implemented yet"),
                }
                w_state.interaction = GraphInteraction::None;
            });
        }));
        let id1 = id.clone();
        context.on_mouse_move = Some(Arc::new(move |state| {
            state.ui_builder.mutate_state(&id1, |w_state| {
                let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                match w_state.interaction {
                    GraphInteraction::None => {}
                    GraphInteraction::Panning {
                        pan_start,
                        mouse_pos: _,
                    } => {
                        w_state.interaction = GraphInteraction::Panning {
                            pan_start,
                            mouse_pos: state.mouse_pos,
                        };
                    }
                    GraphInteraction::BoxZooming => todo!("Box zooming is not implemented yet"),
                }
            });
        }));

        self.tree
            .borrow_mut()
            .new_leaf_with_context(style, context)
            .unwrap()
    }
}

// Graph widget roadmap
// - [/] Shader for rendering
// - [ ] Pan/zoom control
// - [/] Less points than pixels
// - [ ] More points than pixels
// - [ ] Ticks
// - [ ] Axis labels
// - [ ] Show cursor xy-coordinates in data domain
// - [ ] Legend
