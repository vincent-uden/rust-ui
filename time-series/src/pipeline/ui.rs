use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use keybinds::KeyInput;
use rust_ui::id;
use rust_ui::render::{
    COLOR_LIGHT, COLOR_SUCCESS, Text,
    renderer::{Listeners, NodeContext, UiBuilder, UiData},
};
use smol_str::SmolStr;
use taffy::{NodeId, TaffyTree};

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

    pub fn generate_layout(&self, b: &UiBuilder<App>, focused_id: &Option<SmolStr>) -> NodeId {
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
        ]);
        if let Some(idx) = self.selected_source {
            for (c_idx, cfg) in self.pipelines[idx].iter().enumerate() {
                signal_rows.push(self.step_config(&cfg, idx, c_idx, &b, focused_id));
            }
        }
        let outer = b.div("flex-col gap-4", &signal_rows);
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

    fn step_config(
        &self,
        cfg: &StepConfig,
        pipeline_idx: usize,
        step_idx: usize,
        b: &UiBuilder<App>,
        focused_id: &Option<SmolStr>,
    ) -> NodeId {
        #[cfg_attr(any(), rustfmt::skip)]
        let form = match cfg {
            StepConfig::Average => todo!(),
            StepConfig::Variance => todo!(),
            StepConfig::SmoothSignal { window } => todo!(),
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
            StepConfig::PickColumns { column_1, column_2 } => b.div(
                "flex-col gap-4",
                &[
                    b.text("", Text::new("Time column", 12, COLOR_LIGHT)),
                    text_field(b, format!("{column_1}"), format!("cfg-{pipeline_idx}-{step_idx}-c1"), focused_id),
                    b.text("", Text::new("Value column", 12, COLOR_LIGHT)),
                    text_field(b, format!("{column_2}"), format!("cfg-{pipeline_idx}-{step_idx}-c2"), focused_id),
                ],
            ),
            StepConfig::ScaleAxis { axis, factor } => todo!(),
            StepConfig::LogAxis { axis, base } => todo!(),
        };
        #[cfg_attr(any(), rustfmt::skip)]
        b.div("flex-col border-2 border-slate-500 rounded-8 p-8", &[
            b.text("", Text::new(format!("{cfg}"), 14, COLOR_LIGHT)),
            b.div("h-4", &[]),
            form,
        ])
    }

    pub fn handle_key_input(&self, key_input: KeyInput, focused_id: &Option<SmolStr>) {
        //
    }
}

pub fn text_field(
    b: &UiBuilder<App>,
    text: String,
    id: impl Into<SmolStr>,
    focused_id: &Option<SmolStr>,
) -> NodeId {
    let as_smol: SmolStr = id.into();
    let style = if &Some(as_smol.clone()) == focused_id {
        "w-full border-2 border-sky-500 rounded-4 bg-slate-900 py-2 px-4"
    } else {
        "w-full rounded-4 bg-slate-900 py-2 px-4"
    };
    b.ui(
        "",
        Listeners {
            on_left_mouse_down: Some(Arc::new(move |state| {
                let as_smol = as_smol.clone();
                if !as_smol.is_empty() {
                    state.app_state.focus = Some(as_smol);
                }
            })),
            ..Default::default()
        },
        &[b.text_explicit(style, Text::new(text, 12, COLOR_LIGHT))],
    )
}

pub fn stateful(b: &UiBuilder<App>) {}

#[derive(Debug)]
pub struct TextFieldData {
    contents: String,
    cursor_pos: usize,
    select_pos: usize,
}

impl UiData for TextFieldData {}

pub trait StatefulAccess {
    fn stateful(&self);
}

impl StatefulAccess for UiBuilder<App> {
    fn stateful(&self) {
        if let Some(state) = self.accessing_state(id!("TextField{}", 1)) {
            let data: Arc<TextFieldData> = Arc::downcast(state.data).unwrap();
        }
    }
}
