use std::ffi::c_void;

use crate::{
    geometry::Vector,
    render::Color,
    shader::Shader,
};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LineVertex {
    pub position: [f32; 2],
}

#[derive(Debug)]
pub struct LineRenderer {
    shader: Shader,
    vao: u32,
    vbo: u32,
}

impl LineRenderer {
    pub fn new(shader: Shader) -> Self {
        let mut vao = 0;
        let mut vbo = 0;
        
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            
            // Position attribute
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<LineVertex>() as i32,
                std::ptr::null(),
            );
            
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        
        Self {
            shader,
            vao,
            vbo,
        }
    }
    
    pub fn draw(&self, start: Vector<f32>, end: Vector<f32>, color: Color, thickness: f32, window_size: Vector<f32>) {
        let vertices = [
            LineVertex { position: [start.x, start.y] },
            LineVertex { position: [end.x, end.y] },
        ];
        
        self.shader.use_shader();
        
        // Set projection matrix (orthographic, screen coordinates)
        let projection = glm::ortho(0.0, window_size.x, window_size.y, 0.0, -1.0, 1.0);
        self.shader.set_uniform("projection", &projection);
        
        // Set uniforms
        let color_vec = glm::make_vec4(&[color.r, color.g, color.b, color.a]);
        self.shader.set_uniform("color", &color_vec);
        
        // Upload vertex data
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<LineVertex>() * vertices.len()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );
            
            // Set line thickness
            gl::LineWidth(thickness);
            
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::LINES, 0, 2);
            gl::BindVertexArray(0);
            
            // Reset line thickness
            gl::LineWidth(1.0);
        }
    }
}

impl Drop for LineRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}