use std::{cell::RefCell, sync::Arc};

use modes::ModeStack;
use rust_ui::{
    geometry::Vector,
    render::{
        COLOR_LIGHT, NORD3, NORD7, Text,
        renderer::{Listeners, NodeContext, Renderer, UiBuilder, flags},
    },
};
use taffy::{
    AlignItems, Dimension, FlexDirection, NodeId, Rect, Size, Style, TaffyTree,
    prelude::{auto, length},
};

use crate::{
    app::{App, AppMutableState},
    modes::{AppBindableMessage, AppMode},
};

#[derive(Debug, Clone, Copy)]
pub struct SceneExplorer {}

impl SceneExplorer {
    pub fn generate_layout(
        tree: &RefCell<TaffyTree<NodeContext<App>>>,
        parent: NodeId,
        state: &AppMutableState,
        mode_stack: &ModeStack<AppMode, AppBindableMessage>,
    ) {
        let b = UiBuilder::new(tree);
        #[cfg_attr(any(), rustfmt::skip)]
        let header = b.text("mb-12", Text {
            text: state
                .scene
                .path
                .as_ref()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "Untitled".into()),
            font_size: 18,
            color: COLOR_LIGHT,
        });
        let mut sketch_rows = vec![header];
        #[cfg_attr(any(), rustfmt::skip)]
        for (i, sketch) in state.scene.sketches.iter().enumerate() {
            let mut s_color = if sketch.visible { COLOR_LIGHT } else { NORD3 };
            if mode_stack.is_active(&AppMode::Sketch) {
                if state.sketch_mode_data.sketch_id == sketch.id {
                    s_color = NORD7;
                }
            }
            let sketch_id = sketch.id;
            let id = b.div("flex-row gap-4 items-center", &[
                b.text("grow", Text {
                    text: sketch.name.clone(),
                    font_size: 14,
                    color: s_color,
                }),
                b.sprite("w-24 h-24 translate-y-2", if sketch.visible { "Visible" } else { "Invisible" }, Listeners {
                    on_left_mouse_up: Some(Arc::new(move |state| {
                        state.app_state.toggle_visibility(sketch_id);
                    })),
                    ..Default::default()
                }),
                b.sprite("w-24 h-24 translate-y-3", "EditSketch", Listeners {
                    on_left_mouse_up: Some(Arc::new(move |state| {
                        state.app_state.edit_sketch(sketch_id);
                    })),
                    ..Default::default()
                })
            ]);
            sketch_rows.push(id);
        }
        let container = b.div(
            "px-8 pt-30 pb-8 flex-col items-stretch w-full",
            &sketch_rows,
        );

        tree.borrow_mut().add_child(parent, container).unwrap();
    }
}
