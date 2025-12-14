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
