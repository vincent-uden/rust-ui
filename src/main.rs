use std::path::PathBuf;

use glfw;
use glfw::Context;

use crate::geometry::{Rect, Vector};
use crate::render::rect::RectRenderer;
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

// TRY TO USE A SHADER

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let mut state = State {
        width: 1000,
        height: 800,
    };

    let (mut window, events) = glfw
        .create_window(state.width, state.height, "App", glfw::WindowMode::Windowed)
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

    let rect_shader = Shader::from_paths(
        &PathBuf::from("./shaders/rounded_rect.vs"),
        &PathBuf::from("./shaders/rounded_rect.frag"),
        None,
    )
    .unwrap();
    let rect_renderer = RectRenderer::new(rect_shader);

    while !window.should_close() {
        glfw.poll_events();

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        rect_renderer.draw(
            Rect {
                x0: Vector::new(100.0, 100.0),
                x1: Vector::new(200.0, 100.0),
            },
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            Border {
                thickness: 4.0,
                radius: BorderRadius {
                    top_left: 8.0,
                    top_right: 8.0,
                    bottom_left: 8.0,
                    bottom_right: 8.0,
                },
            },
            1.0,
        );

        window.swap_buffers();
    }

    // TODO: callbacks
}
