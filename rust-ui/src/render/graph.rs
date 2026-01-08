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

#[derive(Debug, Clone, Copy)]
pub enum Interpolation {
    Linear,
}

impl Interpolation {
    /// Returns the amount of pixels that could be samples from `points`. The return value will
    /// equal `n` if `points[0] <= limits.x0.x && points[points.len()-1] >= limits.x1.x`.
    /// Otherwise fewer points than `n` will be returned. `out` is assumed to be at least `n` long.
    fn interpolate(
        &self,
        points: &[Vector<f32>],
        limits: Rect<f32>,
        n: usize,
        out: &mut [f32],
    ) -> usize {
        if points.is_empty() {
            return 0;
        }
        let mut x = limits.x0.x;
        let mut i = 0;
        let mut j = 0;
        let dx = limits.width() / (n as f32);

        while x < limits.x1.x && i < (points.len() - 1) && j < n {
            match self {
                Interpolation::Linear => {
                    let delta = points[i + 1] - points[i];
                    out[j] = points[i].y + delta.y * (x - points[i].x);
                }
            }

            x += dx;
            j += 1;
            while x > points[i + 1].x && i < (points.len() - 1) {
                i += 1;
            }
        }

        i
    }
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
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
    /// Params:
    pub fn bind_graph(
        &mut self,
        points: &[Vector<f32>], // data domain, sorted along the x axis
        limits: Rect<f32>,      // data domain
        interpolation: Interpolation,
        graph_size: Vector<f32>, // screen domain
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
        fake_buffer.resize((self.texture_size.x * MAX_TRACES) as usize, 0.0);
        self.active_traces = 1;
        self.limits[channel] = limits;

        let (lower_idx, upper_idx) = binary_search_for_limits(points, limits.x0.x, limits.x1.x);
        interpolation.interpolate(
            &points[lower_idx..upper_idx],
            limits,
            graph_size.x as usize,
            &mut fake_buffer[0..(self.texture_size.x as usize)],
        );

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
        self.shader
            .set_uniform("yLimits", &glm::vec2(limits.x0.y, limits.x1.y));

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

/// Finds the smallest sub-set of `points` that spans (`min_x`, `max_x`) if possible. Returns (0,
/// points.len() - 1) if `points` contains a data range smaller than (`min_x`, `max_x`).
fn binary_search_for_limits(points: &[Vector<f32>], min_x: f32, max_x: f32) -> (usize, usize) {
    const EPSILON: f32 = 1e-6;
    if points.is_empty() {
        return (0, 0);
    }
    let left = points.partition_point(|p| p.x < min_x - EPSILON);
    let right = points
        .partition_point(|p| p.x <= max_x + EPSILON)
        .saturating_sub(1);
    if left <= right && points[left].x <= min_x + EPSILON && points[right].x >= max_x - EPSILON {
        (left, right)
    } else {
        (0, points.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Vector;

    #[test]
    fn test_single_point_inside_range() {
        let points = vec![Vector::new(0.5, 1.0)];
        assert_eq!(binary_search_for_limits(&points, 0.0, 1.0), (0, 0));
    }

    #[test]
    fn test_points_at_exact_min_max() {
        let points = vec![Vector::new(0.0, 1.0), Vector::new(1.0, 2.0)];
        assert_eq!(binary_search_for_limits(&points, 0.0, 1.0), (0, 1));
    }

    #[test]
    fn test_points_near_edges_with_epsilon() {
        let points = vec![Vector::new(0.000001, 1.0), Vector::new(0.999999, 2.0)];
        assert_eq!(binary_search_for_limits(&points, 0.0, 1.0), (0, 1));
    }

    #[test]
    fn test_subset_in_middle() {
        let points = vec![
            Vector::new(-1.0, 0.0),
            Vector::new(0.5, 1.0),
            Vector::new(1.5, 2.0),
        ];
        assert_eq!(binary_search_for_limits(&points, 0.0, 1.0), (0, 2));
    }

    #[test]
    fn test_full_range_covered() {
        let points = vec![
            Vector::new(0.0, 0.0),
            Vector::new(0.5, 1.0),
            Vector::new(1.0, 2.0),
        ];
        assert_eq!(binary_search_for_limits(&points, 0.0, 1.0), (0, 2));
    }

    #[test]
    fn test_partial_coverage_fallback() {
        let points = vec![
            Vector::new(0.0, 0.0),
            Vector::new(0.5, 1.0),
            Vector::new(1.0, 2.0),
        ];
        assert_eq!(binary_search_for_limits(&points, -10.0, 10.0), (0, 2));
    }
}
