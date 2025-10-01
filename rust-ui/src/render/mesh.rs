use std::{ffi::c_void, mem::offset_of};

use crate::shader::Shader;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// Renders a single mesh
#[derive(Debug)]
pub struct MeshRenderer {
    shader: Shader,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl MeshRenderer {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, shader: Shader) -> Self {
        let mut out = MeshRenderer {
            shader,
            vertices,
            indices,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };
        unsafe {
            gl::GenVertexArrays(1, &mut out.vao);
            gl::GenBuffers(1, &mut out.vbo);
            gl::GenBuffers(1, &mut out.ebo);

            gl::BindVertexArray(out.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, out.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<Vertex>() * out.vertices.len()) as isize,
                out.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, out.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (size_of::<u32>() * out.indices.len()) as isize,
                out.indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Vertex positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                std::ptr::null(),
            );
            // Vertex normals
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const c_void,
            );

            gl::BindVertexArray(0);
        }
        out
    }

    /// Angles in radians
    pub fn draw(&self, polar_angle: f32, vertical_angle: f32) {
        self.shader.use_shader();
        let mut model = glm::Mat4::identity();
        model = glm::rotate(&model, polar_angle, &glm::vec3(0.0, 1.0, 0.0));
        model = glm::rotate(&model, vertical_angle, &glm::vec3(1.0, 0.0, 0.0));
        let view = glm::look_at(
            &glm::vec3(0.0, 0.0, 10.0), // camera position
            &glm::vec3(0.0, 0.0, 0.0),  // look at origin
            &glm::vec3(0.0, 1.0, 0.0),  // up vector
        );
        let projection = glm::perspective(
            45.0f32.to_radians(),
            1000.0 / 800.0, // aspect ratio
            0.1,            // near plane
            100.0,          // far plane
        );
        self.shader.set_uniform("model", &model);
        self.shader.set_uniform("view", &view);
        self.shader.set_uniform("projection", &projection);
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                (size_of::<u32>() * self.indices.len()) as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::BindVertexArray(0);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }
}
