// use std::path::PathBuf;
// use std::sync::{Arc, Mutex};

// use clay_layout::ClayLayoutScope;
// use clay_layout::layout::{Padding, Sizing};
// use clay_layout::{
//     Clay, Declaration, fixed, math::Dimensions, render_commands::RenderCommandConfig,
//     text::TextConfig,
// };

// use crate::state::EventListeners;
// use crate::{
//     geometry::{Rect, Vector},
//     render::{Border, BorderRadius, Color, rect::RectRenderer, text::TextRenderer},
//     shader::Shader,
// };

// pub struct ClayRenderer {
//     rect_r: RectRenderer,
//     // Using an Arc<Mutex<_>> here allows the measurements and drawing to share the same glyph
//     // cache.
//     pub text_r: Arc<Mutex<TextRenderer>>,
//     window_height: i32,
// }

// impl ClayRenderer {
//     pub fn new(rect_shader: Shader, text_shader: Shader, screen_height: f32) -> Self {
//         let text_r = Arc::new(Mutex::new(
//             TextRenderer::new(
//                 text_shader,
//                 &PathBuf::from("./assets/fonts/LiberationMono.ttf"),
//             )
//             .unwrap(),
//         ));

//         Self {
//             rect_r: RectRenderer::new(rect_shader),
//             text_r,
//             window_height: screen_height as i32,
//         }
//     }

//     pub fn render_commands(
//         &mut self,
//         render_commands: Vec<clay_layout::render_commands::RenderCommand<(), EventListeners>>,
//     ) {
//         for command in render_commands {
//             match &command.config {
//                 RenderCommandConfig::None() => {}
//                 RenderCommandConfig::Rectangle(rectangle) => {
//                     self.rect_r.draw(
//                         command.bounding_box.into(),
//                         rectangle.color.into(),
//                         Color::default(),
//                         Border {
//                             thickness: 0.,
//                             radius: rectangle.corner_radii.clone().into(),
//                         },
//                         1.0,
//                     );
//                 }
//                 RenderCommandConfig::Border(border) => {
//                     self.rect_r.draw(
//                         command.bounding_box.into(),
//                         Color::default(),
//                         border.color.into(),
//                         Border {
//                             // I dont want variable border width around a box
//                             thickness: border.width.top as f32,
//                             radius: border.corner_radii.clone().into(),
//                         },
//                         1.0,
//                     );
//                 }
//                 RenderCommandConfig::Text(text) => {
//                     if let Ok(mut text_renderer) = self.text_r.lock() {
//                         text_renderer.draw_text(
//                             text.text,
//                             Vector {
//                                 x: command.bounding_box.x,
//                                 y: command.bounding_box.y + command.bounding_box.height * 0.8,
//                             },
//                             text.font_size.into(),
//                             1.0,
//                             text.color.into(),
//                         );
//                     }
//                 }
//                 RenderCommandConfig::Image(_image) => todo!(),
//                 RenderCommandConfig::ScissorStart() => {
//                     self.rect_r.push_scissor_region(
//                         command.bounding_box.x,
//                         command.bounding_box.y,
//                         command.bounding_box.width,
//                         command.bounding_box.height,
//                         self.window_height,
//                     );
//                 }
//                 RenderCommandConfig::ScissorEnd() => {
//                     self.rect_r.pop_scissor_region();
//                 }
//                 RenderCommandConfig::Custom(_custom) => todo!(),
//             }
//         }
//     }

//     pub fn window_size(&mut self, size: (i32, i32)) {
//         self.window_height = size.1;
//     }
// }
