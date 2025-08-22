use std::ffi::c_void;

use crate::{
    geometry::{Rect, Vector},
    render::{Border, Color},
    shader::Shader,
};

#[derive(Debug)]
pub struct ScissorRegion {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug)]
pub struct RectRenderer {
    shader: Shader,
    quad_vao: u32,
    quad_vbo: u32,
    scissor_stack: Vec<ScissorRegion>,
}

#[rustfmt::skip]
fn vertices() -> Vec<f32> {
    vec![
        // pos      // tex
        0.0, 1.0, 0.0, 1.0,
        1.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 

        0.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
        1.0, 0.0, 1.0, 0.0
    ]
}

impl RectRenderer {
    pub fn new(shader: Shader) -> Self {
        let mut quad_vao = 0;
        let mut quad_vbo = 0;
        let vertices = vertices();
        unsafe {
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut quad_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<f32>() * vertices.len()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindVertexArray(quad_vao);
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

            Self {
                shader,
                quad_vao,
                quad_vbo,
                scissor_stack: vec![],
            }
        }
    }

    pub fn draw(
        &self,
        rect: Rect<f32>,
        bg_color: Color,
        border_color: Color,
        border: Border,
        edge_softness: f32,
    ) {
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

        // Set uniforms
        self.shader.set_uniform("model", &model);
        let bg_color_vec = glm::make_vec4(&[bg_color.r, bg_color.g, bg_color.b, bg_color.a]);
        self.shader.set_uniform("bgColor", &bg_color_vec);

        let border_color_vec = glm::make_vec4(&[
            border_color.r,
            border_color.g,
            border_color.b,
            border_color.a,
        ]);
        self.shader.set_uniform("borderColor", &border_color_vec);
        self.shader
            .set_uniform("borderThickness", &border.thickness);

        let border_radius_vec = glm::make_vec4(&[
            border.radius.top_left,
            border.radius.top_right,
            border.radius.bottom_left,
            border.radius.bottom_right,
        ]);
        self.shader.set_uniform("borderRadius", &border_radius_vec);
        self.shader.set_uniform("edgeSoftness", &edge_softness);

        // Make sure size is correct
        let rect_size = glm::Vec2::new(rect.width(), rect.height());
        self.shader.set_uniform("size", &rect_size);

        unsafe {
            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for RectRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
            gl::DeleteBuffers(1, &self.quad_vbo);
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}
