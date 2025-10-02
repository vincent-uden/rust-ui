use std::ffi::c_void;

use crate::{geometry::Vector, render::Color, shader::Shader};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CircleVertex {
    pub position: [f32; 3],
}

#[derive(Debug)]
pub struct CircleRenderer {
    pub shader: Shader,
    vao: u32,
    vbo: u32,
    segments: usize,
}

impl CircleRenderer {
    pub fn new(shader: Shader) -> Self {
        let mut vao = 0;
        let mut vbo = 0;
        let segments = 64;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<CircleVertex>() as i32,
                std::ptr::null(),
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Self {
            shader,
            vao,
            vbo,
            segments,
        }
    }

    pub fn draw(
        &self,
        center: Vector<f32>,
        radius: f32,
        color: Color,
        thickness: f32,
        window_size: Vector<f32>,
    ) {
        let ident: glm::Mat4 = glm::identity();
        let projection = glm::ortho(0.0, window_size.x, window_size.y, 0.0, -1.0, 1.0);
        let center_3d = glm::vec3(center.x, center.y, 0.0);
        self.draw_3d(
            center_3d,
            radius,
            color,
            thickness,
            &projection,
            &ident,
            &ident,
        );
    }

    pub fn draw_3d(
        &self,
        center: glm::Vec3,
        radius: f32,
        color: Color,
        thickness: f32,
        projection: &glm::Mat4,
        model: &glm::Mat4,
        view: &glm::Mat4,
    ) {
        self.draw_3d_oriented(
            center,
            radius,
            color,
            thickness,
            projection,
            model,
            view,
            glm::vec3(1.0, 0.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
        );
    }

    pub fn draw_3d_oriented(
        &self,
        center: glm::Vec3,
        radius: f32,
        color: Color,
        thickness: f32,
        projection: &glm::Mat4,
        model: &glm::Mat4,
        view: &glm::Mat4,
        x_axis: glm::Vec3,
        y_axis: glm::Vec3,
    ) {
        let mut vertices = Vec::with_capacity(self.segments);
        
        for i in 0..self.segments {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (self.segments as f32);
            let x = angle.cos();
            let y = angle.sin();
            
            let pos = center + radius * (x * x_axis + y * y_axis);
            vertices.push(CircleVertex {
                position: [pos.x, pos.y, pos.z],
            });
        }

        self.shader.use_shader();

        self.shader.set_uniform("model", model);
        self.shader.set_uniform("view", view);
        self.shader.set_uniform("projection", projection);
        let color_vec = glm::make_vec4(&[color.r, color.g, color.b, color.a]);
        self.shader.set_uniform("color", &color_vec);

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<CircleVertex>() * vertices.len()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );

            gl::LineWidth(thickness);

            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::LINE_LOOP, 0, self.segments as i32);
            gl::BindVertexArray(0);

            gl::LineWidth(1.0);
        }
    }
}

impl Drop for CircleRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
