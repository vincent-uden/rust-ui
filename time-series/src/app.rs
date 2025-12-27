use std::{any::Any, cell::RefCell, str::FromStr, sync::Arc};

use glfw::Action;
use modes::{Config, ModeStack};
use rust_ui::{
    geometry::Vector,
    input::glfw_key_to_key_input,
    render::{
        COLOR_LIGHT, Text,
        renderer::{AppState, DefaultAtom, RenderLayout, TextFieldData, UiBuilder},
    },
};
use strum::EnumString;
use tracing::error;

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
    pub focus: Option<DefaultAtom>,
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

    pub fn add_step(&mut self) {
        if let Some(selected) = self.pipeline_manager.selected_source {
            self.pipeline_manager.pipelines[selected].push(StepConfig::PickColumns {
                column_1: 0,
                column_2: 0,
            });
        }
    }

    pub fn base_layer(&self, window_size: Vector<f32>, ui: &UiBuilder<Self>) -> RenderLayout<Self> {
        #[cfg_attr(any(), rustfmt::skip)]
        let root = ui.div("w-full h-full flex-col bg-slate-700 p-4 gap-4", &[
            ui.div("flex-row", &[
                ui.text("", Text::new("Time series explorer", 16, COLOR_LIGHT))
            ]),
            ui.div("flex-row grow gap-4", &[
                ui.div("w-full h-full bg-slate-900", &[]),
                self.pipeline_manager.generate_layout(ui, &self.focus),
            ]),
        ]);

        RenderLayout {
            tree: ui.tree(),
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
        ui: &UiBuilder<Self>,
    ) -> Vec<rust_ui::render::renderer::RenderLayout<Self>> {
        vec![self.base_layer(window_size, ui)]
    }

    fn handle_key(
        &mut self,
        key: glfw::Key,
        _scancode: glfw::Scancode,
        action: glfw::Action,
        modifiers: glfw::Modifiers,
        ui: &UiBuilder<Self>,
    ) {
        // TODO: Repeat doesnt seem to be happening
        if action == Action::Press || action == Action::Repeat {
            match glfw_key_to_key_input(key, modifiers) {
                Some(key_input) => {
                    if let Some(msg) = self
                        .mode_stack
                        .dispatch(&mut self.config.bindings, key_input)
                    {
                        self.handle_message(msg);
                    } else {
                        if self.mode_stack.is_outermost(&AppMode::Typing) {
                            if let Some(focused) = &self.focus {
                                match key_input.key() {
                                    keybinds::Key::Char(ch) => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData =
                                                ui_data.downcast_mut().unwrap();
                                            d.write(ch);
                                        });
                                    }
                                    keybinds::Key::Right => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData =
                                                ui_data.downcast_mut().unwrap();
                                            d.move_cursor(1);
                                        });
                                    }
                                    keybinds::Key::Left => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData =
                                                ui_data.downcast_mut().unwrap();
                                            d.move_cursor(-1);
                                        });
                                    }
                                    keybinds::Key::Backspace => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData =
                                                ui_data.downcast_mut().unwrap();
                                            d.delete_char();
                                        });
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
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

    fn handle_mouse_button(
        &mut self,
        _button: glfw::MouseButton,
        action: Action,
        _modifiers: glfw::Modifiers,
        _ui: &UiBuilder<Self>,
    ) {
        match action {
            Action::Press => {
                // Since this runs before event listeners, this won't erase any actual clicks on focused objects
                self.focus = None;
            }
            _ => {}
        }
    }

    fn set_focus(&mut self, focus: Option<rust_ui::render::renderer::DefaultAtom>) {
        self.focus = focus;
        if self.focus.is_some() && !self.mode_stack.is_outermost(&AppMode::Typing) {
            self.mode_stack.push(AppMode::Typing);
        }
    }
}
