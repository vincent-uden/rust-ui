use clay_layout::{
    Clay, Declaration,
    layout::{Alignment, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding, Sizing},
    text::TextConfig,
};

use crate::render::{Color, clay::ClayRenderer};

// Nord color scheme - https://www.nordtheme.com/
// Polar Night (dark colors)
pub const NORD0: Color = Color {
    r: 0.18,
    g: 0.20,
    b: 0.25,
    a: 1.0,
}; // #2e3440
pub const NORD1: Color = Color {
    r: 0.23,
    g: 0.26,
    b: 0.32,
    a: 1.0,
}; // #3b4252
pub const NORD2: Color = Color {
    r: 0.26,
    g: 0.30,
    b: 0.37,
    a: 1.0,
}; // #434c5e
pub const NORD3: Color = Color {
    r: 0.30,
    g: 0.34,
    b: 0.42,
    a: 1.0,
}; // #4c566a

// Snow Storm (light colors)
pub const NORD4: Color = Color {
    r: 0.85,
    g: 0.87,
    b: 0.91,
    a: 1.0,
}; // #d8dee9
pub const NORD5: Color = Color {
    r: 0.90,
    g: 0.91,
    b: 0.94,
    a: 1.0,
}; // #e5e9f0
pub const NORD6: Color = Color {
    r: 0.93,
    g: 0.94,
    b: 0.96,
    a: 1.0,
}; // #eceff4

// Frost (blue colors)
pub const NORD7: Color = Color {
    r: 0.56,
    g: 0.74,
    b: 0.73,
    a: 1.0,
}; // #8fbcbb
pub const NORD8: Color = Color {
    r: 0.53,
    g: 0.75,
    b: 0.82,
    a: 1.0,
}; // #88c0d0
pub const NORD9: Color = Color {
    r: 0.51,
    g: 0.63,
    b: 0.76,
    a: 1.0,
}; // #81a1c1
pub const NORD10: Color = Color {
    r: 0.37,
    g: 0.51,
    b: 0.67,
    a: 1.0,
}; // #5e81ac

// Aurora (accent colors)
pub const NORD11: Color = Color {
    r: 0.75,
    g: 0.38,
    b: 0.42,
    a: 1.0,
}; // #bf616a (red)
pub const NORD12: Color = Color {
    r: 0.82,
    g: 0.53,
    b: 0.44,
    a: 1.0,
}; // #d08770 (orange)
pub const NORD13: Color = Color {
    r: 0.92,
    g: 0.80,
    b: 0.55,
    a: 1.0,
}; // #ebcb8b (yellow)
pub const NORD14: Color = Color {
    r: 0.64,
    g: 0.75,
    b: 0.55,
    a: 1.0,
}; // #a3be8c (green)
pub const NORD15: Color = Color {
    r: 0.71,
    g: 0.56,
    b: 0.68,
    a: 1.0,
}; // #b48ead (purple)

// Semantic color mapping
pub const COLOR_LIGHT: Color = NORD6; // Snow storm lightest
pub const COLOR_PRIMARY: Color = NORD10; // Frost blue
pub const COLOR_SECONDARY: Color = NORD15; // Aurora purple
pub const COLOR_SUCCESS: Color = NORD14; // Aurora green
pub const COLOR_DANGER: Color = NORD11; // Aurora red
pub const COLOR_BLACK: Color = NORD0; // Polar night darkest

pub struct State {
    pub width: u32,
    pub height: u32,
    pub clay: Clay,
    pub clicked_sidebar_item: i32,
    pub click_counter: u32,
    pub mouse_left_down: bool,
    pub mouse_left_was_down: bool,
    pub mouse_x: f64,
    pub mouse_y: f64,
}

impl State {
    pub fn draw_and_render(&mut self, clay_renderer: &mut ClayRenderer) {
        let mut clay_scope = self.clay.begin::<(), ()>();

        // Outer container
        clay_scope.with(
            &Declaration::new()
                .id(clay_scope.id("OuterContainer"))
                .layout()
                .width(Sizing::Percent(1.0))
                .height(Sizing::Percent(1.0))
                .padding(Padding::all(16))
                .child_gap(16)
                .end()
                .background_color(NORD4.into()),
            |outer| {
                // Sidebar
                outer.with(
                    &Declaration::new()
                        .id(outer.id("SideBar"))
                        .layout()
                        .direction(LayoutDirection::TopToBottom)
                        .width(Sizing::Fixed(300.0))
                        .height(Sizing::Percent(1.0))
                        .padding(Padding::all(16))
                        .child_gap(16)
                        .end()
                        .background_color(COLOR_LIGHT.into()),
                    |sidebar| {
                        // Profile section
                        sidebar.with(
                            &Declaration::new()
                                .id(sidebar.id("ProfileSection"))
                                .layout()
                                .width(Sizing::Percent(1.0))
                                .padding(Padding::all(16))
                                .child_gap(16)
                                .end()
                                .background_color(COLOR_DANGER.into())
                                .corner_radius()
                                .all(6.0)
                                .end(),
                            |profile| {
                                profile.text(
                                    "Clay - UI Library",
                                    TextConfig::new()
                                        .font_size(16)
                                        .color(COLOR_BLACK.into())
                                        .end(),
                                );
                            },
                        );

                        // Cycle through items on any click
                        if !self.mouse_left_down && self.mouse_left_was_down {
                            self.clicked_sidebar_item = (self.clicked_sidebar_item + 1) % 5;
                        }

                        // Menu items
                        for i in 0..5 {
                            let bg_color = if self.clicked_sidebar_item == i {
                                COLOR_SECONDARY
                            } else {
                                COLOR_PRIMARY
                            };

                            let item_id_str = match i {
                                0 => "MenuItem0",
                                1 => "MenuItem1", 
                                2 => "MenuItem2",
                                3 => "MenuItem3",
                                4 => "MenuItem4",
                                _ => "MenuItem",
                            };

                            sidebar.with(
                                &Declaration::new()
                                    .id(sidebar.id(item_id_str))
                                    .layout()
                                    .width(Sizing::Percent(1.0))
                                    .height(Sizing::Fixed(50.0))
                                    .padding(Padding::all(16))
                                    .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                                    .end()
                                    .background_color(bg_color.into())
                                    .border()
                                    .all_directions(4)
                                    .color(COLOR_BLACK.into())
                                    .end(),
                                |item| {
                                    let text = if self.clicked_sidebar_item == i {
                                        "Selected Item"
                                    } else {
                                        "Menu Item"
                                    };

                                    item.text(
                                        text,
                                        TextConfig::new()
                                            .font_size(14)
                                            .color(COLOR_BLACK.into())
                                            .end(),
                                    );
                                },
                            );
                        }
                    },
                );

                // Main content area
                outer.with(
                    &Declaration::new()
                        .id(outer.id("MainContent"))
                        .layout()
                        .width(Sizing::Percent(1.0))
                        .height(Sizing::Percent(1.0))
                        .padding(Padding::all(16))
                        .child_gap(16)
                        .direction(LayoutDirection::TopToBottom)
                        .end()
                        .background_color(COLOR_LIGHT.into()),
                    |main| {
                        main.text(
                            "Welcome to the Clay UI Layout Demo!",
                            TextConfig::new()
                                .font_size(24)
                                .color(COLOR_BLACK.into())
                                .end(),
                        );

                        main.text(
                            "This layout demonstrates the adaptation of your Odin Clay UI code to Rust.",
                            TextConfig::new()
                                .font_size(18)
                                .color(COLOR_DANGER.into())
                                .end(),
                        );

                        main.text(
                            "The sidebar has a fixed width while the main content area is responsive.",
                            TextConfig::new()
                                .font_size(18)
                                .color(COLOR_PRIMARY.into())
                                .end(),
                        );

                        // Interactive button
                        main.with(
                            &Declaration::new()
                                .id(main.id("ClickButton"))
                                .layout()
                                .width(Sizing::Fixed(200.0))
                                .height(Sizing::Fixed(50.0))
                                .padding(Padding::all(12))
                                .child_alignment(Alignment::new(LayoutAlignmentX::Center, LayoutAlignmentY::Center))
                                .end()
                                .background_color(COLOR_SUCCESS.into())
                                .corner_radius()
                                .all(8.0)
                                .end()
                                .border()
                                .all_directions(2)
                                .color(COLOR_BLACK.into())
                                .end(),
                            |button| {
                                if !self.mouse_left_down && self.mouse_left_was_down {
                                    self.click_counter += 1;
                                }

                                button.text(
                                    "Click Me!",
                                    TextConfig::new()
                                        .font_size(16)
                                        .color(COLOR_BLACK.into())
                                        .end(),
                                );
                            },
                        );

                        main.text(
                            "Clipping Demo:",
                            TextConfig::new()
                                .font_size(18)
                                .color(COLOR_BLACK.into())
                                .end(),
                        );

                        // Clipped container demo
                        main.with(
                            &Declaration::new()
                                .id(main.id("ClippedBox"))
                                .layout()
                                .width(Sizing::Fixed(300.0))
                                .height(Sizing::Fixed(150.0))
                                .padding(Padding::all(8))
                                .child_gap(8)
                                .direction(LayoutDirection::TopToBottom)
                                .end()
                                .background_color(NORD5.into())
                                .corner_radius()
                                .all(4.0)
                                .end()
                                .border()
                                .all_directions(2)
                                .color(COLOR_BLACK.into())
                                .end()
                                .clip(true, true, (0.0, 0.0).into()),
                            |clipped| {
                                clipped.text(
                                    "This content is clipped!",
                                    TextConfig::new()
                                        .font_size(16)
                                        .color(COLOR_DANGER.into())
                                        .end(),
                                );

                                for _i in 0..8 {
                                    clipped.text(
                                        "This is a long line that should overflow and be clipped by the container bounds",
                                        TextConfig::new()
                                            .font_size(14)
                                            .color(COLOR_PRIMARY.into())
                                            .end(),
                                    );
                                }
                            },
                        );
                    },
                );
            },
        );

        let render_commands: Vec<_> = clay_scope.end().collect();
        clay_renderer.render_commands(render_commands);
    }
}
