use std::{cell::RefCell, fmt, marker::PhantomData, rc::Weak, sync::Arc};

use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        COLOR_LIGHT, COLOR_PRIMARY, Color, Text, TextAlignment,
        graph::Interpolation,
        renderer::{AppState, NodeContext, Renderer, flags},
        widgets::{DefaultAtom, UiBuilder, UiData},
    },
    style::parse_style,
};
use strum::EnumString;
use taffy::{NodeId, Size};

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
    pub graph_data: Weak<RefCell<Vec<Vector<f32>>>>,
    pub x_ticks: i32,
    pub y_ticks: i32,
    pub mouse_pos: Option<Vector<f32>>,
    pub last_bbox: RefCell<Rect<f32>>,
    // Axis colors (set by graph_with_axes container)
    pub y_axis_tick_color: Color,
    pub y_axis_label_color: Color,
    pub x_axis_tick_color: Color,
    pub x_axis_label_color: Color,
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
            .field("last_bbox", &self.last_bbox)
            .field("y_axis_tick_color", &self.y_axis_tick_color)
            .field("y_axis_label_color", &self.y_axis_label_color)
            .field("x_axis_tick_color", &self.x_axis_tick_color)
            .field("x_axis_label_color", &self.x_axis_label_color)
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
            last_bbox: Default::default(),
            y_axis_tick_color: COLOR_PRIMARY,
            y_axis_label_color: COLOR_LIGHT,
            x_axis_tick_color: COLOR_PRIMARY,
            x_axis_label_color: COLOR_LIGHT,
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

    pub fn screen_coord_to_data_coord(&self, pos: Vector<f32>) -> Vector<f32> {
        let last_bbox = self.last_bbox.borrow();
        ((pos - last_bbox.x0).non_uniform_scaled(last_bbox.size().div_inverted()))
            .non_uniform_scaled(self.limits.size())
            + self.limits.x0
    }
}

impl<T> UiData<T> for GraphWidgetData<T>
where
    T: AppState + 'static,
{
    fn custom_render(
        &self,
        _id: &NodeId,
        ctx: &NodeContext<T>,
        layout: &taffy::Layout,
        renderer: &mut Renderer<T>,
        bbox: Rect<f32>,
    ) {
        let mut last_bbox = self.last_bbox.borrow_mut();
        *last_bbox = bbox;
        let _span = tracy_client::span!("Graph widget render");
        if let Some(rc) = self.graph_data.upgrade() {
            let points = (*rc).borrow();
            renderer.graph_r.bind_graph(
                &points,
                self.limits,
                Interpolation::Linear,
                layout.size.into(),
                0,
            );
        } else {
            renderer.graph_r.bind_graph(
                &[],
                Rect::from_points(Vector::new(0.0, -1.0), Vector::new(1.0, 1.0)),
                Interpolation::Linear,
                layout.size.into(),
                0,
            );
        }
        let graph_color = ctx.text.color;
        let tooltip_color = ctx.border.color;

        renderer.graph_r.draw(0, bbox, graph_color, 1.0);

        // Axes
        for i in 0..self.y_ticks {
            let dy = bbox.height() / ((self.y_ticks - 1) as f32);
            let y = (i as f32) * dy + bbox.x0.y;
            renderer.line_r.draw(
                Vector::new(bbox.x0.x - 10.0, y),
                Vector::new(bbox.x0.x, y),
                self.y_axis_tick_color,
                2.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );
            let dy_data = self.limits.height() / ((self.y_ticks - 1) as f32);
            let y_data = self.limits.x0.y + (i as f32) * dy_data;

            renderer.text_r.draw_on_line(
                Text::new(format!("{}", y_data), 12, self.y_axis_label_color)
                    .aligned(TextAlignment::Right),
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
                self.x_axis_tick_color,
                2.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );

            let dx_data = self.limits.width() / ((self.x_ticks - 1) as f32);
            let x_data = self.limits.x0.x + (i as f32) * dx_data;
            renderer.text_r.draw_on_line(
                Text::new(format!("{}", x_data), 12, self.x_axis_label_color)
                    .aligned(TextAlignment::Center),
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
            self.y_axis_tick_color,
            2.0,
            Vector::new(renderer.width as f32, renderer.height as f32),
        );
        renderer.line_r.draw(
            Vector::new(bbox.x0.x, bbox.x1.y),
            Vector::new(bbox.x1.x, bbox.x1.y),
            self.x_axis_tick_color,
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
                tooltip_color,
                1.0,
                Vector::new(renderer.width as f32, renderer.height as f32),
            );
            renderer.line_r.draw(
                Vector::new(bbox.x0.x, mouse_pos.y),
                Vector::new(bbox.x1.x, mouse_pos.y),
                tooltip_color,
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
    /// Creates a standalone graph widget. Should be used with graph_with_axes() container.
    /// The returned node should not be added to the tree directly - pass it to graph_with_axes().
    ///
    /// Style mappings:
    /// - `bg-*` = graph line color
    /// - `hover:bg-*` = tooltip crosshair color
    fn graph_widget(
        &self,
        style: &str,
        id: DefaultAtom,
        data: Weak<RefCell<Vec<Vector<f32>>>>,
    ) -> NodeId;

    /// Creates a y-axis spacer widget. Should be used with graph_with_axes() container.
    /// The returned node should not be added to the tree directly - pass it to graph_with_axes().
    ///
    /// Style mappings:
    /// - `border-*` = tick/axis line color
    /// - `text-*` = label text color
    fn y_axis(&self, style: &str) -> NodeId;

    /// Creates an x-axis spacer widget. Should be used with graph_with_axes() container.
    /// The returned node should not be added to the tree directly - pass it to graph_with_axes().
    ///
    /// Style mappings:
    /// - `border-*` = tick/axis line color
    /// - `text-*` = label text color
    fn x_axis(&self, style: &str) -> NodeId;

    /// Creates a container that arranges graph_widget, y_axis, and x_axis in an L-shaped layout.
    /// Extracts colors from the axis nodes and stores them in the graph's state.
    ///
    /// # Important
    /// The graph, y_axis, and x_axis nodes must be orphan nodes (not yet added to the tree).
    /// They will be reparented as children of the container.
    fn graph_with_axes(
        &self,
        container_style: &str,
        graph_id: DefaultAtom,
        graph: NodeId,
        y_axis: NodeId,
        x_axis: NodeId,
    ) -> NodeId;
}

impl<T> GraphWidgetBuilder<T> for UiBuilder<T>
where
    T: AppState + 'static,
{
    fn graph_widget(
        &self,
        style: &str,
        id: DefaultAtom,
        data: Weak<RefCell<Vec<Vector<f32>>>>,
    ) -> NodeId {
        let _span = tracy_client::span!("graph_widget");

        // Get or create graph state
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), GraphWidgetData::<T>::default()),
        };
        let mut guard = binding.data.lock().unwrap();
        let pdata: &mut GraphWidgetData<T> = guard.downcast_mut().unwrap();
        pdata.graph_data = data;
        drop(guard);

        // Create graph widget
        let (style, mut context) = parse_style::<T>(style);
        context.flags |= flags::GRAPH;
        context.persistent_id = Some(id.clone());
        let graph_node = self
            .tree
            .borrow_mut()
            .new_leaf_with_context(style, context)
            .unwrap();

        // Add event listeners to graph
        let id_clone = id.clone();
        self.mutate_context(graph_node, move |ctx| {
            let id1 = id_clone.clone();
            ctx.on_middle_mouse_down = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    w_state.interaction = GraphInteraction::Panning {
                        pan_start_screen: state.mouse_pos,
                        start_limits: w_state.limits,
                    };
                });
            }));
            let id1 = id_clone.clone();
            ctx.on_middle_mouse_up = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    w_state.interaction = GraphInteraction::None;
                });
            }));
            let graph_node_id = graph_node;
            let id1 = id_clone.clone();
            ctx.on_mouse_move = Some(Arc::new(move |state| {
                let bbox = state
                    .get_node_bbox(graph_node_id)
                    .unwrap_or(Rect::from_points(
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
            let id1 = id_clone.clone();
            ctx.on_scroll = Some(Arc::new(move |state| {
                state.ui_builder.mutate_state(&id1, |w_state| {
                    let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
                    let width = w_state.limits.width();
                    w_state.limits.x0.x += width * 0.1 * state.scroll_delta.y.signum();
                    w_state.limits.x1.x -= width * 0.1 * state.scroll_delta.y.signum();
                });
            }));
        });

        graph_node
    }

    fn y_axis(&self, style: &str) -> NodeId {
        let compound_style = format!("w-100 {style}");
        let (stl, ctx) = parse_style::<T>(&compound_style);
        self.tree
            .borrow_mut()
            .new_leaf_with_context(stl, ctx)
            .unwrap()
    }

    fn x_axis(&self, style: &str) -> NodeId {
        let compound_style = format!("h-40 pl-100 w-full {style}");
        let (stl, ctx) = parse_style::<T>(&compound_style);
        self.tree
            .borrow_mut()
            .new_leaf_with_context(stl, ctx)
            .unwrap()
    }

    fn graph_with_axes(
        &self,
        container_style: &str,
        graph_id: DefaultAtom,
        graph: NodeId,
        y_axis: NodeId,
        x_axis: NodeId,
    ) -> NodeId {
        let _span = tracy_client::span!("graph_with_axes");

        // Extract colors from axis nodes
        let y_axis_tick_color;
        let y_axis_label_color;
        let x_axis_tick_color;
        let x_axis_label_color;

        {
            let tree = self.tree.borrow();
            let y_axis_ctx = tree
                .get_node_context(y_axis)
                .expect("y_axis node must exist in tree");
            let x_axis_ctx = tree
                .get_node_context(x_axis)
                .expect("x_axis node must exist in tree");

            y_axis_tick_color = y_axis_ctx.border.color;
            y_axis_label_color = y_axis_ctx.text.color;
            x_axis_tick_color = x_axis_ctx.border.color;
            x_axis_label_color = x_axis_ctx.text.color;
        }

        // Store axis colors in graph state
        self.mutate_state(&graph_id, |w_state| {
            let w_state: &mut GraphWidgetData<T> = w_state.downcast_mut().unwrap();
            w_state.y_axis_tick_color = y_axis_tick_color;
            w_state.y_axis_label_color = y_axis_label_color;
            w_state.x_axis_tick_color = x_axis_tick_color;
            w_state.x_axis_label_color = x_axis_label_color;
        });

        // Create container with flex-col layout
        // Structure: flex-col [ flex-row [y_axis, graph], x_axis ]
        let inner_row = self.div("flex-row grow", &[y_axis, graph]);
        let container = self.div(container_style, &[inner_row, x_axis]);

        container
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
// - [x] Show cursor xy-coordinates in data domain
// - [x] Trace selection
// - [x] Customizable colors for graph and ticks
