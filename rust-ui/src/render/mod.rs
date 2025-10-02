pub mod line;
pub mod mesh;
pub mod point;
pub mod rect;
pub mod renderer;
pub mod sprite;
pub mod text;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl BorderRadius {
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Border {
    pub thickness: f32,
    pub radius: BorderRadius,
    pub color: Color,
}

impl Border {
    pub fn debug() -> Self {
        Self {
            thickness: 2.0,
            radius: BorderRadius::default(),
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub font_size: u32,
    pub color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: Default::default(),
            font_size: 12,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

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
