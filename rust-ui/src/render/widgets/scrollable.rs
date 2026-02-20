use std::sync::Arc;

use crate::render::renderer::{AppState, Listeners, NodeContext, Renderer};
use crate::render::widgets::{DefaultAtom, UiBuilder, UiData};
use crate::style::parse_style;
use taffy::NodeId;

#[derive(Debug, Clone)]
pub struct ScrollableData {
    /// Scroll position from 0.0 (top) to 1.0 (bottom)
    pub scroll_position: f32,
    /// How much to scroll per wheel tick (0.0 to 1.0)
    pub scroll_step: f32,
}

impl Default for ScrollableData {
    fn default() -> Self {
        Self {
            scroll_position: 0.0,
            scroll_step: 0.2,
        }
    }
}

impl<T> UiData<T> for ScrollableData where T: AppState {}

pub trait ScrollableBuilder {
    fn scrollable(
        &self,
        id: DefaultAtom,
        style: &str,
        children: impl IntoIterator<Item = NodeId>,
    ) -> NodeId;
}

impl<T> ScrollableBuilder for UiBuilder<T>
where
    T: AppState,
{
    fn scrollable(
        &self,
        id: DefaultAtom,
        style: &str,
        children: impl IntoIterator<Item = NodeId>,
    ) -> NodeId {
        let state = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), ScrollableData::default()),
        };
        let guard = state.data.lock().unwrap();
        let data: &ScrollableData = guard.downcast_ref().unwrap();
        let scroll_position = data.scroll_position;
        let scroll_step = data.scroll_step;
        drop(guard);

        let scrollbar = {
            let mut tree = self.tree.borrow_mut();
            let (stl, mut ctx) =
                parse_style("w-full bg-zinc-300 hover:bg-zinc-200 h-32 scroll-bar rounded-4");
            ctx.offset.y = scroll_position;
            tree.new_leaf_with_context(stl, ctx).unwrap()
        };

        let scroll_content = {
            let mut tree = self.tree.borrow_mut();
            let (mut stl, mut ctx) =
                parse_style(&format!("flex-col scroll-content min-h-0 w-full {}", style));
            // Make content absolutely positioned so it doesn't contribute to parent's content_size
            stl.position = taffy::Position::Absolute;
            stl.inset = taffy::Rect {
                left: taffy::LengthPercentageAuto::length(0.0),
                right: taffy::LengthPercentageAuto::length(0.0),
                top: taffy::LengthPercentageAuto::auto(),
                bottom: taffy::LengthPercentageAuto::auto(),
            };
            ctx.offset.y = scroll_position;
            let parent = tree.new_leaf_with_context(stl, ctx).unwrap();
            for child in children {
                tree.add_child(parent, child).unwrap();
            }
            parent
        };

        self.ui(
            "flex-row overflow-clip h-full min-h-0",
            Listeners::default(),
            &[
                self.ui(
                    "grow",
                    Listeners {
                        on_scroll: Some(Arc::new(move |renderer: &mut Renderer<T>| {
                            let delta = renderer.scroll_delta.y.signum() * scroll_step;
                            renderer
                                .ui_builder
                                .mutate_state(&id, |ui_data: &mut dyn UiData<T>| {
                                    let d: &mut ScrollableData = ui_data.downcast_mut().unwrap();
                                    d.scroll_position = (d.scroll_position - delta).clamp(0.0, 1.0);
                                });
                        })),
                        ..Default::default()
                    },
                    &[scroll_content],
                ),
                self.div("w-8", &[scrollbar]),
            ],
        )
    }
}
