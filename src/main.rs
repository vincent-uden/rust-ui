use std::io;
use std::path::PathBuf;

use clay_layout::Clay;
use glfw;
use glfw::Context;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::render::clay::ClayRenderer;

use crate::shader::Shader;
use crate::state::State;

mod geometry;
mod render;
mod shader;
mod state;

fn init_open_gl(
    inital_state: &State,
) -> (
    glfw::Glfw,
    glfw::PWindow,
    glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let (mut window, events) = glfw
        .create_window(
            inital_state.width,
            inital_state.height,
            "App",
            glfw::WindowMode::Windowed,
        )
        .unwrap();

    window.make_current();
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);

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

    (glfw, window, events)
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_env_filter(EnvFilter::new("rust_ui"))
        .init();

    let mut state = State {
        width: 1000,
        height: 800,
        clay: Clay::new((1000.0, 800.0).into()),
        clicked_sidebar_item: -1,
        click_counter: 0,
        mouse_left_down: false,
        mouse_left_was_down: false,
        mouse_x: 0.0,
        mouse_y: 0.0,
        button_text_buffer: String::with_capacity(20),
    };
    let (mut glfw, mut window, events) = init_open_gl(&state);

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

        state.mouse_left_was_down = state.mouse_left_down;
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, action, _) => {
                    state.mouse_left_down =
                        action == glfw::Action::Press || action == glfw::Action::Repeat;
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    state.mouse_x = x;
                    state.mouse_y = y;
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    info!("Width {width}, {height}");
                    state.window_size((width, height));
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                    clay_renderer.window_size((width, height));
                }
                _ => {}
            }
        }

        state.clay.pointer_state(
            (state.mouse_x as f32, state.mouse_y as f32).into(),
            state.mouse_left_down,
        );
        let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

        rect_shader.use_shader();
        rect_shader.set_uniform("projection", &projection);

        text_shader.use_shader();
        text_shader.set_uniform("projection", &projection);

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
