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
        DataFrame, PipelineIntermediate, Record, StepConfig,
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
        signal_rows.extend_from_slice(&[
            b.text("", Text::new("Pipeline", 14, COLOR_LIGHT)),
            b.ui(
                "py-6 px-8 rounded-8 bg-slate-600 hover:bg-slate-500",
                Listeners {
                    on_left_mouse_up: Some(Arc::new(|state| {
                        state.app_state.add_step();
                    })),
                    ..Default::default()
                },
                &[b.text("", Text::new("Add", 14, COLOR_SUCCESS))],
            ),
        ]);
        let mut pipeline_rows = vec![];
        if let Some(idx) = self.selected_source {
            for (c_idx, cfg) in self.pipelines[idx].steps.iter().enumerate() {
                let step_id = self.pipelines[idx].step_ids[c_idx];
                pipeline_rows.push(self.step_config(*cfg, step_id, c_idx, &b, focused_id));
            }
        }
        let pipeline_container = b.scrollable(id!("pipeline_scrollable"), "", pipeline_rows);
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
        b: &UiBuilder<App>,
        focused_id: &Option<DefaultAtom>,
    ) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        let form = match cfg {
            StepConfig::Average => todo!(),
            StepConfig::Variance => todo!(),
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
            StepConfig::SmoothReals { window } => todo!(),
            StepConfig::AbsoluteValueOfReals => todo!(),
            StepConfig::FourierTransform => todo!(),
            StepConfig::InverseFourierTransform => todo!(),
            StepConfig::PostFFTFormatting => todo!(),
            StepConfig::SkipFirstEntry => todo!(),
            StepConfig::SkipFirstComplexEntry => todo!(),
            StepConfig::Normalize => todo!(),
            StepConfig::BandpassFilter { middle, half_width } => todo!(),
            StepConfig::CurrentCalculator {
                capacitance,
                x1,
                x2,
            } => todo!(),
            StepConfig::PickColumns { column_1, column_2 } => {
                let available: Vec<_> = self
                    .available_columns()
                    .iter()
                    .map(|col| b.text("", Text::new(col.clone(), 12, COLOR_LIGHT)))
                    .collect();
                let mut controls = vec![
                    b.text(
                        "",
                        Text::new(format!("Time column ({column_1})"), 12, COLOR_LIGHT),
                    ),
                    b.text_field(
                        id!("cfg-{step_id}-c1"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            let pm = &mut app.pipeline_manager;
                            let pos = pm
                                .available_columns()
                                .iter()
                                .position(|col| col == &data.contents);
                            pm.pipelines[pm.selected_source.unwrap()].steps[step_idx] =
                                StepConfig::PickColumns {
                                    column_1: pos.unwrap_or(column_1),
                                    column_2: column_2,
                                };
                        })),
                    ),
                    b.text(
                        "",
                        Text::new(format!("Value column ({column_2})"), 12, COLOR_LIGHT),
                    ),
                    b.text_field(
                        id!("cfg-{step_id}-c2"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            let pm = &mut app.pipeline_manager;
                            let pos = pm
                                .available_columns()
                                .iter()
                                .position(|col| col == &data.contents);
                            pm.pipelines[pm.selected_source.unwrap()].steps[step_idx] =
                                StepConfig::PickColumns {
                                    column_1: column_1,
                                    column_2: pos.unwrap_or(column_2),
                                };
                        })),
                    ),
                ];
                controls.extend_from_slice(&available);
                b.div("flex-col gap-4", &controls)
            }
            StepConfig::ScaleAxis { axis, factor } => todo!(),
            StepConfig::LogAxis { axis, base } => todo!(),
        };

        let mut inner = vec![
            b.div(
                "flex-row gap-4",
                &[
                    b.select(
                        id!("step-select-{step_id}"),
                        Some(cfg.clone()),
                        &StepConfig::all(),
                        Some(Arc::new(move |app, _, selected| {
                            app.pipeline_manager.set_cfg_step(*selected, step_idx);
                        })),
                    ),
                    #[cfg_attr(any(), rustfmt::skip)]
                    Self::text_button(b, Text::new("X", 18, COLOR_DANGER), Arc::new(move |state| {
                        state.app_state.pipeline_manager.remove_step(step_idx);
                    })),
                ],
            ),
            b.div("h-4", &[]),
        ];
        inner.extend_from_slice(&[b.div("h-4", &[]), form]);

        #[cfg_attr(any(), rustfmt::skip)]
        b.div("flex-col border-2 border-slate-500 rounded-8 p-8 mt-8", &inner)
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
