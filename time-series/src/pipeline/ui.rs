use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use rust_ui::render::{
    COLOR_LIGHT, COLOR_SUCCESS, Text,
    renderer::{Listeners, NodeContext, UiBuilder},
};
use taffy::{NodeId, TaffyTree};

use crate::{app::App, pipeline::DataFrame};

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
}

impl PipelineManagerUi {
    pub fn new(sources: Arc<RefCell<Vec<DataSource>>>) -> Self {
        Self {
            sources,
            selected_source: None,
        }
    }

    pub fn generate_layout(&self, tree: &RefCell<TaffyTree<NodeContext<App>>>) -> NodeId {
        let b = UiBuilder::new(tree);

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
            signal_rows.push(signal_row(&source, &b, i, Some(i) == self.selected_source));
        }
        signal_rows.extend_from_slice(&[
            b.div("h-1 w-full bg-slate-500 my-4", &[]),
            b.text("", Text::new("Pipeline", 14, COLOR_LIGHT)),
        ]);
        let outer = b.div("flex-col gap-4", &signal_rows);
        outer
    }
}

pub fn signal_row(source: &DataSource, b: &UiBuilder<App>, idx: usize, selected: bool) -> NodeId {
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
            if selected { COLOR_SUCCESS } else {COLOR_LIGHT})
        ),
    ])
}
