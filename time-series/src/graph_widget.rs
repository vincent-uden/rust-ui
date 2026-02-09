use std::{cell::RefCell, fmt, marker::PhantomData, rc::Weak, sync::Arc};

use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        COLOR_DANGER, COLOR_LIGHT, COLOR_PRIMARY, COLOR_SECONDARY, Text, TextAlignment,
        graph::Interpolation,
        renderer::{AppState, NodeContext, Renderer, flags, visual_log},
        widgets::{DefaultAtom, UiBuilder, UiData},
    },
    style::parse_style,
};
use strum::EnumString;
use taffy::{NodeId, Size};
use tracing::debug;

#[derive(Debug, Copy, Clone, EnumString, Default)]
pub enum GraphInteraction {
    #[default]
    None,
    Panning {
        pan_start_screen: Vector<f32>,
        start_limits: Rect<f32>,
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
    pub x_ticks: i32,
    pub y_ticks: i32,
    pub mouse_pos: Option<Vector<f32>>,
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
            .field("graph_data", &"[..]")
            .field("x_ticks", &self.x_ticks)
            .field("y_ticks", &self.y_ticks)
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
            x_ticks: 7,
            y_ticks: 7,
            mouse_pos: None,
        }
    }
}

impl<T> GraphWidgetData<T>
where
    T: AppState,
{
    /// Converts a difference between two positions in screen space to a a difference in data space.
    /// It only scales the vector *without* translating it, thus it *can't* be used on an absolute position.
    /// Used for panning where we need to translate screen movement to data movement.
    pub fn screen_delta_to_data_delta(
        &self,
        screen_delta: Vector<f32>,
        abs_bbox: Rect<f32>,
    ) -> Vector<f32> {
        let data_width = self.limits.x1.x - self.limits.x0.x;
        let data_height = self.limits.x1.y - self.limits.x0.y;
        let dx = screen_delta.x / abs_bbox.width() * data_width;
        // Flip Y: screen Y increases downward, data Y increases upward
        let dy = -screen_delta.y / abs_bbox.height() * data_height;
        Vector::new(dx, dy)
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
        let _span = tracy_client::span!("Graph widget render");
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
        renderer.graph_r.draw(0, bbox, COLOR_PRIMARY, 1.0);

        // Axes
        for i in 0..self.y_ticks {
            let dy = bbox.height() / ((self.y_ticks - 1) as f32);
            let y = (i as f32) * dy + bbox.x0.y;
            renderer.line_r.draw(
                Vector::new(bbox.x0.x - 10.0, y),
                Vector::new(bbox.x0.x, y),
                COLOR_PRIMARY,
                2.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );
            let dy_data = self.limits.height() / ((self.y_ticks - 1) as f32);
            let y_data = self.limits.x0.y + (i as f32) * dy_data;

            renderer.text_r.draw_on_line(
                Text::new(format!("{}", y_data), 12, COLOR_PRIMARY).aligned(TextAlignment::Right),
                Vector::new(bbox.x0.x - 110.0, y - 8.0),
                Size {
                    height: 12.0,
                    width: 90.0,
                },
                None,
            );
        }
        for i in 0..self.x_ticks {
            let dx = bbox.width() / ((self.x_ticks - 1) as f32);
            let x = (i as f32) * dx + bbox.x0.x;
            renderer.line_r.draw(
                Vector::new(x, bbox.x1.y),
                Vector::new(x, bbox.x1.y + 10.0),
                COLOR_PRIMARY,
                2.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );

            let dx_data = self.limits.width() / ((self.x_ticks - 1) as f32);
            let x_data = self.limits.x0.x + (i as f32) * dx_data;
            renderer.text_r.draw_on_line(
                Text::new(format!("{}", x_data), 12, COLOR_PRIMARY).aligned(TextAlignment::Center),
                Vector::new(x - 45.0, bbox.x1.y + 12.0),
                Size {
                    height: 12.0,
                    width: 90.0,
                },
                None,
            );
        }
        renderer.line_r.draw(
            Vector::new(bbox.x0.x, bbox.x0.y),
            Vector::new(bbox.x0.x, bbox.x1.y),
            COLOR_PRIMARY,
            2.0,
            Vector::new(renderer.width as f32, renderer.height as f32),
        );
        renderer.line_r.draw(
            Vector::new(bbox.x0.x, bbox.x1.y),
            Vector::new(bbox.x1.x, bbox.x1.y),
            COLOR_PRIMARY,
            2.0,
            Vector::new(renderer.width as f32, renderer.height as f32),
        );

        // Tooltip
        if let Some(mouse_pos) = self.mouse_pos
            && matches!(self.interaction, GraphInteraction::None)
        {
            renderer.line_r.draw(
                Vector::new(mouse_pos.x, bbox.x0.y),
                Vector::new(mouse_pos.x, bbox.x1.y),
                COLOR_SECONDARY,
                1.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );
            renderer.line_r.draw(
                Vector::new(bbox.x0.x, mouse_pos.y),
                Vector::new(bbox.x1.x, mouse_pos.y),
                COLOR_SECONDARY,
                1.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );
        }
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
    /// Exists just to provide the space necessary to render the axes. Doesn't actually do anything
    fn y_axis(&self, style: &str) -> NodeId;
    /// Exists just to provide the space necessary to render the axes. Doesn't actually do anything
    fn x_axis(&self, style: &str) -> NodeId;
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
        let _span = tracy_client::span!("graph_time_series");
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), GraphWidgetData::<T>::default()),
        };
        let mut guard = binding.data.lock().unwrap();
        let pdata: &mut GraphWidgetData<T> = guard.downcast_mut().unwrap();

        let (style, mut context) = parse_style::<T>(style);
        context.flags |= flags::GRAPH;
        context.persistent_id = Some(id.clone());
        pdata.graph_data = data;

        // We need to get the node id before adding the event listeners since it's needed to find the layout of the graph in the event listeners
        let node_id = self
            .tree
            .borrow_mut()
            .new_leaf_with_context(style, context)
            .unwrap();

        self.mutate_context(node_id, move |ctx| {
            let id1 = id.clone();
            ctx.on_middle_mouse_down = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    w_state.interaction = GraphInteraction::Panning {
                        pan_start_screen: state.mouse_pos,
                        start_limits: w_state.limits,
                    };
                });
            }));
            let id1 = id.clone();
            ctx.on_middle_mouse_up = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    w_state.interaction = GraphInteraction::None;
                });
            }));
            let node_id3 = node_id;
            let id1 = id.clone();
            ctx.on_mouse_move = Some(Arc::new(move |state| {
                let bbox = state.get_node_bbox(node_id3).unwrap_or(Rect::from_points(
                    Vector::new(0.0, 0.0),
                    Vector::new(1.0, 1.0),
                ));
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    w_state.mouse_pos = Some(state.mouse_pos);
                    match w_state.interaction {
                        GraphInteraction::None => {}
                        GraphInteraction::Panning {
                            pan_start_screen,
                            start_limits: original_limits,
                        } => {
                            let screen_delta = state.mouse_pos - pan_start_screen;
                            let data_delta = w_state.screen_delta_to_data_delta(screen_delta, bbox);
                            w_state.limits.x0.x = original_limits.x0.x - data_delta.x;
                            w_state.limits.x1.x = original_limits.x1.x - data_delta.x;
                            w_state.limits.x0.y = original_limits.x0.y - data_delta.y;
                            w_state.limits.x1.y = original_limits.x1.y - data_delta.y;
                            w_state.interaction = GraphInteraction::Panning {
                                pan_start_screen,
                                start_limits: original_limits,
                            };
                        }
                        GraphInteraction::BoxZooming => todo!("Box zooming is not implemented yet"),
                    }
                });
            }));
            let id1 = id.clone();
            ctx.on_scroll = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    let width = w_state.limits.width();
                    w_state.limits.x0.x += width * 0.1 * state.scroll_delta.y.signum();
                    w_state.limits.x1.x -= width * 0.1 * state.scroll_delta.y.signum();
                });
            }));
        });

        node_id
    }

    fn y_axis(&self, _style: &str) -> NodeId {
        self.div("w-100", &[])
    }

    fn x_axis(&self, _style: &str) -> NodeId {
        self.div("pl-100 h-40 w-full", &[self.div("h-20 w-full", &[])])
    }
}

// Graph widget roadmap
// - [/] Shader for rendering
// - [x] Pan/zoom control
// - [x] Less points than pixels
// - [x] Clip data that is out of bounds
// - [x] More points than pixels
// - [x] Ticks
// - [x] Axis labels
// - [ ] Show cursor xy-coordinates in data domain
// - [ ] Legend
