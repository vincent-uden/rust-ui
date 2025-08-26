use std::ffi::c_void;

use crate::{
    geometry::Rect,
    render::{Border, Color},
    shader::Shader,
};

#[derive(Debug, Clone, Copy)]
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

    pub fn draw(&self, rect: Rect<f32>, bg_color: Color, border: Border, edge_softness: f32) {
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
            border.color.r,
            border.color.g,
            border.color.b,
            border.color.a,
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

    pub fn push_scissor_region(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        window_height: i32,
    ) {
        // Convert Clay coordinates (top-left origin) to OpenGL coordinates (bottom-left origin)
        let clay_y = y as i32;
        let clay_height = height as i32;
        let opengl_y = window_height - clay_y - clay_height;

        let mut new_region = ScissorRegion {
            x: x as i32,
            y: opengl_y,
            width: width as i32,
            height: clay_height,
        };

        if !self.scissor_stack.is_empty() {
            let current = &self.scissor_stack[self.scissor_stack.len() - 1];

            // Calculate intersection
            let left = std::cmp::max(current.x, new_region.x);
            let right = std::cmp::min(current.x + current.width, new_region.x + new_region.width);
            let bottom = std::cmp::max(current.y, new_region.y);
            let top = std::cmp::min(current.y + current.height, new_region.y + new_region.height);

            // Ensure valid intersection
            if left < right && bottom < top {
                new_region = ScissorRegion {
                    x: left,
                    y: bottom,
                    width: right - left,
                    height: top - bottom,
                };
            } else {
                // No intersection - create empty region
                new_region = ScissorRegion {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                };
            }
        }

        self.scissor_stack.push(new_region);
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(
                new_region.x,
                new_region.y,
                new_region.width,
                new_region.height,
            );
        }
    }

    pub fn pop_scissor_region(&mut self) {
        if !self.scissor_stack.is_empty() {
            self.scissor_stack.pop();
            if !self.scissor_stack.is_empty() {
                // Restore previous scissor region
                let region = &self.scissor_stack[self.scissor_stack.len() - 1];
                unsafe {
                    gl::Scissor(region.x, region.y, region.width, region.height);
                }
            } else {
                // No more scissor regions, disable scissor test
                unsafe {
                    gl::Disable(gl::SCISSOR_TEST);
                }
            }
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
