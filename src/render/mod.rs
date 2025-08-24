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

#[derive(Debug)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

#[derive(Debug)]
pub struct Border {
    pub thickness: f32,
    pub radius: BorderRadius,
}
