use std::fmt::{Debug, Display};
use std::sync::Arc;

use string_cache::DefaultAtom;
use taffy::NodeId;

use crate::render::renderer::{AppState, Listeners};
use crate::render::widgets::{UiBuilder, UiData};
use crate::render::{COLOR_LIGHT, Text};

pub struct SelectData<T, S>
where
    S: PartialEq + Display,
    T: AppState,
{
    pub selected: Option<S>,
    pub open: bool,
    pub on_select: EventListener<T, S>,
}
impl<T, S> Clone for SelectData<T, S>
where
    T: AppState,
    S: PartialEq + Display + Clone,
{
    fn clone(&self) -> Self {
        Self {
            selected: self.selected.clone(),
            open: self.open.clone(),
            on_select: self.on_select.clone(),
        }
    }
}
impl<T, S> Default for SelectData<T, S>
where
    T: AppState,
    S: PartialEq + Display,
{
    fn default() -> Self {
        Self {
            selected: Default::default(),
            open: Default::default(),
            on_select: Default::default(),
        }
    }
}
impl<T, S> Debug for SelectData<T, S>
where
    T: AppState,
    S: PartialEq + Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectData")
            .field("selected", &self.selected)
            .field("open", &self.open)
            .field(
                "on_select",
                match self.on_select {
                    Some(_) => &"Some(...)",
                    None => &"None",
                },
            )
            .finish()
    }
}
impl<T, S> UiData<T> for SelectData<T, S>
where
    T: AppState + 'static,
    S: PartialEq + Display + Debug + 'static,
{
    fn run_event_listener(&mut self, name: &str, app: &mut T) {
        if name == "on_select" {
            if let Some(el) = self.on_select.take() {
                el(app, self)
            }
        }
    }
}

pub type EventListener<T, S> = Option<Arc<dyn Fn(&mut T, &SelectData<T, S>)>>;

pub trait SelectBuilder<T, S>
where
    T: AppState + 'static,
    S: PartialEq + Display + Debug + 'static,
{
    fn select(&self, id: DefaultAtom, options: &[S], on_select: EventListener<T, S>) -> NodeId;
}

impl<T, S> SelectBuilder<T, S> for UiBuilder<T>
where
    T: AppState + 'static,
    S: PartialEq + Display + Debug + 'static,
{
    fn select(&self, id: DefaultAtom, options: &[S], on_select: EventListener<T, S>) -> NodeId {
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), SelectData::<T, S>::default()),
        };
        let mut guard = binding.data.lock().unwrap();
        let state: &mut SelectData<T, S> = guard.downcast_mut().unwrap();
        state.on_select = on_select;

        let selected_label = match &state.selected {
            Some(s) => format!("{s}"),
            None => format!("Select..."),
        };

        let id1 = id.clone();
        let out = self.ui(
            "bg-slate-900 hover:bg-slate-800 w-200 rounded-4",
            Listeners {
                on_left_mouse_up: Some(Arc::new(move |state| {
                    state.ui_builder.mutate_state(&id1, |w_state| {
                        let w_state: &mut SelectData<T, S> = w_state.downcast_mut().unwrap();
                        w_state.open = !w_state.open;
                    });
                })),
                ..Default::default()
            },
            &[self.marker(
                "p-2",
                id.clone(),
                &[self.text_explicit("", Text::new(selected_label, 12, COLOR_LIGHT))],
            )],
        );
        let children: Vec<_> = options
            .iter()
            .map(|opt| self.text("", Text::new(format!("{opt}"), 12, COLOR_LIGHT)))
            .collect();
        if state.open {
            self.popup("bg-slate-900 flex-col flex-col", id.clone(), &children);
        }
        out
    }
}
