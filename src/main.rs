use std::path::PathBuf;

use clay_layout::layout::{Padding, Sizing};

use clay_layout::text::TextConfig;
use clay_layout::{Clay, Declaration};
use glfw;
use glfw::Context;

use crate::render::clay::ClayRenderer;

use crate::render::Color;
use crate::shader::Shader;

mod geometry;
mod render;
mod shader;

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
    let mut state = State {
        width: 1000,
        height: 800,
        clay: Clay::new((1000.0, 800.0).into()),
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

    let mut clay_renderer = ClayRenderer::new(rect_shader, text_shader, state.height as f32);

    let text_renderer = clay_renderer.text_r.clone();
    state.clay.set_measure_text_function(move |text, config| {
        use clay_layout::math::Dimensions;
        if text.is_empty() || config.font_size == 0 {
            return Dimensions {
                width: 0.0,
                height: config.font_size as f32,
            };
        }
        let mut text_renderer = text_renderer
            .lock()
            .expect("The TextRenderer mutex should never be poisoned");
        let size = text_renderer.measure_text_size(text, config.font_size as u32);
        Dimensions {
            width: size.x,
            height: size.y,
        }
    });

    while !window.should_close() {
        glfw.poll_events();

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        state.draw_and_render(&mut clay_renderer);

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
