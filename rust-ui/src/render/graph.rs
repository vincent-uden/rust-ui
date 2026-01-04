use core::f32;
use std::ffi::c_void;

use gl::types::GLuint;
use tracing::info;

use crate::{
    geometry::{Rect, Vector},
    render::{Color, rect::vertices},
    shader::Shader,
};
use anyhow::{Result, anyhow};

const MAX_TRACES: i32 = 10;

#[derive(Debug)]
pub enum Interpolation {
    Linear,
}

/// Renders a line graph onto a single quad somehwere on the screen.
#[derive(Debug)]
pub struct GraphRenderer {
    shader: Shader,
    quad_vao: GLuint,
    quad_vbo: GLuint,
    texture_id: GLuint,
    /// The texture will be roughly window_width*MAX_TRACES in size
    texture_size: Vector<i32>,
    /// The actual dimensions on the screen
    graph_size: Vector<f32>,
    limits: [Rect<f32>; MAX_TRACES as usize],
    active_traces: usize,
}

impl GraphRenderer {
    /// Creates a new graph renderer. For now we'll assume a graph can't be bigger than the screen.
    /// If it is it will be interpolated in the shader.
    pub fn new(shader: Shader, window_size: Vector<i32>) -> Self {
        let mut quad_vao = 0;
        let mut quad_vbo = 0;
        let quad_vertices = vertices();
        let mut texture_id = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut quad_vbo);

            gl::BindVertexArray(quad_vao);

            // Setup static quad geometry
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * quad_vertices.len()) as isize,
                quad_vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindVertexArray(quad_vao);
            // Setup input to shader
            // layout (location = 0) in vec4 vertex; // <vec2 position, vec2 texCoords>
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                (4 * size_of::<f32>()) as i32,
                std::ptr::null(),
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            // Allocate texture
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R32F as i32,
                window_size.x,
                MAX_TRACES,
                0,
                gl::RED, // Single channel texture
                gl::FLOAT,
                std::ptr::null(),
            );
            // Interpolation parameters lifted from font rendering. Perhaps i want something else
            // here
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Self {
            shader,
            quad_vao,
            quad_vbo,
            texture_id,
            texture_size: window_size,
            graph_size: Vector::zero(),
            limits: [Rect::default(); MAX_TRACES as usize],
            active_traces: 0,
        }
    }

    /// Takes a [Vec] of points representing a line graph. We want to draw some subset of the line
    /// graph (or a zoomed out view containing the entire graph), this is controlled by [limits].
    pub fn bind_graph(
        &mut self,
        points: &[Vector<f32>],
        limits: Rect<f32>,
        interpolation: Interpolation,
        graph_size: Vector<f32>,
        channel: usize,
    ) {
        // - Determine which points are in the visible x-range (and just outside, since they're
        //   needed for interpolation)
        // - Calculate a height (by interpolation) for every pixel in the visual graph_size
        // - Store these heights on the channel-th channel of the texture
        // - Then (outside the scope of this function) a shader will draw the line graph
        //
        // To start off, I will just draw a flat line
        let mut fake_buffer: Vec<f32> = vec![];
        for i in 0..self.texture_size.x {
            let x: f32 = (i as f32) / (self.texture_size.x as f32) * f32::consts::TAU;
            fake_buffer.push((x * 5.0).sin());
        }
        for _ in 2..MAX_TRACES {
            for _ in 0..self.texture_size.x {
                fake_buffer.push(0.0)
            }
        }
        for i in 0..self.texture_size.x {
            fake_buffer.push((i as f32) / (self.texture_size.x as f32))
        }
        self.active_traces = 1;
        self.limits[channel] = limits;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                channel as i32,
                fake_buffer.len() as i32 / MAX_TRACES,
                MAX_TRACES,
                gl::RED,
                gl::FLOAT,
                fake_buffer.as_ptr() as *const c_void,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn draw(
        &self,
        channel: i32,
        rect: Rect<f32>,
        bg_color: Color,
        trace_color: Color,
        edge_softness: f32,
    ) {
        if bg_color == Color::new(0.0, 0.0, 0.0, 0.0) {
            return;
        }
        self.shader.use_shader();
        let mut model = glm::Mat4::identity();
        model *= &glm::translation(&glm::Vec3::new(-0.5, -0.5, 0.0));
        let mut scale = glm::make_vec3(&[1.0, 1.0, 1.0]);
        scale.x = rect.size().x + edge_softness * 2.0;
        scale.y = rect.size().y + edge_softness * 2.0;
        model = glm::scale(&model, &scale);
        model = glm::translate(&model, &glm::Vec3::new(0.5, 0.5, 0.0).component_div(&scale));
        model = glm::translate(
            &model,
            &glm::Vec3::new(rect.x0.x, rect.x0.y, 0.0).component_div(&scale),
        );
        model = glm::translate(
            &model,
            &glm::Vec3::new(-edge_softness, -edge_softness, 0.0).component_div(&scale),
        );
        self.shader.set_uniform("model", &model);
        let bg_color_vec = glm::make_vec4(&[bg_color.r, bg_color.g, bg_color.b, bg_color.a]);
        self.shader.set_uniform("bgColor", &bg_color_vec);
        let trace_color_vec =
            glm::make_vec4(&[trace_color.r, trace_color.g, trace_color.b, trace_color.a]);
        self.shader.set_uniform("traceColor", &trace_color_vec);
        let rect_size = glm::Vec2::new(rect.width(), rect.height());
        self.shader.set_uniform("size", &rect_size);
        self.shader.set_uniform("text", &0);
        self.shader.set_uniform("maxTraces", &(MAX_TRACES as f32));
        let limits = self.limits[channel as usize];
        self.shader.set_uniform(
            "limits",
            &glm::vec4(limits.x0.x, limits.x0.y, limits.x1.x, limits.x1.y),
        );

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
