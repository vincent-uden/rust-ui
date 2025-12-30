use std::sync::Arc;

use crate::render::renderer::{AppState, EventListener, Listeners, Renderer, flags};
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
    pub on_confirm: Option<EventListener<T>>,
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
impl<T> UiData for TextFieldData<T> where T: AppState + 'static {}

pub trait TextFieldBuilder {
    // TODO: Event listeners
    /// Text fields are single line text inputs
    fn text_field(&self, id: DefaultAtom, focused_id: &Option<DefaultAtom>) -> NodeId;
}

impl<T> TextFieldBuilder for UiBuilder<T>
where
    T: AppState + 'static,
{
    fn text_field(&self, id: DefaultAtom, focused_id: &Option<DefaultAtom>) -> NodeId {
        // TODO:
        //       Also include a scrollable in case the text grows larger than the box for fixed-width cases
        let binding = match self.accessing_state(&id) {
            Some(s) => s,
            None => self.insert_state(id.clone(), TextFieldData::<T>::default()),
        };
        let guard = binding.data.lock().unwrap();
        let state: &TextFieldData<T> = guard.downcast_ref().unwrap();

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
