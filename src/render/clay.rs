use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clay_layout::layout::{Padding, Sizing};
use clay_layout::{
    Clay, Declaration, fixed, math::Dimensions, render_commands::RenderCommandConfig,
    text::TextConfig,
};

use crate::{
    geometry::{Rect, Vector},
    render::{
        Border, BorderRadius, Color,
        rect::{RectRenderer, ScissorRegion},
        text::TextRenderer,
    },
    shader::Shader,
};

pub struct ClayRenderer {
    rect_r: RectRenderer,
    // Using an Arc<Mutex<_>> here allows the measurements and drawing to share the same glyph
    // cache.
    text_r: Arc<Mutex<TextRenderer>>,
    clay: Clay,
    scissor_stack: Vec<ScissorRegion>,
}

impl ClayRenderer {
    pub fn new(
        rect_shader: Shader,
        text_shader: Shader,
        screen_width: f32,
        screen_height: f32,
    ) -> Self {
        let text_r = Arc::new(Mutex::new(
            TextRenderer::new(
                text_shader,
                &PathBuf::from("./assets/fonts/LiberationMono.ttf"),
            )
            .unwrap(),
        ));

        let mut clay = Clay::new((screen_width, screen_height).into());

        let text_r_clone = Arc::clone(&text_r);
        clay.set_measure_text_function(move |text, config| {
            if text.is_empty() || config.font_size == 0 {
                return Dimensions {
                    width: 0.0,
                    height: config.font_size as f32,
                };
            }
            let mut text_renderer = text_r_clone
                .lock()
                .expect("The TextRenderer mutex should never be poisoned");
            let size = text_renderer.measure_text_size(text, config.font_size as u32);
            Dimensions {
                width: size.x,
                height: size.y,
            }
        });

        Self {
            rect_r: RectRenderer::new(rect_shader),
            text_r,
            clay,
            scissor_stack: vec![],
        }
    }

    pub fn draw(&mut self) {
        let mut clay = self.clay.begin::<(), ()>();

        clay.with(
            &Declaration::new()
                .id(clay.id("red_rectangle"))
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

        let render_commands = clay.end();

        for command in render_commands {
            match &command.config {
                RenderCommandConfig::None() => {}
                RenderCommandConfig::Rectangle(rectangle) => {
                    self.rect_r.draw(
                        command.bounding_box.into(),
                        rectangle.color.into(),
                        Color::default(),
                        Border {
                            thickness: 0.,
                            radius: rectangle.corner_radii.clone().into(),
                        },
                        1.0,
                    );
                }
                RenderCommandConfig::Border(border) => {
                    self.rect_r.draw(
                        command.bounding_box.into(),
                        Color::default(),
                        border.color.into(),
                        Border {
                            // I dont want variable border width around a box
                            thickness: border.width.top as f32,
                            radius: border.corner_radii.clone().into(),
                        },
                        1.0,
                    );
                }
                RenderCommandConfig::Text(text) => {
                    if let Ok(mut text_renderer) = self.text_r.lock() {
                        text_renderer.draw_text(
                            text.text,
                            Vector {
                                x: command.bounding_box.x,
                                y: command.bounding_box.y + command.bounding_box.height * 0.8,
                            },
                            text.font_size.into(),
                            1.0,
                            text.color.into(),
                        );
                    }
                }
                RenderCommandConfig::Image(image) => todo!(),
                RenderCommandConfig::ScissorStart() => todo!(),
                RenderCommandConfig::ScissorEnd() => todo!(),
                RenderCommandConfig::Custom(custom) => todo!(),
            }
        }
    }
}
