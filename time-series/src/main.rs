use std::{path::PathBuf, str::FromStr, time::Duration};

use glfw::Context;
use rust_ui::{
    init_open_gl,
    render::{
        line::LineRenderer,
        rect::RectRenderer,
        renderer::Renderer,
        sprite::{SpriteAtlas, SpriteRenderer},
        text::TextRenderer,
    },
    shader::{Shader, ShaderName},
};
use tracing_subscriber::EnvFilter;

use crate::app::App;

mod app;

const TARGET_FPS: u64 = 60;
const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(EnvFilter::new("time_series"))
        .init();

    let (mut glfw, mut window, events) = init_open_gl(1000, 800, true, true);

    let rect_shader = Shader::new_from_name(&ShaderName::Rect).unwrap();
    let text_shader = Shader::new_from_name(&ShaderName::Text).unwrap();
    let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();
    let sprite_shader = Shader::new_from_name(&ShaderName::Sprite).unwrap();

    let rect_r = RectRenderer::new(rect_shader);
    let text_r = TextRenderer::new(
        text_shader,
        &PathBuf::from_str("assets/fonts/LiberationMono.ttf").unwrap(),
    )
    .unwrap();
    let line_r = LineRenderer::new(line_shader);
    let sprite_r = SpriteRenderer::new(Shader::empty(), SpriteAtlas::empty());

    let mut state = Renderer::new(rect_r, text_r, line_r, sprite_r, App::new());

    // Set up projection matrix for 2D rendering
    let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

    rect_shader.use_shader();
    rect_shader.set_uniform("projection", &projection);

    text_shader.use_shader();
    text_shader.set_uniform("projection", &projection);

    sprite_shader.use_shader();
    sprite_shader.set_uniform("projection", &projection);

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, action, _) => {
                    state.mouse_left_down =
                        action == glfw::Action::Press || action == glfw::Action::Repeat;
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    state.last_mouse_pos.x = state.mouse_pos.x;
                    state.last_mouse_pos.y = state.mouse_pos.y;
                    state.mouse_pos.x = x as f32;
                    state.mouse_pos.y = y as f32;
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    state.window_size((width, height));
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
                    state.handle_key(key, scancode, action, modifiers);
                }
                _ => {}
            }
        }
        state.update();
        let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

        rect_shader.use_shader();
        rect_shader.set_uniform("projection", &projection);

        text_shader.use_shader();
        text_shader.set_uniform("projection", &projection);

        sprite_shader.use_shader();
        sprite_shader.set_uniform("projection", &projection);
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        state.render();
        window.swap_buffers();
    }

    unsafe {
        gl::Flush();
        gl::Finish();
    }
    glfw::make_context_current(None);
    // Segfaults due to bug in glfw with wayland
    std::mem::forget(window);
}
