use std::{cell::RefCell, sync::Arc};

use glm::vec3;
use rust_ui::render::{
    COLOR_LIGHT, Text,
    renderer::{Listeners, NodeContext, Renderer, UiBuilder},
};
use taffy::{NodeId, TaffyTree};

use crate::app::{self, App, AppMutableState, SketchMode};

#[derive(Debug, Clone, Copy)]
pub struct Modes {}

impl Modes {
    pub fn generate_layout(
        tree: &RefCell<TaffyTree<NodeContext<App>>>,
        parent: NodeId,
        state: &AppMutableState,
    ) {
        let b = UiBuilder::new(tree);
        #[cfg_attr(any(), rustfmt::skip)]
        let container = b.div( "px-8 pb-8 pt-30 flex-row gap-8 items-stretch w-full h-auto", match state.mode {
            app::Mode::EditSketch(i, _) => vec![
                mode_button(&b, "Point", Arc::new(move |state| {
                    state.app_state.mutable_state.borrow_mut().mode =
                        app::Mode::EditSketch(i, SketchMode::Point);
                })),
                mode_button(&b, "Finish Sketch", Arc::new(move |state| {
                    state.app_state.mutable_state.borrow_mut().mode =
                        app::Mode::None;
                })),
            ],
            app::Mode::None => vec![
                mode_button(&b, "New sketch", Arc::new(move |state| {
                    let mut mut_state = state.app_state.mutable_state.borrow_mut();
                    // TODO: Pick the plane
                    mut_state.scene.add_sketch(cad::Plane {
                        x: vec3(1.0, 0.0, 0.0),
                        y: vec3(0.0, 1.0, 0.0),
                    });
                }))
            ],
        });
        tree.borrow_mut().add_child(parent, container).unwrap();
    }
}

fn mode_button(
    b: &UiBuilder<App>,
    label: &str,
    on_click: Arc<dyn Fn(&mut Renderer<App>)>,
) -> NodeId {
    let mut listeners = Listeners::default();
    listeners.on_left_mouse_up = Some(on_click);
    #[cfg_attr(any(), rustfmt::skip)]
    b.ui( "p-4 items-center bg-nord1 hover:bg-nord3", listeners, &[
        b.text( "", Text { text: label.into(), font_size: 14, color: COLOR_LIGHT, },
    )])
}
