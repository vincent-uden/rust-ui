use clay_layout::{
    Clay, Declaration,
    layout::{Padding, Sizing},
    text::TextConfig,
};

use crate::render::{Color, clay::ClayRenderer};

pub struct State {
    pub width: u32,
    pub height: u32,
    pub clay: Clay,
}

impl State {
    pub fn draw_and_render(&mut self, clay_renderer: &mut ClayRenderer) {
        let mut clay_scope = self.clay.begin::<(), ()>();
        clay_scope.with(
            &Declaration::new()
                .id(clay_scope.id("red_rectangle"))
                .layout()
                .width(Sizing::Percent(1.0))
                .height(Sizing::Percent(1.0))
                .padding(Padding::all(24))
                .end()
                .corner_radius()
                .all(5.)
                .end()
                .background_color(
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }
                    .into(),
                )
                .border()
                .all_directions(10)
                .color((0x00, 0xFF, 0x00).into())
                .end(),
            |parent| {
                parent.text(
                    "Hello, world!",
                    TextConfig::new()
                        .font_size(16)
                        .color((0xff, 0xff, 0xff).into())
                        .end(),
                )
            },
        );
        let render_commands: Vec<_> = clay_scope.end().collect();
        clay_renderer.render_commands(render_commands);
    }
}
