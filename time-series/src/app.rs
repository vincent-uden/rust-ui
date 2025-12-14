use std::{cell::RefCell, sync::Arc};

use rust_ui::{
    geometry::Vector,
    render::{
        COLOR_LIGHT, Text,
        renderer::{AppState, RenderLayout, UiBuilder},
    },
};
use taffy::TaffyTree;
use tracing::info;

use crate::pipeline::ui::{DataSource, PipelineManagerUi};

pub struct App {
    pub sources: Arc<RefCell<Vec<DataSource>>>,
    pub pipeline_manager: PipelineManagerUi,
}

impl App {
    pub fn new() -> Self {
        let sources = Arc::new(RefCell::new(Vec::new()));
        Self {
            sources: sources.clone(),
            pipeline_manager: PipelineManagerUi::new(sources.clone()),
        }
    }

    pub fn add_source(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .pick_file()
        {
            if let Ok(source) = DataSource::from_path(path) {
                self.sources.borrow_mut().push(source.into());
            }
        }
    }

    pub fn base_layer(&self, window_size: Vector<f32>) -> RenderLayout<Self> {
        let tree = TaffyTree::new().into();
        let b = UiBuilder::new(&tree);
        #[cfg_attr(any(), rustfmt::skip)]
        let root = b.div("w-full h-full flex-col bg-slate-700 p-4 gap-4", &[
            b.div("flex-row", &[
                b.text("", Text::new("Time series explorer", 16, COLOR_LIGHT))
            ]),
            b.div("flex-row grow gap-4", &[
                b.div("w-full h-full bg-slate-900", &[]),
                self.pipeline_manager.generate_layout(&tree),
            ]),
        ]);

        RenderLayout {
            tree: tree.into_inner(),
            root,
            desired_size: window_size.into(),
            ..Default::default()
        }
    }
}

impl AppState for App {
    type SpriteKey = String;

    fn generate_layout(
        &mut self,
        window_size: rust_ui::geometry::Vector<f32>,
    ) -> Vec<rust_ui::render::renderer::RenderLayout<Self>> {
        vec![self.base_layer(window_size)]
    }
}
