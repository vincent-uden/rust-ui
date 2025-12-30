use std::sync::Arc;

use crate::render::renderer::{AppState, Listeners, Renderer, flags};
use crate::render::widgets::{DefaultAtom, UiBuilder, UiData};
use crate::render::{COLOR_LIGHT, Text};
use crate::style::parse_style;
use taffy::NodeId;

pub struct TextFieldData<T>
where
    T: AppState,
{
    pub contents: String,
    pub cursor_pos: usize,
    pub select_pos: usize,
    pub on_confirm: EventListener<T>,
}
impl<T> Clone for TextFieldData<T>
where
    T: AppState,
{
    fn clone(&self) -> Self {
        Self {
            contents: self.contents.clone(),
            cursor_pos: self.cursor_pos.clone(),
            select_pos: self.select_pos.clone(),
            on_confirm: self.on_confirm.clone(),
        }
    }
}
impl<T> Default for TextFieldData<T>
where
    T: AppState,
{
    fn default() -> Self {
        Self {
            contents: Default::default(),
            cursor_pos: Default::default(),
            select_pos: Default::default(),
            on_confirm: None,
        }
    }
}
impl<T> std::fmt::Debug for TextFieldData<T>
where
    T: AppState,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextFieldData")
            .field("contents", &self.contents)
            .field("cursor_pos", &self.cursor_pos)
            .field("select_pos", &self.select_pos)
            .field(
                "on_confirm",
                match self.on_confirm {
                    Some(_) => &"Some(...)",
                    None => &"None",
                },
            )
            .finish()
    }
}
impl<T> TextFieldData<T>
where
    T: AppState,
{
    pub fn move_cursor(&mut self, arg: isize) {
        self.cursor_pos = self
            .cursor_pos
            .saturating_add_signed(arg)
            .clamp(0, self.contents.len());
    }

    pub fn write(&mut self, ch: char) {
        self.contents.insert(self.cursor_pos, ch);
        self.move_cursor(1);
    }

    pub fn delete_char(&mut self) {
        if !self.contents.is_empty() {
            self.contents.remove(self.cursor_pos - 1);
            self.move_cursor(-1)
        }
    }
}
impl<T> UiData<T> for TextFieldData<T>
where
    T: AppState + 'static,
{
    fn run_event_listener(&mut self, name: &str, app: &mut T) {
        if name == "confirm" {
            if let Some(el) = self.on_confirm.take() {
                el(app, self)
            }
        }
    }
}

pub type EventListener<T> = Option<Arc<dyn Fn(&mut T, &TextFieldData<T>)>>;

pub trait TextFieldBuilder<T>
where
    T: AppState,
{
    // TODO: Event listeners
    /// Text fields are single line text inputs
    fn text_field(
        &self,
        id: DefaultAtom,
        focused_id: &Option<DefaultAtom>,
        on_confirm: EventListener<T>,
    ) -> NodeId;
}

impl<T> TextFieldBuilder<T> for UiBuilder<T>
where
    T: AppState,
{
    fn text_field(
        &self,
        id: DefaultAtom,
        focused_id: &Option<DefaultAtom>,
        on_confirm: EventListener<T>,
    ) -> NodeId {
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), TextFieldData::<T>::default()),
        };
        let mut guard = binding.data.lock().unwrap();
        let state: &mut TextFieldData<T> = guard.downcast_mut().unwrap();
        state.on_confirm = on_confirm;

        let (style, mut context) = parse_style("");
        context.text = Text::new(state.contents.clone(), 12, COLOR_LIGHT);
        context.flags |= flags::TEXT | flags::TEXT_SCROLL | flags::TEXT_SINGLE_LINE;
        context.cursor_idx = Some(state.cursor_pos);
        let inner_text = {
            let mut tree = self.tree.borrow_mut();
            tree.new_leaf_with_context(style, context).unwrap()
        };

        let style = if Some(&id) == focused_id.as_ref() {
            "bg-slate-900 h-14 w-200 p-2 rounded-4 border-2 border-sky-500"
        } else {
            "bg-slate-900 hover:bg-slate-800 h-14 w-200 p-2 rounded-4"
        };
        self.ui(
            style,
            Listeners {
                on_left_mouse_down: Some(Arc::new(move |state: &mut Renderer<T>| {
                    state.set_focus(Some(id.clone()));
                })),
                ..Default::default()
            },
            &[inner_text],
        )
    }
}
