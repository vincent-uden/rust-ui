use std::ffi::c_void;

use crate::{render::Color, shader::Shader};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct PointVertex {
    pub position: [f32; 3],
}

#[derive(Debug)]
pub struct PointRenderer {
    pub shader: Shader,
    vao: u32,
    vbo: u32,
}

impl PointRenderer {
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
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<PointVertex>() as i32,
                std::ptr::null(),
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Self { shader, vao, vbo }
    }

    /// Draw a point in 3D space
    pub fn draw_3d(
        &self,
        pos: glm::Vec3,
        color: Color,
        size: f32,
        projection: &glm::Mat4,
        model: &glm::Mat4,
        view: &glm::Mat4,
    ) {
        let vertices = [PointVertex {
            position: [pos.x, pos.y, pos.z],
        }];

        self.shader.use_shader();

        self.shader.set_uniform("model", model);
        self.shader.set_uniform("view", view);
        self.shader.set_uniform("projection", projection);
        let color_vec = glm::make_vec4(&[color.r, color.g, color.b, color.a]);
        self.shader.set_uniform("color", &color_vec);

        // Upload vertex data
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<PointVertex>() * vertices.len()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );

            gl::PointSize(size);

            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::POINTS, 0, 1);
            gl::BindVertexArray(0);

            gl::PointSize(1.0);
        }
    }
}

impl Drop for PointRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
