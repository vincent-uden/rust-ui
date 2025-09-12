#![allow(clippy::uninlined_format_args)]

use std::{
    io,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use glfw::{Action, Context as _, Key, Modifiers, Scancode};
use rust_ui::{
    geometry::Vector,
    init_open_gl,
    render::{
        Border, BorderRadius, COLOR_DANGER, COLOR_LIGHT, COLOR_SUCCESS, Color, Text,
        renderer::{Anchor, AppState, NodeContext, RenderLayout, Renderer, flags},
    },
    shader::{Shader, ShaderName},
};
use sysinfo::{ProcessesToUpdate, System};
use taffy::{
    AvailableSpace, Dimension, FlexDirection, Rect, Size, Style, TaffyTree,
    prelude::{TaffyMaxContent, auto, length},
};
use tracing::info;
use tracing_subscriber::EnvFilter;

pub const TARGET_FPS: u64 = 60;
pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

#[derive(Default)]
struct PerfStats {
    pub header_bg: Color,
    pub visible: bool,
    pub avg_sleep_ms: f64,
    pub ram_usage: u64,
}

impl PerfStats {
    pub fn update(&mut self, avg_sleep_ms: f64, ram_usage: u64) {
        self.avg_sleep_ms = avg_sleep_ms;
        self.ram_usage = ram_usage;
    }

    fn stats_overlay(&mut self, size: rust_ui::geometry::Vector<f32>) -> RenderLayout<Self> {
        let mut tree = TaffyTree::new();

        let title = tree
            .new_leaf_with_context(
                Style {
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Performance stats".into(),
                        font_size: 18,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let frame_time = tree
            .new_leaf_with_context(
                Style::default(),
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: format!(
                            "Frame time: {:.2} ms",
                            FRAME_TIME.as_millis() as f64 - self.avg_sleep_ms
                        ),
                        font_size: 14,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let ram_usage = tree
            .new_leaf_with_context(
                Style::default(),
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: format!("RAM: {:.2} MB", self.ram_usage / 1_000_000,),
                        font_size: 14,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let root = tree
            .new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: Dimension::percent(1.0),
                        height: Dimension::percent(1.0),
                    },
                    gap: Size {
                        width: length(0.0),
                        height: length(8.0),
                    },
                    max_size: size.into(),
                    padding: Rect::length(12.0),
                    ..Default::default()
                },
                NodeContext {
                    bg_color: Color::new(0.0, 0.0, 0.0, 0.5),
                    ..Default::default()
                },
            )
            .unwrap();

        tree.add_child(root, title).unwrap();
        tree.add_child(root, frame_time).unwrap();
        tree.add_child(root, ram_usage).unwrap();

        RenderLayout {
            tree,
            root,
            desired_size: Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MinContent,
            },
            root_pos: Vector::zero(),
            anchor: Anchor::BottomRight,
            ..Default::default()
        }
    }

    fn base_layer(&mut self, window_size: Vector<f32>) -> RenderLayout<Self> {
        let mut tree = TaffyTree::new();
        let header_node = tree
            .new_leaf_with_context(
                Style {
                    padding: Rect::length(20.0),
                    size: Size {
                        width: length(window_size.x),
                        height: length(100.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    bg_color: self.header_bg,
                    border: Border {
                        radius: BorderRadius {
                            bottom_left: 40.0,
                            bottom_right: 40.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    on_mouse_enter: Some(Arc::new(|state: &mut Renderer<Self>| {
                        info!("Entering");
                        state.app_state.header_bg = COLOR_DANGER;
                    })),
                    on_mouse_exit: Some(Arc::new(|state| {
                        info!("Exiting");
                        state.app_state.header_bg = COLOR_LIGHT;
                    })),
                    on_mouse_down: Some(Arc::new(|_| {
                        info!("Mouse down");
                    })),
                    on_mouse_up: Some(Arc::new(|_| {
                        info!("Mouse up");
                    })),
                    ..Default::default()
                },
            )
            .unwrap();

        let header_text = tree
            .new_leaf_with_context(
                Style {
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Flygande bäckasiner söka hwila på mjuka tuvor".into(),
                        font_size: 18,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        tree.add_child(header_node, header_text).unwrap();

        let body_node = tree
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: length(window_size.x),
                        height: auto(),
                    },
                    border: Rect {
                        left: length(40.0),
                        right: length(40.0),
                        top: length(40.0),
                        bottom: length(40.0),
                    },
                    flex_grow: 1.0,
                    ..Default::default()
                },
                NodeContext {
                    bg_color: COLOR_SUCCESS,
                    border: Border {
                        thickness: 20.0,
                        radius: BorderRadius::all(40.0),
                        color: COLOR_LIGHT,
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let root = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(window_size.x),
                        height: length(window_size.y),
                    },
                    gap: Size {
                        width: length(16.0),
                        height: length(16.0),
                    },
                    ..Default::default()
                },
                &[header_node, body_node],
            )
            .unwrap();
        RenderLayout {
            tree,
            root,
            desired_size: Size::MAX_CONTENT,
            root_pos: Vector::zero(),
            anchor: Anchor::TopLeft,
            ..Default::default()
        }
    }
}

impl AppState for PerfStats {
    fn generate_layout(&mut self, window_size: Vector<f32>) -> Vec<RenderLayout<Self>> {
        if self.visible {
            let mut top_left = self.stats_overlay(window_size);
            top_left.anchor = Anchor::TopLeft;
            top_left.scissor = true;
            let mut top_right = self.stats_overlay(window_size);
            top_right.anchor = Anchor::TopRight;
            top_right.scissor = true;
            let mut bottom_left = self.stats_overlay(window_size);
            bottom_left.anchor = Anchor::BottomLeft;
            let mut bottom_right = self.stats_overlay(window_size);
            bottom_right.anchor = Anchor::BottomRight;
            let mut center = self.stats_overlay(window_size);
            center.anchor = Anchor::Center;
            vec![
                self.base_layer(window_size),
                top_left,
                top_right,
                bottom_left,
                bottom_right,
                center,
            ]
        } else {
            vec![self.base_layer(window_size)]
        }
    }

    fn handle_key(&mut self, key: Key, _scancode: Scancode, action: Action, _modifiers: Modifiers) {
        #[allow(clippy::single_match)]
        match key {
            Key::F12 => match action {
                Action::Release => {
                    self.visible = !self.visible;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_env_filter(EnvFilter::new("demo"))
        .init();

    let (mut glfw, mut window, events) = init_open_gl(1000, 800);

    // Select shader directory based on target architecture
    #[cfg(target_arch = "aarch64")]
    let shader_dir = "./shaders/gles300";
    #[cfg(not(target_arch = "aarch64"))]
    let shader_dir = "./shaders/glsl330";

    let rect_shader = Shader::new_from_name(&ShaderName::Rect).unwrap();

    let text_shader = Shader::new_from_name(&ShaderName::Text).unwrap();

    let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();

    let mut state = Renderer::new(
        rect_shader,
        text_shader,
        line_shader,
        PerfStats {
            header_bg: COLOR_LIGHT,
            ..Default::default()
        },
    );

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
        state.app_state.update(avg_sleep_ms, ram_usage);

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

        // Demonstrate line rendering
        use rust_ui::{geometry::Vector, render::{COLOR_DANGER, COLOR_SUCCESS, COLOR_PRIMARY}};
        let window_size = Vector::new(state.width as f32, state.height as f32);
        
        // Draw some sample lines
        state.line_r.draw(
            Vector::new(50.0, 50.0),
            Vector::new(200.0, 100.0),
            COLOR_DANGER,
            2.0,
            window_size,
        );
        
        state.line_r.draw(
            Vector::new(50.0, 120.0),
            Vector::new(300.0, 120.0),
            COLOR_SUCCESS,
            3.0,
            window_size,
        );
        
        state.line_r.draw(
            Vector::new(100.0, 150.0),
            Vector::new(100.0, 250.0),
            COLOR_PRIMARY,
            1.5,
            window_size,
        );

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
