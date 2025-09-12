use rust_ui::{
    geometry::Vector,
    render::{
        COLOR_LIGHT, Color, Text,
        renderer::{Anchor, NodeContext, RenderLayout, flags},
    },
};
use taffy::{AvailableSpace, Dimension, NodeId, Rect, Size, Style, TaffyTree};

use crate::app::App;

#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportData {
    /// Angle from the horizon up to the camera in radians. At 0.0 degrees the camera is parallel
    /// to the ground. At 90.0 degrees the camera is looking straight at the ground.
    pub horizontal_angle: f32,
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
