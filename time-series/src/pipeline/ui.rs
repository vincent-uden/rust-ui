use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use keybinds::KeyInput;
use rust_ui::render::{
    COLOR_LIGHT, COLOR_SUCCESS, Text,
    renderer::{AppState, Listeners, NodeContext, Renderer, UiBuilder, UiData},
};
use rust_ui::{id, render::renderer::DefaultAtom};
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
        focused_id: &Option<DefaultAtom>,
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
                    text_field(b, format!("{column_1}"), id!("cfg-{pipeline_idx}-{step_idx}-c1"), focused_id),
                    b.text("", Text::new("Value column", 12, COLOR_LIGHT)),
                    text_field(b, format!("{column_2}"), id!("cfg-{pipeline_idx}-{step_idx}-c2"), focused_id),
                    b.text_field(id!("cfg-{pipeline_idx}-{step_idx}-c2"), focused_id)
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
}

pub fn text_field(
    b: &UiBuilder<App>,
    text: String,
    id: DefaultAtom,
    focused_id: &Option<DefaultAtom>,
) -> NodeId {
    let style = if &Some(id) == focused_id {
        "w-full border-2 border-sky-500 rounded-4 bg-slate-900 py-2 px-4"
    } else {
        "w-full rounded-4 bg-slate-900 py-2 px-4"
    };
    b.ui(
        "",
        Listeners {
            on_left_mouse_down: Some(Arc::new(move |state| {})),
            ..Default::default()
        },
        &[b.text_explicit(style, Text::new(text, 12, COLOR_LIGHT))],
    )
}

/// This could in theory be used as a generic library, although how  styling is
/// supposed to work is not entirely well defined. Probably something like cva from
/// shadcn would work decently at least.
#[derive(Debug, Default)]
pub struct TextFieldData {
    pub contents: String,
    pub cursor_pos: usize,
    pub select_pos: usize,
}
impl UiData for TextFieldData {}

pub trait TextFieldBuilder {
    // TODO: Event listeners
    fn text_field(&self, id: DefaultAtom, focused_id: &Option<DefaultAtom>) -> NodeId;
}

impl<T> TextFieldBuilder for UiBuilder<T>
where
    T: AppState,
{
    fn text_field(&self, id: DefaultAtom, focused_id: &Option<DefaultAtom>) -> NodeId {
        // TODO: Render cursor and selection via context flag
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), TextFieldData::default()),
        };
        let guard = binding.data.lock().unwrap();
        let state: &TextFieldData = guard.downcast_ref().unwrap();

        let style = if Some(&id) == focused_id.as_ref() {
            "bg-slate-900 h-14 w-full p-2 rounded-4 border-2 border-sky-500"
        } else {
            "bg-slate-900 hover:bg-slate-800 h-14 w-full p-2 rounded-4"
        };
        self.ui(
            style,
            Listeners {
                on_left_mouse_down: Some(Arc::new(move |state: &mut Renderer<T>| {
                    state.set_focus(Some(id.clone()));
                })),
                ..Default::default()
            },
            &[self.text_explicit("", Text::new(state.contents.clone(), 12, COLOR_LIGHT))],
        )
    }
}
