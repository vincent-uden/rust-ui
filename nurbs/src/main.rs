use std::{
    f64::consts::FRAC_PI_2,
    io,
    path::PathBuf,
    str::FromStr as _,
    sync::Arc,
    time::{Duration, Instant},
};

use curvo::prelude::{
    AdaptiveTessellationOptions, NurbsCurve3D, NurbsSurface, SurfaceTessellation3D,
    Tessellation as _, Transformable as _,
};
use glfw::{Action, Context as _, Key, Modifiers, Scancode};
use nalgebra::{Point3, Rotation3, Translation3, Vector3};
use rust_ui::{
    geometry::Vector,
    init_open_gl,
    render::{
        Border, BorderRadius, COLOR_DANGER, COLOR_LIGHT, COLOR_SUCCESS, Color, Text,
        line::LineRenderer,
        mesh::{MeshRenderer, Vertex},
        rect::RectRenderer,
        renderer::{Anchor, AppState, NodeContext, RenderLayout, Renderer, flags},
        sprite::{SpriteAtlas, SpriteRenderer},
        text::TextRenderer,
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
                    on_left_mouse_down: Some(Arc::new(|_| {
                        info!("Mouse down");
                    })),
                    on_left_mouse_up: Some(Arc::new(|_| {
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
            let mut top_right = self.stats_overlay(window_size);
            top_right.anchor = Anchor::TopRight;
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

fn generate_curve() -> (Vec<Vertex>, Vec<u32>) {
    let points = vec![
        Point3::new(-1.0, -1.0, 0.),
        Point3::new(1.0, -1.0, 0.),
        Point3::new(1.0, 1.0, 0.),
        Point3::new(-1.0, 1.0, 0.),
    ];

    // Create a NURBS curve that interpolates the given points with degree 3
    // You can also specify the precision of the curve by generic type (f32 or f64)
    let interpolated = NurbsCurve3D::<f64>::try_interpolate(&points, 3).unwrap();

    // NURBS curve & surface can be transformed by nalgebra's matrix
    // let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), FRAC_PI_2);
    let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), FRAC_PI_2 * 1.0);
    let translation = Translation3::new(0., 0., 3.);
    let transform_matrix = translation * rotation; // nalgebra::Isometry3

    // Transform the curve by the given matrix (nalgebra::Isometry3 into nalgebra::Matrix4)
    let offsetted = interpolated.transformed(&transform_matrix.into());

    // Create a NURBS surface by lofting two NURBS curves
    let lofted = NurbsSurface::try_loft(
        &[interpolated, offsetted],
        Some(3), // degree of v direction
    )
    .unwrap();

    // Tessellate the surface in adaptive manner about curvature for efficient rendering
    let option = AdaptiveTessellationOptions {
        norm_tolerance: 1e-2,
        ..Default::default()
    };
    let tessellation = lofted.tessellate(Some(option));
    to_vertices(&tessellation)
}

fn to_vertices(tesselation: &SurfaceTessellation3D<f64>) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = vec![];
    let mut indicies = vec![];

    for (p, n) in tesselation.points().iter().zip(tesselation.normals()) {
        vertices.push(Vertex {
            position: [p.x as f32, p.y as f32, p.z as f32],
            normal: [n.x as f32, n.y as f32, n.z as f32],
        });
    }

    // I think these are the indices
    for index in tesselation.faces() {
        indicies.push(index[0] as u32);
        indicies.push(index[1] as u32);
        indicies.push(index[2] as u32);
    }

    (vertices, indicies)
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_env_filter(EnvFilter::new("nurbs,rust_ui"))
        .init();

    let (mut glfw, mut window, events) = init_open_gl(1000, 800, true);

    let rect_shader = Shader::new_from_name(&ShaderName::Rect).unwrap();
    let text_shader = Shader::new_from_name(&ShaderName::Text).unwrap();
    let mesh_shader = Shader::new_from_name(&ShaderName::Mesh).unwrap();
    let (vertices, indices) = generate_curve();
    let mesh_r = MeshRenderer::new(vertices, indices, mesh_shader);

    let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();

    let rect_r = RectRenderer::new(rect_shader);
    let text_r = TextRenderer::new(
        text_shader,
        &PathBuf::from_str("assets/fonts/LiberationMono.ttf").unwrap(),
    )
    .unwrap();
    let line_r = LineRenderer::new(line_shader);
    let sprite_r = SpriteRenderer::new(Shader::empty(), SpriteAtlas::empty());

    let mut state = Renderer::new(
        rect_r,
        text_r,
        line_r,
        sprite_r,
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

    generate_curve();

    // Perf stats
    let mut sleep_time_accumulator = Duration::ZERO;
    let mut frame_count = 0u64;
    let mut last_log_time = Instant::now();
    let mut avg_sleep_ms = 0.0;
    let mut sys = System::new_all();
    let pid = sysinfo::get_current_pid().unwrap();
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    let mut ram_usage = sys.process(pid).unwrap().memory();

    let mut polar_angle = 0.0;
    let mut horizontal_angle = 0.0;
    let mut delta_polar = 0.0;
    let mut delta_horiz = 0.0;

    while !window.should_close() {
        let frame_start = Instant::now();

        glfw.poll_events();

        state.pre_update();
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
                    match key {
                        Key::A => match action {
                            Action::Release => {
                                delta_polar = 0.0;
                            }
                            Action::Press => {
                                delta_polar = -0.01;
                            }
                            _ => {}
                        },
                        Key::D => match action {
                            Action::Release => {
                                delta_polar = 0.0;
                            }
                            Action::Press => {
                                delta_polar = 0.01;
                            }
                            _ => {}
                        },
                        Key::W => match action {
                            Action::Release => {
                                delta_horiz = 0.0;
                            }
                            Action::Press => {
                                delta_horiz = -0.01;
                            }
                            _ => {}
                        },
                        Key::S => match action {
                            Action::Release => {
                                delta_horiz = 0.0;
                            }
                            Action::Press => {
                                delta_horiz = 0.01;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        polar_angle += delta_polar;
        horizontal_angle += delta_horiz;
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
        state.render();
        mesh_r.draw(polar_angle, horizontal_angle);

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
