use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use anyhow::{Result, anyhow};
use keybinds::KeyInput;
use rust_ui::{
    geometry::{Rect, Vector},
    id,
    render::{
        COLOR_DANGER, COLOR_LIGHT, COLOR_SUCCESS, Text,
        renderer::{AppState, Listeners, NodeContext, Renderer},
        widgets::{
            DefaultAtom, UiBuilder,
            scrollable::ScrollableBuilder as _,
            select::{self, SelectBuilder},
            text_field::TextFieldBuilder as _,
        },
    },
};
use taffy::{NodeId, TaffyTree};
use tracing::{error, info};

use crate::{
    app::{App, AppMessage},
    pipeline::{
        AxisSelection, DataFrame, PipelineIntermediate, Record, SignalKind, StepConfig,
        processing::{average, run_pipeline},
    },
};

pub struct DataSource {
    df: DataFrame,
    path: PathBuf,
}

impl DataSource {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        Ok(Self {
            df: DataFrame::from_path(&path)?,
            path: path,
        })
    }
}

pub struct Pipeline {
    pub steps: Vec<StepConfig>,
    pub step_ids: Vec<usize>,
    pub next_id: usize,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            step_ids: Vec::new(),
            next_id: 0,
        }
    }

    pub fn push(&mut self, step: StepConfig) {
        self.steps.push(step);
        self.step_ids.push(self.next_id);
        self.next_id += 1;
    }

    pub fn remove(&mut self, idx: usize) -> StepConfig {
        self.step_ids.remove(idx);
        self.steps.remove(idx)
    }
}

pub struct PipelineManagerUi {
    pub sources: Arc<RefCell<Vec<DataSource>>>,
    pub selected_source: Option<usize>,
    pub pipelines: Vec<Pipeline>,
    pub outputs: Vec<PipelineIntermediate>,
    pub as_points: Vec<Rc<RefCell<Vec<Vector<f32>>>>>,
}

impl PipelineManagerUi {
    pub fn new(sources: Arc<RefCell<Vec<DataSource>>>) -> Self {
        Self {
            sources,
            selected_source: None,
            pipelines: Vec::new(),
            outputs: Vec::new(),
            as_points: Vec::new(),
        }
    }

    pub fn generate_layout(&self, b: &UiBuilder<App>, focused_id: &Option<DefaultAtom>) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        let mut signal_rows = vec![ b.div("flex-row gap-8", &[
            b.div("p-4 pt-6", &[
                b.text("", Text::new("Sources", 14, COLOR_LIGHT)),
            ]),
            b.ui("py-6 px-8 rounded-8 bg-slate-600 hover:bg-slate-500", Listeners {
                on_left_mouse_up: Some(Arc::new(|state| {
                    state.app_state.add_source_dialog();
                })),
                ..Default::default()
            }, &[
                b.text("", Text::new("Add", 14, COLOR_SUCCESS)),
            ])
        ])];
        for (i, source) in self.sources.borrow().iter().enumerate() {
            signal_rows.push(self.signal_row(&source, &b, i));
        }
        signal_rows.extend_from_slice(&[b.text("", Text::new("Pipeline", 14, COLOR_LIGHT))]);
        let mut pipeline_rows = vec![];
        if let Some(idx) = self.selected_source {
            for (c_idx, cfg) in self.pipelines[idx].steps.iter().enumerate() {
                let step_id = self.pipelines[idx].step_ids[c_idx];
                let valid_steps = self.get_valid_steps(c_idx);
                pipeline_rows.push(self.step_config(
                    *cfg,
                    step_id,
                    c_idx,
                    &valid_steps,
                    &b,
                    focused_id,
                ));
            }
        }
        pipeline_rows.push(b.ui(
            "py-6 px-8 rounded-8 bg-slate-600 hover:bg-slate-500",
            Listeners {
                on_left_mouse_up: Some(Arc::new(|state| {
                    state.app_state.add_step();
                })),
                ..Default::default()
            },
            &[b.text("", Text::new("Add", 14, COLOR_SUCCESS))],
        ));
        let pipeline_container = b.scrollable(id!("pipeline_scrollable"), "gap-8", pipeline_rows);
        signal_rows.push(pipeline_container);
        signal_rows.push(b.div(
            "p-8",
            &[b.ui(
                "bg-green-600 hover:bg-green-700 rounded-8 flex-col items-center w-full py-4",
                Listeners {
                    on_left_mouse_down: Some(Arc::new(move |state| {
                        state.app_state.pipeline_manager.run();
                    })),
                    ..Default::default()
                },
                &[b.text("", Text::new("Run", 14, COLOR_LIGHT))],
            )],
        ));
        let outer = b.div("flex-col gap-4 min-h-0", &signal_rows);
        outer
    }

    fn signal_row(&self, source: &DataSource, b: &UiBuilder<App>, idx: usize) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        b.ui("flex-row hover:bg-slate-600 py-2", Listeners {
            on_left_mouse_up: Some(Arc::new(move |state| {
                state.app_state.pipeline_manager.selected_source = Some(idx);
                state.app_state.handle_message(AppMessage::ZoomFit, &state.ui_builder);
            })),
            ..Default::default()
        }, &[
            b.text("", Text::new(
                format!("{}", source.path.file_name().unwrap_or_default().display()),
                12,
                if Some(idx) == self.selected_source { COLOR_SUCCESS } else {COLOR_LIGHT})
            ),
        ])
    }

    fn text_button(
        b: &UiBuilder<App>,
        text: Text,
        on_click: Arc<dyn Fn(&mut Renderer<App>)>,
    ) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        b.ui("flex-row hover:bg-slate-600 py-2", Listeners {
            on_left_mouse_up: Some(on_click),
            ..Default::default()
        }, &[
            b.text("", text)
        ])
    }

    fn step_config(
        &self,
        cfg: StepConfig,
        step_id: usize,
        step_idx: usize,
        valid_steps: &[StepConfig],
        b: &UiBuilder<App>,
        focused_id: &Option<DefaultAtom>,
    ) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        let form = match cfg {
            StepConfig::Average => b.div("", &[]),
            StepConfig::Variance => b.div("", &[]),
            StepConfig::SmoothSignal { window } => b.div(
                "flex-col gap-4",
                &[b.text_field(
                    id!("cfg-{step_id}-smoothing"),
                    focused_id,
                    Some(Arc::new(move |app, data| {
                        if let Ok(new_window) = data.contents.parse() {
                            app.pipeline_manager.set_cfg_step(
                                StepConfig::SmoothSignal { window: new_window },
                                step_idx,
                            );
                        }
                    })),
                )],
            ),
            StepConfig::SmoothReals { window } => b.div(
                "flex-col gap-4",
                &[
                    b.text("", Text::new("Window size", 12, COLOR_LIGHT)),
                    b.text_field(
                        id!("cfg-{step_id}-smoothing-reals"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            if let Ok(new_window) = data.contents.parse() {
                                app.pipeline_manager.set_cfg_step(
                                    StepConfig::SmoothReals { window: new_window },
                                    step_idx,
                                );
                            }
                        })),
                    ),
                ],
            ),
            StepConfig::AbsoluteValueOfReals => b.div("", &[]),
            StepConfig::FourierTransform => b.div("", &[]),
            StepConfig::InverseFourierTransform => b.div("", &[]),
            StepConfig::PostFFTFormatting => b.div("", &[]),
            StepConfig::SkipFirstEntry => b.div("", &[]),
            StepConfig::SkipFirstComplexEntry => b.div("", &[]),
            StepConfig::Normalize => b.div("", &[]),
            StepConfig::BandpassFilter { middle, half_width } => b.div(
                "flex-col gap-4",
                &[
                    b.text("", Text::new("Middle frequency", 12, COLOR_LIGHT)),
                    b.text_field(
                        id!("cfg-{step_id}-bp-middle"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            if let Ok(new_middle) = data.contents.parse() {
                                app.pipeline_manager.set_cfg_step(
                                    StepConfig::BandpassFilter {
                                        middle: new_middle,
                                        half_width,
                                    },
                                    step_idx,
                                );
                            }
                        })),
                    ),
                    b.text("", Text::new("Half width", 12, COLOR_LIGHT)),
                    b.text_field(
                        id!("cfg-{step_id}-bp-width"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            if let Ok(new_width) = data.contents.parse() {
                                app.pipeline_manager.set_cfg_step(
                                    StepConfig::BandpassFilter {
                                        middle,
                                        half_width: new_width,
                                    },
                                    step_idx,
                                );
                            }
                        })),
                    ),
                ],
            ),
            StepConfig::CurrentCalculator {
                capacitance: _,
                x1: _,
                x2: _,
            } => b.text("", Text::new("Current calculator (not implemented)", 12, COLOR_LIGHT)),
            StepConfig::PickColumns { column_1, column_2 } => {
                let columns = self.available_columns();
                let selected_time = columns.get(column_1).cloned();
                let selected_value = columns.get(column_2).cloned();

                b.div(
                    "flex-col gap-4",
                    &[
                        b.text("", Text::new("Time column", 12, COLOR_LIGHT)),
                        b.select(
                            id!("cfg-{step_id}-c1"),
                            selected_time,
                            &columns,
                            Some(Arc::new(move |app, _, selected| {
                                let pm = &mut app.pipeline_manager;
                                let columns = pm.available_columns();
                                let pos = columns.iter().position(|col| col == selected);
                                if let Some(idx) = pos {
                                    pm.set_cfg_step(
                                        StepConfig::PickColumns {
                                            column_1: idx,
                                            column_2,
                                        },
                                        step_idx,
                                    );
                                }
                            })),
                        ),
                        b.text("", Text::new("Value column", 12, COLOR_LIGHT)),
                        b.select(
                            id!("cfg-{step_id}-c2"),
                            selected_value,
                            &columns,
                            Some(Arc::new(move |app, _, selected| {
                                let pm = &mut app.pipeline_manager;
                                let columns = pm.available_columns();
                                let pos = columns.iter().position(|col| col == selected);
                                if let Some(idx) = pos {
                                    pm.set_cfg_step(
                                        StepConfig::PickColumns {
                                            column_1,
                                            column_2: idx,
                                        },
                                        step_idx,
                                    );
                                }
                            })),
                        ),
                    ],
                )
            }
            StepConfig::ScaleAxis { axis, factor } => b.div(
                "flex-col gap-4",
                &[
                    b.text("", Text::new("Axis", 12, COLOR_LIGHT)),
                    b.select(
                        id!("cfg-{step_id}-scale-axis"),
                        Some(axis),
                        &[AxisSelection::X, AxisSelection::Y],
                        Some(Arc::new(move |app, _, selected| {
                            app.pipeline_manager.set_cfg_step(
                                StepConfig::ScaleAxis {
                                    axis: *selected,
                                    factor,
                                },
                                step_idx,
                            );
                        })),
                    ),
                    b.text("", Text::new("Factor", 12, COLOR_LIGHT)),
                    b.text_field(
                        id!("cfg-{step_id}-scale-factor"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            if let Ok(new_factor) = data.contents.parse() {
                                app.pipeline_manager.set_cfg_step(
                                    StepConfig::ScaleAxis {
                                        axis,
                                        factor: new_factor,
                                    },
                                    step_idx,
                                );
                            }
                        })),
                    ),
                ],
            ),
            StepConfig::LogAxis { axis, base } => b.div(
                "flex-col gap-4",
                &[
                    b.text("", Text::new("Axis", 12, COLOR_LIGHT)),
                    b.select(
                        id!("cfg-{step_id}-log-axis"),
                        Some(axis),
                        &[AxisSelection::X, AxisSelection::Y],
                        Some(Arc::new(move |app, _, selected| {
                            app.pipeline_manager.set_cfg_step(
                                StepConfig::LogAxis {
                                    axis: *selected,
                                    base,
                                },
                                step_idx,
                            );
                        })),
                    ),
                    b.text("", Text::new("Base", 12, COLOR_LIGHT)),
                    b.text_field(
                        id!("cfg-{step_id}-log-base"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            if let Ok(new_base) = data.contents.parse() {
                                app.pipeline_manager.set_cfg_step(
                                    StepConfig::LogAxis {
                                        axis,
                                        base: new_base,
                                    },
                                    step_idx,
                                );
                            }
                        })),
                    ),
                ],
            ),
        };

        let mut inner = vec![b.div(
            "flex-row gap-4",
            &[
                b.select(
                    id!("step-select-{step_id}"),
                    Some(cfg.clone()),
                    valid_steps,
                    Some(Arc::new(move |app, _, selected| {
                        app.pipeline_manager.set_cfg_step(*selected, step_idx);
                    })),
                ),
                #[cfg_attr(any(), rustfmt::skip)]
                    Self::text_button(b, Text::new("X", 18, COLOR_DANGER), Arc::new(move |state| {
                        state.app_state.pipeline_manager.remove_step(step_idx);
                    })),
            ],
        )];
        inner.extend_from_slice(&[b.div("h-4", &[]), form]);

        #[cfg_attr(any(), rustfmt::skip)]
        b.div("flex-col border-2 border-slate-500 rounded-8 p-8", &inner)
    }

    fn available_columns(&self) -> Vec<String> {
        let sources = self.sources.borrow();
        if let Some(idx) = self.selected_source {
            sources
                .get(idx)
                .map(|s| s.df.column_names.clone())
                .unwrap_or(vec![])
        } else {
            vec![]
        }
    }

    /// Modifies an existing configuration step in the currently selected pipeline.
    pub fn set_cfg_step(&mut self, new_step: StepConfig, step_idx: usize) {
        if let Some(selected) = self.selected_source {
            self.pipelines[selected].steps[step_idx] = new_step;
        }
    }

    /// Returns the valid steps that can be placed at the given position in the pipeline.
    /// First step must accept DataFrame, other steps must match previous step's output.
    /// If there's a next step, the selected step's output must match the next step's input.
    fn get_valid_steps(&self, step_idx: usize) -> Vec<StepConfig> {
        let Some(pipeline_idx) = self.selected_source else {
            return Vec::new();
        };
        let pipeline = &self.pipelines[pipeline_idx];

        // Determine expected input kind
        let expected_input = if step_idx == 0 {
            SignalKind::DataFrame
        } else {
            pipeline.steps[step_idx - 1].output_kind()
        };

        // Determine required output kind (if there's a next step)
        let required_output = if step_idx < pipeline.steps.len() - 1 {
            Some(pipeline.steps[step_idx + 1].input_kind())
        } else {
            None
        };

        // Filter all steps to only include valid ones
        StepConfig::all()
            .into_iter()
            .filter(|step| {
                let input_matches = step.input_kind() == expected_input;
                let output_matches = match &required_output {
                    Some(required) => step.output_kind() == *required,
                    None => true,
                };
                input_matches && output_matches
            })
            .collect()
    }

    /// Returns a default step that can be added at the given position.
    /// Returns None if no valid step exists for this position.
    pub fn get_default_step_for_position(&self, step_idx: usize) -> Option<StepConfig> {
        let valid_steps = self.get_valid_steps(step_idx);
        valid_steps.first().cloned()
    }

    pub fn remove_step(&mut self, idx: usize) {
        if let Some(selected) = self.selected_source {
            self.pipelines[selected].remove(idx);
        }
    }

    pub fn run(&mut self) {
        self.outputs.clear();
        let sources = self.sources.borrow();
        for (source, pipeline) in sources.iter().zip(self.pipelines.iter()) {
            match run_pipeline(
                &pipeline.steps,
                PipelineIntermediate::DataFrame(source.df.clone()),
            ) {
                Ok(signal) => {
                    self.outputs.push(signal);
                }
                Err(e) => {
                    error!("{}", e);
                    self.outputs.push(PipelineIntermediate::Signal(Vec::new()));
                }
            }
        }
        self.as_points.clear();
        for output in &self.outputs {
            match output {
                PipelineIntermediate::Signal(records) => {
                    let points: Vec<Vector<f32>> = records
                        .iter()
                        .map(|r| Vector::new(r.x as f32, r.y as f32))
                        .collect();
                    self.as_points.push(Rc::new(RefCell::new(points)));
                }
                PipelineIntermediate::Complex(_) | PipelineIntermediate::DataFrame(_) => {
                    self.as_points.push(Rc::new(RefCell::new(Vec::new())));
                }
            }
        }
    }

    /// Returns the smallest possible rect containing all points in all outputs
    pub fn minimum_spanning_limits(&self) -> Rect<f32> {
        let mut limits = Rect::default();
        self.selected_source
            .map(|idx| match &self.outputs.get(idx) {
                Some(PipelineIntermediate::Signal(records)) => {
                    for r in records {
                        if r.x < limits.x0.x {
                            limits.x0.x = r.x;
                        } else if r.x > limits.x1.x {
                            limits.x1.x = r.x;
                        }
                        if r.y < limits.x0.y {
                            limits.x0.y = r.y;
                        } else if r.y > limits.x1.y {
                            limits.x1.y = r.y;
                        }
                    }
                }
                _ => {}
            });

        limits.into()
    }
}
