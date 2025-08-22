use std::path::PathBuf;

use glfw;
use glfw::Context;

use crate::geometry::{Rect, Vector};
use crate::render::clay::ClayRenderer;
use crate::render::rect::RectRenderer;
use crate::render::text::TextRenderer;
use crate::render::{Border, BorderRadius, Color};
use crate::shader::Shader;

mod geometry;
mod render;
mod shader;

#[derive(Debug)]
pub struct State {
    pub width: u32,
    pub height: u32,
}

fn init_open_gl(inital_state: &State) -> (glfw::Glfw, glfw::PWindow) {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let (mut window, _events) = glfw
        .create_window(
            inital_state.width,
            inital_state.height,
            "App",
            glfw::WindowMode::Windowed,
        )
        .unwrap();

    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|ptr| {
        let f = window.get_proc_address(ptr);
        match f {
            Some(f) => f as *const _,
            None => std::ptr::null(),
        }
    });

    unsafe {
        gl::Viewport(0, 0, inital_state.width as i32, inital_state.height as i32);
        gl::Enable(gl::BLEND);
        gl::Enable(gl::MULTISAMPLE);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    (glfw, window)
}

fn main() {
    let state = State {
        width: 1000,
        height: 800,
    };
    let (mut glfw, mut window) = init_open_gl(&state);

    let rect_shader = Shader::from_paths(
        &PathBuf::from("./shaders/rounded_rect.vs"),
        &PathBuf::from("./shaders/rounded_rect.frag"),
        None,
    )
    .unwrap();

    let text_shader = Shader::from_paths(
        &PathBuf::from("./shaders/text.vs"),
        &PathBuf::from("./shaders/text.frag"),
        None,
    )
    .unwrap();

    // Set up projection matrix for 2D rendering
    let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

    rect_shader.use_shader();
    rect_shader.set_uniform("projection", &projection);

    text_shader.use_shader();
    text_shader.set_uniform("projection", &projection);

    let rect_renderer = RectRenderer::new(rect_shader);
    let mut text_renderer = TextRenderer::new(
        text_shader,
        &PathBuf::from("./assets/fonts/LiberationMono.ttf"),
    )
    .unwrap();

    let mut clay_renderer = ClayRenderer::new(
        rect_shader,
        text_shader,
        state.width as f32,
        state.height as f32,
    );

    while !window.should_close() {
        glfw.poll_events();

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Draw text below the rectangle
        // text_renderer.draw_text(
        //     "Hello, Rust UI!",
        //     Vector::new(150.0, 250.0), // Position below the rectangle
        //     25,                        // Font size
        //     1.0,                       // Scale
        //     [1.0, 1.0, 1.0],           // White color
        // );
u
        clay_renderer.draw();

        window.swap_buffers();
    }

    // TODO: callbacks

    unsafe {
        gl::Flush();
        gl::Finish();
    }
    glfw::make_context_current(None);
    // Segfaults due to bug in glfw with wayland
    std::mem::forget(window);
}
