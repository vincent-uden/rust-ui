use std::cell::RefCell;

use rust_ui::render::{
    COLOR_LIGHT, Text,
    renderer::{NodeContext, UiBuilder},
};
use taffy::{NodeId, TaffyTree};

use crate::app::App;

pub struct PipelineManagerUi {}

impl PipelineManagerUi {
    pub fn generate_layout(self, tree: &RefCell<TaffyTree<NodeContext<App>>>, parent: NodeId) {
        let b = UiBuilder::new(tree);
        #[cfg_attr(any(), rustfmt::skip)]
        let outer = b.div("", &[
            b.text("", Text::new("Signals", 14, COLOR_LIGHT)),
        ]);

        tree.borrow_mut().add_child(parent, outer).unwrap();
    }
}
