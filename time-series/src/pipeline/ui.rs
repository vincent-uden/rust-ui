use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use keybinds::KeyInput;
use rust_ui::{
    id,
    render::{
        COLOR_LIGHT, COLOR_SUCCESS, Text,
        renderer::{AppState, Listeners, NodeContext, Renderer},
        widgets::{
            DefaultAtom, UiBuilder, scrollable::ScrollableBuilder as _,
            text_field::TextFieldBuilder as _,
        },
    },
};
use taffy::{NodeId, TaffyTree};
use tracing::info;

use crate::{
    app::App,
    pipeline::{DataFrame, StepConfig},
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

pub struct PipelineManagerUi {
    pub sources: Arc<RefCell<Vec<DataSource>>>,
    pub selected_source: Option<usize>,
    pub pipelines: Vec<Vec<StepConfig>>,
}

impl PipelineManagerUi {
    pub fn new(sources: Arc<RefCell<Vec<DataSource>>>) -> Self {
        Self {
            sources,
            selected_source: None,
            pipelines: Vec::new(),
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
                    state.app_state.add_source();
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
            b.div("h-1 w-full bg-slate-500 my-4", &[]),
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
            for (c_idx, cfg) in self.pipelines[idx].iter().enumerate() {
                pipeline_rows.push(self.step_config(*cfg, idx, c_idx, &b, focused_id));
            }
        }
        let pipeline_container = b.scrollable(id!("pipeline_scrollable"), "", pipeline_rows);
        signal_rows.push(pipeline_container);
        let outer = b.div("flex-col gap-4 min-h-0", &signal_rows);
        outer
    }

    fn signal_row(&self, source: &DataSource, b: &UiBuilder<App>, idx: usize) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        b.ui("flex-row hover:bg-slate-600 py-2", Listeners {
            on_left_mouse_up: Some(Arc::new(move |state| {
                state.app_state.pipeline_manager.selected_source = Some(idx);
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
        pipeline_idx: usize,
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
                    id!("cfg-{pipeline_idx}-{step_idx}-smoothing"),
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
                        id!("cfg-{pipeline_idx}-{step_idx}-c1"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            let pm = &mut app.pipeline_manager;
                            let pos = pm
                                .available_columns()
                                .iter()
                                .position(|col| col == &data.contents);
                            pm.pipelines[pm.selected_source.unwrap()][step_idx] =
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
                        id!("cfg-{pipeline_idx}-{step_idx}-c2"),
                        focused_id,
                        Some(Arc::new(move |app, data| {
                            let pm = &mut app.pipeline_manager;
                            let pos = pm
                                .available_columns()
                                .iter()
                                .position(|col| col == &data.contents);
                            pm.pipelines[pm.selected_source.unwrap()][step_idx] =
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

        let step_types = &[
            b.text_button(
                "flex-row hover:bg-slate-600 py-2",
                Text::new("Pick columns", 12, COLOR_LIGHT),
                Listeners {
                    on_left_mouse_down: Some(Arc::new(move |state| {
                        state.app_state.pipeline_manager.set_cfg_step(
                            StepConfig::PickColumns {
                                column_1: 0,
                                column_2: 1,
                            },
                            step_idx,
                        );
                    })),
                    ..Default::default()
                },
            ),
            b.text_button(
                "flex-row hover:bg-slate-600 py-2",
                Text::new("Smooth signal", 12, COLOR_LIGHT),
                Listeners {
                    on_left_mouse_down: Some(Arc::new(move |state| {
                        state
                            .app_state
                            .pipeline_manager
                            .set_cfg_step(StepConfig::SmoothSignal { window: 10 }, step_idx);
                    })),
                    ..Default::default()
                },
            ),
        ];
        let mut inner = vec![
            b.text("", Text::new(format!("{cfg}"), 14, COLOR_LIGHT)),
            b.div("h-4", &[]),
        ];
        inner.extend_from_slice(step_types);
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
            self.pipelines[selected][step_idx] = new_step;
        }
    }
}
