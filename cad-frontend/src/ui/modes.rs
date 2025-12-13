use std::{cell::RefCell, sync::Arc};

use glm::vec3;
use modes::ModeStack;
use rust_ui::render::{
    COLOR_LIGHT, Text,
    renderer::{Listeners, NodeContext, Renderer, UiBuilder},
};
use taffy::{NodeId, TaffyTree};

use crate::{
    app::{self, App, AppMutableState},
    modes::{AppBindableMessage, AppMode},
};

#[derive(Debug, Clone, Copy)]
pub struct Modes {}

impl Modes {
    pub fn generate_layout(
        tree: &RefCell<TaffyTree<NodeContext<App>>>,
        parent: NodeId,
        mode_stack: &ModeStack<AppMode, AppBindableMessage>,
    ) {
        let mut mode_stack_fmt = String::from("Mode: ");
        for m in mode_stack.modes() {
            mode_stack_fmt.push_str(&format!("{} > ", m));
        }
        mode_stack_fmt.pop();
        mode_stack_fmt.pop();
        mode_stack_fmt.pop();

        let b = UiBuilder::new(tree);
        #[cfg_attr(any(), rustfmt::skip)]
        let outer = b.div("flex-col", [
            b.div("px-8 pb-8 pt-30 flex-row gap-8 items-stretch w-full h-auto", match *mode_stack.outermost().unwrap() {
                AppMode::Sketch | AppMode::Point | AppMode::Line | AppMode::Circle => vec![
                    mode_button(&b, "Point", Arc::new(move |state| {
                        state.app_state.mode_stack.pop_until(&AppMode::Sketch);
                        state.app_state.mode_stack.push(AppMode::Point);
                    })),
                    mode_button(&b, "Line", Arc::new(move |state| {
                        state.app_state.mode_stack.pop_until(&AppMode::Sketch);
                        state.app_state.mode_stack.push(AppMode::Line);
                    })),
                    mode_button(&b, "Circle", Arc::new(move |state| {
                        state.app_state.mode_stack.pop_until(&AppMode::Sketch);
                        state.app_state.mode_stack.push(AppMode::Circle);
                    })),
                    mode_button(&b, "Finish Sketch", Arc::new(move |state| {
                        state.app_state.mode_stack.pop_until(&AppMode::Base);
                    })),
                ],
                AppMode::Base => vec![
                    mode_button(&b, "New sketch", Arc::new(move |state| {
                        let mut mut_state = state.app_state.mutable_state.borrow_mut();
                        // TODO: Pick the plane
                        mut_state.scene.add_sketch(cad::Plane {
                            x: vec3(1.0, 0.0, 0.0),
                            y: vec3(0.0, 1.0, 0.0),
                        });
                    }))
                ],
            }),
            b.div("", [
                b.text("pl-12", Text { text: mode_stack_fmt, font_size: 14, color: COLOR_LIGHT })
            ])
        ]);
        tree.borrow_mut().add_child(parent, outer).unwrap();
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
