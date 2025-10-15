#![allow(clippy::uninlined_format_args)]

use std::{
    io,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

use glfw::Context as _;
use rust_ui::{
    geometry::Vector,
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
use sysinfo::{ProcessesToUpdate, System};
use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::app::App;

mod app;
mod entity_picker;
mod sketch_renderer;
mod ui;

pub const TARGET_FPS: u64 = 60;
pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_env_filter(EnvFilter::new("cad_frontend,rust_ui"))
        .init();

    let (mut glfw, mut window, events) = init_open_gl(1000, 800, false);

    let rect_shader = Shader::new_from_name(&ShaderName::Rect).unwrap();
    let text_shader = Shader::new_from_name(&ShaderName::Text).unwrap();
    let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();
    let sprite_shader = Shader::new_from_name(&ShaderName::Sprite).unwrap();
    let sprite_atlas = SpriteAtlas::from_path(
        &PathBuf::from_str("assets/atlas/icons.png").unwrap(),
        &PathBuf::from_str("assets/atlas/icons.csv").unwrap(),
    )
    .unwrap();

    let rect_r = RectRenderer::new(rect_shader);
    let text_r = TextRenderer::new(
        text_shader,
        &PathBuf::from("./assets/fonts/LiberationMono.ttf"),
    )
    .unwrap();
    let line_r = LineRenderer::new(line_shader);
    let sprite_r = SpriteRenderer::new(sprite_shader, sprite_atlas);

    let mut state = Renderer::new(rect_r, text_r, line_r, sprite_r, App::default());

    // Set up projection matrix for 2D rendering
    let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);

    rect_shader.use_shader();
    rect_shader.set_uniform("projection", &projection);
    text_shader.use_shader();
    text_shader.set_uniform("projection", &projection);
    sprite_shader.use_shader();
    sprite_shader.set_uniform("projection", &projection);

    // Perf stats
    let mut sleep_time_accumulator = Duration::ZERO;
    let mut frame_count = 0u64;
    let mut last_log_time = Instant::now();
    let mut avg_sleep_ms = 0.0;
    let mut sys = System::new_all();
    let pid = sysinfo::get_current_pid().unwrap();
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    let mut ram_usage = sys.process(pid).unwrap().memory();

    let debug_renderer = LineRenderer::new(line_shader);

    // Defer the direct handling of some of these events until we know they weren't handled by UI
    let mut window_events = vec![];

    while !window.should_close() {
        let frame_start = Instant::now();

        glfw.poll_events();

        state.pre_update();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Scroll(x, y) => {
                    if x.abs() > 0.01 || y.abs() > 0.01 {
                        state.handle_mouse_scroll(Vector::new(x as f32, y as f32));
                        window_events.push(event);
                    }
                }
                glfw::WindowEvent::MouseButton(button, action, modifiers) => {
                    state.handle_mouse_button(button, action, modifiers);
                    window_events.push(event);
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    state.handle_mouse_position(Vector::new(x as f32, y as f32));
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    let new_size = Vector::new(width as f32, height as f32);
                    state.app_state.resize_areas(new_size);
                    state.window_size((width, height));
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
                    state.handle_key(key, scancode, action, modifiers);
                    match key {
                        glfw::Key::F12 => match action {
                            glfw::Action::Release => {
                                state.show_debug_layer = !state.show_debug_layer;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        state.update();
        state.app_state.perf_overlay.update(avg_sleep_ms, ram_usage);

        state.app_state.handle_area_events(&window_events, state.mouse_hit_layer);
        window_events.clear();

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
        state.app_state.update_areas();
        state.app_state.draw_special_areas();
        state.render();
        if state.app_state.debug_draw {
            state.app_state.debug_draw(
                &debug_renderer,
                Vector::new(state.width as f32, state.height as f32),
            )
        }

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
