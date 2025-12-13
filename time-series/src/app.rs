use rust_ui::{
    geometry::Vector,
    render::renderer::{AppState, RenderLayout, UiBuilder},
};
use taffy::TaffyTree;

#[derive(Default)]
pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn base_layer(&self, window_size: Vector<f32>) -> RenderLayout<Self> {
        let tree = TaffyTree::new().into();
        let b = UiBuilder::new(&tree);
        let root = b.div("w-full h-full", &[]);

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
