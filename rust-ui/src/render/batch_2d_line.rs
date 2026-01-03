use std::ffi::c_void;

use gl::types::GLuint;

use crate::{geometry::Vector, render::rect::vertices, shader::Shader};
use anyhow::{Result, anyhow};

const MAX_TRACES: usize = 20;

/// Renders a line graph onto a single quad somehwere on the screen.
#[derive(Debug)]
pub struct GraphRenderer {
    shader: Shader,
    quad_vao: GLuint,
    quad_vbo: GLuint,
    texture_id: GLuint,
    /// The texture will be roughly window_width*MAX_TRACES in size
    texture_size: Vector<i32>,
}

impl GraphRenderer {
    /// Creates a new graph renderer. For now we'll assume a graph can't be bigger than the screen.
    /// If it is it will be interpolated in the shader.
    pub fn new(shader: Shader, window_size: Vector<i32>) -> Result<Self> {
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
                gl::TEXTURE,
                0,
                gl::RED as i32,
                window_size.x,
                window_size.y,
                0,
                gl::RED,
                gl::FLOAT,
                std::ptr::null(),
            );
            // Interpolation parameters lifted from font rendering. Perhaps i want something else
            // here
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        Ok(Self {
            shader,
            quad_vao,
            quad_vbo,
            texture_id,
            texture_size: window_size,
        })
    }
}
