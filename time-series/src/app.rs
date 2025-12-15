use std::{cell::RefCell, str::FromStr, sync::Arc};

use glfw::Action;
use modes::{Config, ModeStack};
use rust_ui::{
    geometry::Vector,
    input::glfw_key_to_key_input,
    render::{
        COLOR_LIGHT, Text,
        renderer::{AppState, RenderLayout, UiBuilder},
    },
};
use smol_str::SmolStr;
use strum::EnumString;
use taffy::TaffyTree;
use tracing::{error, info};

use crate::pipeline::{
    StepConfig,
    ui::{DataSource, PipelineManagerUi},
};

#[derive(EnumString, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AppMode {
    Base,
    Typing,
}

#[derive(EnumString, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMessage {
    PopMode,
    Confirm,
}

fn default_config() -> Config<AppMode, AppMessage, AppMessage> {
    Config::from_str(include_str!("../assets/default.conf")).unwrap()
}

pub struct App {
    pub sources: Arc<RefCell<Vec<DataSource>>>,
    pub pipeline_manager: PipelineManagerUi,
    pub focus: Option<SmolStr>,
    pub mode_stack: ModeStack<AppMode, AppMessage>,
    pub config: Config<AppMode, AppMessage, AppMessage>,
}

impl App {
    pub fn new() -> Self {
        let sources = Arc::new(RefCell::new(Vec::new()));
        Self {
            sources: sources.clone(),
            pipeline_manager: PipelineManagerUi::new(sources.clone()),
            focus: None,
            mode_stack: ModeStack::with_base(AppMode::Base),
            config: default_config(),
        }
    }

    pub fn add_source(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .pick_file()
        {
            if let Ok(source) = DataSource::from_path(path) {
                self.sources.borrow_mut().push(source.into());
                self.pipeline_manager
                    .pipelines
                    .push(vec![StepConfig::PickColumns {
                        column_1: 0,
                        column_2: 1,
                    }]);
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
                self.pipeline_manager.generate_layout(&tree, &self.focus),
            ]),
        ]);

        RenderLayout {
            tree: tree.into_inner(),
            root,
            desired_size: window_size.into(),
            ..Default::default()
        }
    }

    pub fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::PopMode => {
                self.mode_stack.pop();
            }
            AppMessage::Confirm => todo!(),
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

    fn handle_key(
        &mut self,
        key: glfw::Key,
        scancode: glfw::Scancode,
        action: glfw::Action,
        modifiers: glfw::Modifiers,
    ) {
        if action == Action::Press {
            match glfw_key_to_key_input(key, modifiers) {
                Some(key_input) => {
                    if let Some(msg) = self
                        .mode_stack
                        .dispatch(&mut self.config.bindings, key_input)
                    {
                        self.handle_message(msg);
                    }
                }
                None => {
                    error!(
                        "Couldn't convert GLFW key {:?} {:?} {:?} to keybinds-key",
                        key, modifiers, action
                    );
                }
            }
        }
    }
}
