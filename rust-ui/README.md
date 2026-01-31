## Tailwind Style parsing

### NodeContext attributes

flags *Should be controlled by the semantic piece of UI it is in. Not a styling option*
sprite_key *Probably not managed by text attribute*
on_mouse_enter
on_mouse_exit
on_mouse_down
on_mouse_up

bg_color `bg-<color>`
bg_color_hover `hover:bg-<color>`

       Corner radius    Thickness       Color
border `rounded-<size>` `border-<size>` `border-<color>`

       Color          Font size
text   `text-<color>` `text-<size>`

       Horizontal translation Vertical Translatio
offset `translate-x-<pixels>` `translate-y-<pixels>`



### Style attributes

        Dont display element and its children, otherwise this is automatically flex
display `none`

margin  `m-<size>` `mx-<size>` `my-<size>` `ml-<size>` `mr-<size>` `mt-<size>` `mb-<size>`
padding `p-<size>` `px-<size>` `py-<size>` `pl-<size>` `pr-<size>` `pt-<size>` `pb-<size>`

flex_direction `flex-row` `flex-col`
flex_wrap `flex-nowrap` `flex-wrap` `flex-wrap-reverse`
flex_grow `grow` `grow-<size>`
flex_shrink `shrink` `shrink-<size>`

gap `gap-<size>`

align_items `items-start` `items-end`...
align_self `self-start` `self-end`...
justify_items `justify-items-start` `justify-items-end`...
justify_self `justify-self-start` `justify-self-end`...


inset
size
min_size
max_size
aspect_ratio
border
align_content
justify_content
text_align
flex_basis

grid_template_rows
grid_template_columns
grid_auto_rows
grid_auto_columns
grid_auto_flow
grid_template_areas
grid_template_column_names
grid_template_row_names
grid_row
grid_column

Ignored style attributes (for now?)

item_is_table
item_is_replaced
box_sizing

*Think about these when implementing scroll*
overflow `overflow-visible` `overflow-clip` `overflow-scroll`
scrollbar_width
position

## Interactive UI elements
Some of these will be tricky to implement.

### Editable text field
This can basically be rendered as a regular Text ui element with the addition of a cursor. But I need to think about how the keybinds and focus state needs to work. Focus is global, only one element can be focused at a time so this should probably be stored in the app state. Since apps can move focus using bindings in the mode system this focus state needs to be stored in there and not in the renderer.
```rust
pub struct TextField {
    pub text: String,
    pub font_size: u32,
    pub color: Color,
    pub cursor_idx: usize,
    pub focused: bool,
}
```

How is this stored in the App state? An id for the text field? I think that is the only way since I don't know up front what the UI heirarchy is going to look like.
```rust
pub struct App {
    focus: SmolStr,
}
```
How do I generate this id? I guess the id can be a string for now.

This is such a big pain point since I dont update and render at the same time.

## What if I wrote a retained GUI mode instead?
This would simplify so incredibly much of my thinking. I do think it would need both reflection and a lot of dynamic dispatch though. The most important thing is that stateful widgets are so much easier to create in a retained mode. 

The biggest gripe I have so far with immediate mode gui is that I can't build re-usable widgets that can be dropped in anywhere in the codebase.

Since I don't have inheritance to work with, I guess I would need to build an entity-component-system for my UI. A button would for example be composed of a container and an event listener component.

Ryan Fleury uses a global HashMap for all his persistent state which is keyed by a special text string syntax. This is really clever, but a global HashMap is not very Rusty. We can pass it down to every single ui function. Ryan seems to be doing that for his arena allocator, so why not pass the state HashMap? I'd need some kind of refcell to hold the hashmap and other related state in the renderer I guess. Or should it live in the app state? App state for now, renderer later if possible.

How would that HashMap work? I'm guessing Ryan just throws in a void pointer to an arena-allocated piece of memory. Then, based on what sort of UI element he's using he can cast that pointer to an actual struct pointer. Since the arena is heap memory I guess my equivalent would be a 
```rust
struct UiState {
    last_touched: usize,
    data: Box<dyn Any>,
}

let state = HashMap<String, UiState>;
```
## A render "Plugin" system
The goal isn't actually to write a plugin system, but just to allow for some way to render widgets with persistent data that isn't defined in the core `rust-ui` crate. As an example we have the graph widget I am building right now. In `time-series` we the `GraphWidgetData` which implements `UiData`. It contains cached data needed to display the graph. I can't downcast `&UiData` to `&GraphWidgetData` in `rust-ui` since it doesn't (and can't) depend on `time-series`.

Why not move `GraphWidgetData` into `rust-ui`? It contains data types specific to `time-series`. While simple to fix for this case, moving everything to core doesn't generalize well. It might be easy for `time-series` but not for `cad-frontend`.

The obvious solution is a trait
```rust
pub trait WidgetRenderer<T> where T: AppState {
    fn render(id: &NodeId, ctx: &NodeContext<T>, state: HashMap<DefaultAtom, UiState<T>>) -> Result<(), Box<dyn Error>)>;
}
```
where the trait method has to determine if it can actually render the widget it is attempting to render. This will however create a lot of redundant work for every non-standard widget. Ideally the widget would somehow know which trait object it is supposed to be rendered by. And this can't be determined at the compiletime of `rust-ui`. I think this would need some kind of reflection.

`UiState` already uses reflection but `NodeContext` doesn't but would need to in order for the system to be able to render widgets without any persistent state. It feels like this is starting to become a lot of boilerplate. 

Would it be possible for `UiState` to have this trait method instead? It can have an empty default implementation for widgets that don't need any special case rendering. I think so!
