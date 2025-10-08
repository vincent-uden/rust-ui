NodeContext attributes

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



Style attributes

        Dont display element and its children, otherwise this is automatically flex
display `None`

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
