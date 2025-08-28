#![allow(clippy::uninlined_format_args)]

use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use glfw::Context as _;
use rust_ui::{
    init_open_gl,
    render::{COLOR_LIGHT, renderer::Renderer},
    shader::Shader,
};
use sysinfo::{ProcessesToUpdate, System};
use tracing_subscriber::EnvFilter;

use crate::app::App;

mod app;
mod ui;

pub const TARGET_FPS: u64 = 60;
pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_env_filter(EnvFilter::new("cad_frontend"))
        .init();

    let (mut glfw, mut window, events) = init_open_gl(1000, 800);

    // Select shader directory based on target architecture
    #[cfg(target_arch = "aarch64")]
    let shader_dir = "./shaders/gles300";
    #[cfg(not(target_arch = "aarch64"))]
    let shader_dir = "./shaders/glsl330";

    let rect_shader = Shader::from_paths(
        &PathBuf::from(format!("{}/rounded_rect.vs", shader_dir)),
        &PathBuf::from(format!("{}/rounded_rect.frag", shader_dir)),
        None,
    )
    .unwrap();

    let text_shader = Shader::from_paths(
        &PathBuf::from(format!("{}/text.vs", shader_dir)),
        &PathBuf::from(format!("{}/text.frag", shader_dir)),
        None,
    )
    .unwrap();

    let mut state = Renderer::new(rect_shader, text_shader, App::default());

    // Set up projection matrix for 2D rendering
    let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

    rect_shader.use_shader();
    rect_shader.set_uniform("projection", &projection);

    text_shader.use_shader();
    text_shader.set_uniform("projection", &projection);

    // Perf stats
    let mut sleep_time_accumulator = Duration::ZERO;
    let mut frame_count = 0u64;
    let mut last_log_time = Instant::now();
    let mut avg_sleep_ms = 0.0;
    let mut sys = System::new_all();
    let pid = sysinfo::get_current_pid().unwrap();
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    let mut ram_usage = sys.process(pid).unwrap().memory();

    while !window.should_close() {
        let frame_start = Instant::now();

        glfw.poll_events();

        state.mouse_left_was_down = state.mouse_left_down;
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
                    state.app_state.mouse_pos.x = x as f32;
                    state.app_state.mouse_pos.y = y as f32;
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
        state.app_state.perf_overlay.update(avg_sleep_ms, ram_usage);

        let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

        rect_shader.use_shader();
        rect_shader.set_uniform("projection", &projection);

        text_shader.use_shader();
        text_shader.set_uniform("projection", &projection);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        state.compute_layout_and_render();

        window.swap_buffers();

        let frame_time = frame_start.elapsed();
        let sleep_duration = if frame_time < FRAME_TIME {
            let sleep_time = FRAME_TIME - frame_time;
            std::thread::sleep(sleep_time);
            sleep_time
        } else {
            Duration::ZERO
        };

        sleep_time_accumulator += sleep_duration;
        frame_count += 1;

        if last_log_time.elapsed() >= Duration::from_secs(1) {
            avg_sleep_ms = if frame_count > 0 {
                sleep_time_accumulator.as_micros() as f64 / frame_count as f64 / 1000.0
            } else {
                0.0
            };
            sleep_time_accumulator = Duration::ZERO;
            frame_count = 0;
            last_log_time = Instant::now();
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
            ram_usage = sys.process(pid).unwrap().memory();
        }
    }

    unsafe {
        gl::Flush();
        gl::Finish();
    }
    glfw::make_context_current(None);
    // Segfaults due to bug in glfw with wayland
    std::mem::forget(window);
}
