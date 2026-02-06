use std::{cell::RefCell, fmt, marker::PhantomData, rc::Weak, sync::Arc};

use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        COLOR_DANGER,
        graph::Interpolation,
        renderer::{AppState, NodeContext, Renderer, flags, visual_log},
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
    pub graph_data: Weak<RefCell<Vec<Vec<Vector<f32>>>>>,
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
            .field("graph_data", &self.graph_data.upgrade().unwrap_or_default())
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
            graph_data: Weak::new(),
        }
    }
}

impl<T> GraphWidgetData<T>
where
    T: AppState,
{
    /// Convert screen coordinates to data domain coordinates using the provided bounding box
    pub fn screen_to_data(&self, screen_pos: Vector<f32>, abs_bbox: Rect<f32>) -> Vector<f32> {
        let x = self.limits.x0.x
            + (screen_pos.x - abs_bbox.x0.x) / abs_bbox.width()
                * (self.limits.x1.x - self.limits.x0.x);
        let y = self.limits.x0.y
            + (screen_pos.y - abs_bbox.x0.y) / abs_bbox.height()
                * (self.limits.x1.y - self.limits.x0.y);
        Vector::new(x, y)
    }
}

impl<T> UiData<T> for GraphWidgetData<T>
where
    T: AppState + 'static,
{
    fn custom_render(
        &self,
        _id: &NodeId,
        _ctx: &NodeContext<T>,
        layout: &taffy::Layout,
        renderer: &mut Renderer<T>,
        bbox: Rect<f32>,
    ) {
        if let Some(rc) = self.graph_data.upgrade() {
            // TODO: Loop over traces
            let points = (*rc).borrow();
            if !points.is_empty() {
                renderer.graph_r.bind_graph(
                    &points[0],
                    self.limits,
                    Interpolation::Linear,
                    layout.size.into(),
                    0,
                );
            }
        } else {
            renderer.graph_r.bind_graph(
                &[],
                Rect::from_points(Vector::new(0.0, -1.0), Vector::new(1.0, 1.0)),
                Interpolation::Linear,
                layout.size.into(),
                0,
            );
        }
        renderer
            .graph_r
            .draw(0, bbox, COLOR_DANGER, COLOR_DANGER, 1.0);
    }
}

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
        let pdata: &mut GraphWidgetData<T> = guard.downcast_mut().unwrap();
        visual_log("", format!("{:#?}", pdata));

        let (style, mut context) = parse_style::<T>(style);
        context.flags |= flags::GRAPH;
        context.persistent_id = Some(id.clone());
        pdata.graph_data = data;

        // We need to get the node id first since it's needed to find the layout of the graph in the event listeners
        let node_id = self
            .tree
            .borrow_mut()
            .new_leaf_with_context(style, context)
            .unwrap();

        let node_id1 = node_id;
        self.mutate_context(node_id, move |ctx| {
            let id1 = id.clone();
            ctx.on_middle_mouse_down = Some(Arc::new(move |state| {
                let bbox = state.get_node_bbox(node_id1).unwrap_or(Rect::from_points(
                    Vector::new(0.0, 0.0),
                    Vector::new(1.0, 1.0),
                ));
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    let data_pos = w_state.screen_to_data(state.mouse_pos, bbox);
                    w_state.interaction = GraphInteraction::Panning {
                        pan_start: data_pos,
                        mouse_pos: data_pos,
                    };
                });
            }));
            let id1 = id.clone();
            ctx.on_middle_mouse_up = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    match w_state.interaction {
                        GraphInteraction::None => {}
                        GraphInteraction::Panning {
                            pan_start,
                            mouse_pos,
                        } => {
                            let delta = mouse_pos - pan_start;
                            w_state.limits.x0.x -= delta.x;
                            w_state.limits.x1.x -= delta.x;
                            // Flip Y: dragging down (positive delta) should move limits up (positive)
                            w_state.limits.x0.y += delta.y;
                            w_state.limits.x1.y += delta.y;
                        }
                        GraphInteraction::BoxZooming => todo!("Box zooming is not implemented yet"),
                    }
                    w_state.interaction = GraphInteraction::None;
                });
            }));
            let id1 = id.clone();
            ctx.on_mouse_move = Some(Arc::new(move |state| {
                let bbox = state.get_node_bbox(node_id1).unwrap_or(Rect::from_points(
                    Vector::new(0.0, 0.0),
                    Vector::new(1.0, 1.0),
                ));
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    match w_state.interaction {
                        GraphInteraction::None => {}
                        GraphInteraction::Panning {
                            pan_start,
                            mouse_pos: _,
                        } => {
                            let data_pos = w_state.screen_to_data(state.mouse_pos, bbox);
                            w_state.interaction = GraphInteraction::Panning {
                                pan_start,
                                mouse_pos: data_pos,
                            };
                        }
                        GraphInteraction::BoxZooming => todo!("Box zooming is not implemented yet"),
                    }
                });
            }));
        });

        node_id
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
