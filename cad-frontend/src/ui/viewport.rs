use std::{
    cell::RefCell,
    f32::consts::PI,
    time::{Duration, Instant},
};

use rust_ui::{
    geometry::Vector,
    render::{
        COLOR_LIGHT, Color, Text,
        renderer::{NodeContext, flags},
    },
};
use taffy::{Dimension, NodeId, Rect, Size, Style, TaffyTree};

use crate::app::App;

#[derive(Debug, Clone, Copy, Default)]
pub enum ProjectionMode {
    #[default]
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum InteractionState {
    Orbit,
    Pan,
    AutoMoving,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportData {
    /// Angle from the horizon up to the camera in radians. At 0.0 degrees the camera is parallel
    /// to the ground. At 90.0 degrees the camera is looking straight at the ground.
    pub azimuthal_angle: f32,
    /// Angle "around" the pole. At 0.0 degrees the camera is looking towards the negative x-axis,
    /// at 90.0 degrees it is looking towards the negative y-axis. (I think)
    pub polar_angle: f32,
    /// The point around which the camera is orbiting. Panning moves this point
    pub looking_at: glm::Vec3,
    /// The distance the camera is from [ViewportData::looking_at], similar to a zoom, but not
    /// quite
    pub distance: f32,
    /// The size on the screen of the viewport
    pub size: Vector<f32>,
    /// A possible mouse state
    pub interaction_state: InteractionState,
    /// Animation state
    pub target_polar_angle: f32,
    /// Animation state
    pub target_azimuthal_angle: f32,
    /// Animation state
    pub auto_move_start: Instant,
    /// Animation state
    pub auto_move_duration: Duration,
    /// Animation state
    pub start_azimuthal_angle: f32,
    /// Animation state
    pub start_polar_angle: f32,
    pub debug_hovered_pixel: (u8, u8, u8, u8),
    pub projection_mode: ProjectionMode,
}

impl Default for ViewportData {
    fn default() -> Self {
        Self {
            azimuthal_angle: PI / 4.0,
            polar_angle: PI / 4.0,
            looking_at: glm::vec3(0.0, 0.0, 0.0),
            distance: 2.0,
            size: Vector::default(),
            interaction_state: InteractionState::default(),
            target_polar_angle: 0.0,
            target_azimuthal_angle: 0.0,
            auto_move_start: Instant::now(),
            auto_move_duration: Duration::from_millis(500),
            start_azimuthal_angle: 0.0,
            start_polar_angle: 0.0,
            debug_hovered_pixel: (0, 0, 0, 0),
            projection_mode: ProjectionMode::default(),
        }
    }
}

impl ViewportData {
    pub fn projection(&self) -> glm::Mat4 {
        match self.projection_mode {
            ProjectionMode::Perspective => {
                glm::perspective(self.size.x / self.size.y, 45.0, 0.0001, 100.0)
            }
            ProjectionMode::Orthographic => {
                let aspect = self.size.x / self.size.y;
                let height = self.distance;
                let width = height * aspect;
                glm::ortho(-width, width, -height, height, 0.0001, 100.0)
            }
        }
    }

    pub fn model(&self) -> glm::Mat4 {
        glm::scaling(&glm::vec3(1.0, 1.0, 1.0))
    }

    pub fn view(&self) -> glm::Mat4 {
        let camera_pos = self.looking_at
            + glm::Vec3::new(
                self.distance * self.azimuthal_angle.sin() * self.polar_angle.cos(),
                self.distance * self.azimuthal_angle.sin() * self.polar_angle.sin(),
                self.distance * self.azimuthal_angle.cos(),
            );
        glm::look_at(
            &camera_pos,
            &self.looking_at,
            &glm::Vec3::new(0.0, 0.0, 1.0),
        )
    }

    pub fn right_vector(&self) -> glm::Vec3 {
        let forward = -glm::Vec3::new(
            self.azimuthal_angle.sin() * self.polar_angle.cos(),
            self.azimuthal_angle.sin() * self.polar_angle.sin(),
            self.azimuthal_angle.cos(),
        );
        let up = glm::Vec3::new(0.0, 0.0, 1.0);
        glm::cross(&forward, &up).normalize()
    }

    pub fn up_vector(&self) -> glm::Vec3 {
        let right = self.right_vector();
        let forward = -glm::Vec3::new(
            self.azimuthal_angle.sin() * self.polar_angle.cos(),
            self.azimuthal_angle.sin() * self.polar_angle.sin(),
            self.azimuthal_angle.cos(),
        );
        glm::cross(&right, &forward).normalize()
    }

    pub fn screen_to_ray(&self, screen_pos: Vector<f32>) -> (glm::Vec3, glm::Vec3) {
        let ndc_x = (2.0 * screen_pos.x) / self.size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / self.size.y;

        match self.projection_mode {
            ProjectionMode::Perspective => {
                let view_proj = self.projection() * self.view();
                let inv_view_proj = glm::inverse(&view_proj);

                let near_point = inv_view_proj * glm::vec4(ndc_x, ndc_y, -1.0, 1.0);
                let far_point = inv_view_proj * glm::vec4(ndc_x, ndc_y, 1.0, 1.0);

                let near_point = glm::vec3(
                    near_point.x / near_point.w,
                    near_point.y / near_point.w,
                    near_point.z / near_point.w,
                );
                let far_point = glm::vec3(
                    far_point.x / far_point.w,
                    far_point.y / far_point.w,
                    far_point.z / far_point.w,
                );

                let ray_direction = (far_point - near_point).normalize();
                (near_point, ray_direction)
            }
            ProjectionMode::Orthographic => {
                let view_proj = self.projection() * self.view();
                let inv_view_proj = glm::inverse(&view_proj);

                let ray_start = inv_view_proj * glm::vec4(ndc_x, ndc_y, -1.0, 1.0);
                let ray_start = glm::vec3(
                    ray_start.x / ray_start.w,
                    ray_start.y / ray_start.w,
                    ray_start.z / ray_start.w,
                );

                let camera_pos = self.looking_at
                    + glm::Vec3::new(
                        self.distance * self.azimuthal_angle.sin() * self.polar_angle.cos(),
                        self.distance * self.azimuthal_angle.sin() * self.polar_angle.sin(),
                        self.distance * self.azimuthal_angle.cos(),
                    );
                let ray_direction = (self.looking_at - camera_pos).normalize();

                (ray_start, ray_direction)
            }
        }
    }

    fn ray_plane_intersection(
        ray_origin: &glm::Vec3,
        ray_direction: &glm::Vec3,
        plane_normal: &glm::Vec3,
        plane_point: &glm::Vec3,
    ) -> Option<glm::Vec3> {
        let denom = glm::dot(ray_direction, plane_normal);

        if denom.abs() < 1e-6 {
            return None;
        }

        let t = glm::dot(&(plane_point - ray_origin), plane_normal) / denom;

        if t < 0.0 {
            return None;
        }

        Some(ray_origin + t * ray_direction)
    }

    pub fn screen_to_sketch_coords(
        &self,
        screen_pos: Vector<f32>,
        plane: &cad::Plane,
    ) -> Option<glm::DVec2> {
        let (ray_origin, ray_direction) = self.screen_to_ray(screen_pos);

        let plane_normal = plane.normal().cast::<f32>();
        let plane_origin = plane.origin().cast::<f32>();

        let intersection = Self::ray_plane_intersection(
            &ray_origin,
            &ray_direction,
            &plane_normal,
            &plane_origin,
        )?;

        let x_coord = glm::dot(&intersection, &plane.x.cast::<f32>()) as f64;
        let y_coord = glm::dot(&intersection, &plane.y.cast::<f32>()) as f64;

        Some(glm::vec2(x_coord, y_coord))
    }
}

// Perhaps each area type will have its own struct like this that can generate a layout?
#[derive(Debug, Clone, Copy)]
pub struct Viewport {}

impl Viewport {
    pub fn generate_layout(
        tree: &RefCell<TaffyTree<NodeContext<App>>>,
        parent: NodeId,
        data: &ViewportData,
    ) {
        let mut tree = tree.borrow_mut();
        // For debug purposes
        let data_disp = tree
            .new_leaf_with_context(
                Style {
                    padding: Rect::length(8.0),
                    size: Size {
                        width: Dimension::length(280.0),
                        height: Dimension::auto(),
                    },
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT | flags::EXPLICIT_TEXT_LAYOUT,
                    bg_color: Color::new(0.0, 0.0, 0.0, 0.2),
                    text: Text {
                        text: format!("{:#?}", data),
                        font_size: 12,
                        color: COLOR_LIGHT,
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let spacer = tree
            .new_leaf(Style {
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();
        let container = tree
            .new_with_children(
                Style {
                    flex_direction: taffy::FlexDirection::Column,
                    ..Default::default()
                },
                &[spacer],
            )
            .unwrap();
        tree.add_child(parent, container).unwrap();
    }
}
