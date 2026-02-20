use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr, sync::Arc};

use glfw::Action;
use modes::{Config, ModeStack};
use rust_ui::{
    geometry::Vector,
    id,
    input::glfw_key_to_key_input,
    render::{
        COLOR_DANGER, COLOR_LIGHT, Text,
        renderer::{AppState, Listeners, RenderLayout},
        widgets::{DefaultAtom, UiBuilder, UiData, text_field::TextFieldData},
    },
};
use strum::EnumString;
use tracing::error;

use crate::{
    graph_widget::{GraphInteraction, GraphWidgetBuilder, GraphWidgetData},
    pipeline::{
        StepConfig,
        ui::{DataSource, Pipeline, PipelineManagerUi},
    },
};

#[derive(EnumString, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AppMode {
    Base,
    Typing,
}

#[derive(EnumString, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMessage {
    PopMode,
    ZoomFit,
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
    pub mouse_pos: Vector<f32>,
    pub test_popup_open: bool,
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
            mouse_pos: Default::default(),
            test_popup_open: false,
        }
    }

    pub fn add_source_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            //.add_filter("CSV files", &["csv"])
            .pick_file()
        {
            self.add_source(path);
        }
    }

    pub fn add_source(&mut self, path: PathBuf) {
        if let Ok(source) = DataSource::from_path(path) {
            self.sources.borrow_mut().push(source.into());
            let mut pipeline = Pipeline::new();
            pipeline.push(StepConfig::PickColumns {
                column_1: 0,
                column_2: 1,
            });
            self.pipeline_manager.pipelines.push(pipeline);
            self.pipeline_manager.run();
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
            ui.div("flex-row pb-12", &[
                ui.text("", Text::new("Time series explorer", 16, COLOR_LIGHT)),
                ui.div("w-30", &[]),
                ui.ui("", Listeners {
                    on_left_mouse_up: Some(Arc::new(|state| { state.app_state.test_popup_open = !state.app_state.test_popup_open })),
                    ..Default::default()
                }, &[
                    ui.marker("hover:bg-slate-900", id!("popup0"), &[
                        ui.text("", Text::new("Popuptest", 16, COLOR_LIGHT)),
                    ]),
                ])
            ]),
            ui.div("flex-row grow gap-4 h-full", &[
                ui.div("flex-col gap-4 grow pr-32", &[
                    {
                        ui.graph_with_axes(
                            "flex-col h-full",
                            id!("main_graph"),
                            ui.graph_widget(
                                "w-full h-full text-white border-red-400",
                                id!("main_graph"),
                                Rc::downgrade(self.pipeline_manager.as_points
                                    .get(self.pipeline_manager.selected_source.unwrap_or(0))
                                    .unwrap_or(&Rc::new(RefCell::new(Vec::new())))),
                            ),
                            ui.y_axis("border-slate-400 text-gray-300"),
                            ui.x_axis("border-slate-400 text-gray-300"),
                        )
                    },
                    ui.div("flex-row grow gap-4 p-4", &[
                        ui.text_button("py-6 px-8 rounded-8 bg-slate-600 hover:bg-slate-500", Text::new("Zoom fit", 16, COLOR_LIGHT), Listeners {
                            on_left_mouse_up: Some(Arc::new(|state| {
                                state.app_state.handle_message(AppMessage::ZoomFit, &state.ui_builder);
                            })),
                            ..Default::default()
                        }),
                    ]),
                ]),
                self.pipeline_manager.generate_layout(ui, &self.focus),
            ]),
        ]);

        if self.test_popup_open {
            ui.popup(
                "bg-slate-800 hover:bg-sky-600 rounded-8 p-8 border-slate-500 border-2",
                id!("popup0"),
                &[ui.ui(
                    "",
                    Listeners {
                        on_left_mouse_down: Some(Arc::new(|state| {
                            state.app_state.test_popup_open = false
                        })),
                        ..Default::default()
                    },
                    &[ui.text("", Text::new("I am the popup!", 16, COLOR_LIGHT))],
                )],
            );
        }

        RenderLayout {
            tree: ui.tree(),
            delayed_markers: ui.delayed_ids(),
            root,
            desired_size: window_size.into(),
            ..Default::default()
        }
    }

    pub fn tooltip_layer(
        &self,
        _window_size: Vector<f32>,
        ui: &UiBuilder<Self>,
    ) -> RenderLayout<Self> {
        let binding = match ui.accessing_state(&id!("main_graph")) {
            Some(s) => s,
            None => ui.insert_state(id!("main_graph"), GraphWidgetData::<Self>::default()),
        };
        let mut guard = binding.data.lock().unwrap();
        let data: &mut GraphWidgetData<Self> = guard.downcast_mut().unwrap();
        let data_pos = data.screen_coord_to_data_coord(self.mouse_pos);
        let mut tooltip_info = vec![
            ui.text("", Text::new(format!("x: {}", data_pos.x), 12, COLOR_LIGHT)),
            ui.text("", Text::new(format!("y: {}", data_pos.y), 12, COLOR_LIGHT)),
        ];

        if let GraphInteraction::Measuring { measure_start } = data.interaction {
            let start_pos = data.screen_coord_to_data_coord(measure_start);
            let diff = data_pos - start_pos;
            tooltip_info.push(ui.text(
                "",
                Text::new(
                    format!("Measurement delta: ({}, {})", diff.x, diff.y),
                    12,
                    COLOR_DANGER,
                ),
            ));
            if diff.x != 0.0 {
                tooltip_info.push(ui.text(
                    "",
                    Text::new(format!("Slope: ({})", diff.y / diff.x), 12, COLOR_DANGER),
                ));
            }
        }

        #[cfg_attr(any(), rustfmt::skip)]
        let root = ui.div("bg-slate-700 rounded-8 flex-col w-200 px-12 pb-8 pt-6 gap-0 border-2 border-slate-500", &tooltip_info);

        RenderLayout {
            tree: ui.tree(),
            delayed_markers: ui.delayed_ids(),
            root,
            desired_size: taffy::Size {
                width: taffy::AvailableSpace::MinContent,
                height: taffy::AvailableSpace::MinContent,
            },
            root_pos: Vector::new(110.0, 40.0),
            ..Default::default()
        }
    }

    pub fn handle_message(&mut self, msg: AppMessage, ui: &UiBuilder<Self>) -> Vec<String> {
        match msg {
            AppMessage::PopMode => {
                self.mode_stack.pop();
                self.focus = None;
            }
            AppMessage::ZoomFit => {
                let limits = self.pipeline_manager.minimum_spanning_limits();
                ui.mutate_state(&id!("main_graph"), move |w_state| {
                    let w_state: &mut GraphWidgetData<Self> = w_state.downcast_mut().unwrap();
                    w_state.limits = limits;
                });
            }
            // TODO: Think about how this should be communicated. I want the state change
            // localized at the UI. That requires notifying Renderer. Still the App might
            // want to decide when a "Confirm is taking place"
            //
            // The Renderer won't necessarily know what sort of widgets exist. I guess
            // the widgets need to know when they are triggered?
            AppMessage::Confirm => {
                if let Some(focus) = &self.focus
                    && let Some(state) = ui.accessing_state(focus)
                {
                    let mut data = state.data.lock().unwrap();
                    let text_data: &mut TextFieldData<Self> = data.downcast_mut().unwrap();
                    text_data.run_event_listener("confirm", self);
                }
            }
        }
        vec![]
    }
}

impl AppState for App {
    type SpriteKey = String;

    fn generate_layout(
        &mut self,
        window_size: rust_ui::geometry::Vector<f32>,
        ui: &UiBuilder<Self>,
    ) -> Vec<rust_ui::render::renderer::RenderLayout<Self>> {
        vec![
            self.base_layer(window_size, ui),
            self.tooltip_layer(window_size, ui),
        ]
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
                        self.handle_message(msg, ui);
                    } else {
                        if self.mode_stack.is_outermost(&AppMode::Typing) {
                            if let Some(focused) = &self.focus {
                                match key_input.key() {
                                    keybinds::Key::Right => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData<Self> =
                                                ui_data.downcast_mut().unwrap();
                                            d.move_cursor(1);
                                        });
                                    }
                                    keybinds::Key::Left => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData<Self> =
                                                ui_data.downcast_mut().unwrap();
                                            d.move_cursor(-1);
                                        });
                                    }
                                    keybinds::Key::Backspace => {
                                        ui.mutate_state(focused, |ui_data| {
                                            let d: &mut TextFieldData<Self> =
                                                ui_data.downcast_mut().unwrap();
                                            d.delete_char();
                                        });
                                    }
                                    _ => {}
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

    fn handle_char(&mut self, unicode: u32, ui: &UiBuilder<Self>) {
        if let Some(ch) = char::from_u32(unicode) {
            if self.mode_stack.is_outermost(&AppMode::Typing) {
                if let Some(focused) = &self.focus {
                    if !ch.is_control() {
                        ui.mutate_state(focused, |ui_data| {
                            let d: &mut TextFieldData<Self> = ui_data.downcast_mut().unwrap();
                            d.write(ch);
                        });
                    }
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

    fn set_focus(&mut self, focus: Option<DefaultAtom>) {
        self.focus = focus;
        if self.focus.is_some() && !self.mode_stack.is_outermost(&AppMode::Typing) {
            self.mode_stack.push(AppMode::Typing);
        }
    }

    fn handle_mouse_position(&mut self, position: Vector<f32>, _delta: Vector<f32>) {
        self.mouse_pos = position;
    }
}

/// Test scenarios to speed up debugging
impl App {
    pub fn new_with_sawtooth_data_added() -> (Self, Vec<AppMessage>) {
        let mut out = Self::new();
        out.add_source(
            PathBuf::from_str("time-series/assets/test_csvs/sawtooth.csv")
                .expect("sawtooth.csv must exist. Make sure cwd is the rust-ui workspace root"),
        );
        out.pipeline_manager.selected_source = Some(0);
        out.add_step();
        (out, vec![AppMessage::ZoomFit])
    }

    pub fn new_with_voltage_data_added() -> (App, Vec<AppMessage>) {
        let mut out = Self::new();
        out.add_source(
            PathBuf::from_str("time-series/assets/test_csvs/VOUT02.CSV")
                .expect("VOUT02.CSV must exist. Make sure cwd is the rust-ui workspace root"),
        );
        out.pipeline_manager.selected_source = Some(0);
        (out, vec![AppMessage::ZoomFit])
    }
}
