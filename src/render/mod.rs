pub mod clay;
pub mod rect;
pub mod text;

#[derive(Debug, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Into<Color> for clay_layout::Color {
    fn into(self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

#[derive(Debug)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl Into<BorderRadius> for clay_layout::render_commands::CornerRadii {
    fn into(self) -> BorderRadius {
        BorderRadius {
            top_left: self.top_left,
            top_right: self.top_right,
            bottom_left: self.bottom_left,
            bottom_right: self.bottom_right,
        }
    }
}

#[derive(Debug)]
pub struct Border {
    pub thickness: f32,
    pub radius: BorderRadius,
}
