use std::fmt::Debug;
use std::sync::Arc;

use crate::render::renderer::AppState;
use crate::render::widgets::UiData;

pub struct SelectData<T>
where
    T: AppState,
{
    pub open: bool,
    pub on_select: EventListener<T>,
}
impl<T> Clone for SelectData<T>
where
    T: AppState,
{
    fn clone(&self) -> Self {
        Self {
            open: self.open.clone(),
            on_select: self.on_select.clone(),
        }
    }
}
impl<T> Default for SelectData<T>
where
    T: AppState,
{
    fn default() -> Self {
        Self {
            open: Default::default(),
            on_select: Default::default(),
        }
    }
}
impl<T> Debug for SelectData<T>
where
    T: AppState,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectData")
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
impl<T> UiData<T> for SelectData<T>
where
    T: AppState + 'static,
{
    fn run_event_listener(&mut self, name: &str, app: &mut T) {
        if name == "on_select" {
            if let Some(el) = self.on_select.take() {
                el(app, self)
            }
        }
    }
}

pub type EventListener<T> = Option<Arc<dyn Fn(&mut T, &SelectData<T>)>>;
