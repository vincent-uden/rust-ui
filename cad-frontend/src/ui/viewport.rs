use std::{
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
        }
    }
}

impl ViewportData {
    pub fn projection(&self) -> glm::Mat4 {
        glm::perspective(self.size.x / self.size.y, 45.0, 0.0001, 100.0)
    }

    pub fn model(&self) -> glm::Mat4 {
        glm::scaling(&glm::vec3(1.0, 1.0, 1.0))
    }

    pub fn view(&self) -> glm::Mat4 {
        // Create camera position using spherical coordinates
        let camera_distance = self.distance;
        let camera_pos = glm::Vec3::new(
            camera_distance * self.azimuthal_angle.sin() * self.polar_angle.cos(),
            camera_distance * self.azimuthal_angle.sin() * self.polar_angle.sin(),
            camera_distance * self.azimuthal_angle.cos(),
        );
        glm::look_at(
            &camera_pos,                    // Camera position
            &glm::Vec3::new(0.0, 0.0, 0.0), // Look at origin
            &glm::Vec3::new(0.0, 0.0, 1.0), // Up vector
        )
    }
}

// Perhaps each area type will have its own struct like this that can generate a layout?
#[derive(Debug, Clone, Copy)]
pub struct Viewport {}

impl Viewport {
    pub fn generate_layout(
        tree: &mut TaffyTree<NodeContext<App>>,
        parent: NodeId,
        data: &ViewportData,
    ) {
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
                &[spacer, data_disp],
            )
            .unwrap();
        tree.add_child(parent, container).unwrap();
    }
}
