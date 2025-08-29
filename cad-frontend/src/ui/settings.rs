use std::sync::Arc;

use rust_ui::{
    geometry::Vector,
    render::{
        Border, BorderRadius, COLOR_BLACK, COLOR_LIGHT, COLOR_PRIMARY, COLOR_SECONDARY,
        COLOR_SUCCESS, Color, NORD2, Text,
        renderer::{Anchor, NodeContext, RenderLayout, Renderer, flags},
    },
};
use taffy::{
    AlignItems, AvailableSpace, FlexDirection, Rect, Size, Style, TaffyTree,
    prelude::{auto, length},
};

use crate::app::App;

#[derive(Default)]
pub struct Settings {}

impl Settings {
    pub fn generate_layout(&mut self, size: rust_ui::geometry::Vector<f32>) -> RenderLayout<App> {
        let mut tree = TaffyTree::new();

        let root = tree
            .new_leaf_with_context(
                Style {
                    size: Size::from_lengths(size.x, size.y),
                    align_items: Some(AlignItems::Center),
                    justify_items: Some(AlignItems::Center),
                    ..Default::default()
                },
                NodeContext {
                    bg_color: Color::new(0.0, 0.0, 0.0, 0.1),
                    ..Default::default()
                },
            )
            .unwrap();

        let modal = tree
            .new_leaf_with_context(
                Style {
                    margin: Rect::auto(),
                    padding: Rect {
                        left: length(12.0),
                        right: length(12.0),
                        top: length(12.0),
                        bottom: length(12.0),
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: Some(AlignItems::Start),
                    max_size: Size::from_lengths(600.0, 400.0),
                    gap: Size {
                        width: length(0.0),
                        height: length(8.0),
                    },
                    flex_grow: 1.0,
                    ..Default::default()
                },
                NodeContext {
                    bg_color: COLOR_BLACK,
                    border: Border {
                        thickness: 2.0,
                        radius: BorderRadius::all(12.0),
                        color: NORD2,
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let title = tree
            .new_leaf_with_context(
                Style {
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Settings".into(),
                        font_size: 36,
                        color: COLOR_LIGHT,
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let save_layout_btn = tree
            .new_leaf_with_context(
                Style {
                    padding: Rect {
                        left: length(8.0),
                        right: length(8.0),
                        top: length(8.0),
                        bottom: length(8.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT | flags::HOVER_BG,
                    text: Text {
                        text: "Save layout".into(),
                        font_size: 18,
                        color: COLOR_LIGHT,
                    },
                    bg_color: COLOR_PRIMARY,
                    bg_color_hover: COLOR_SECONDARY,
                    border: Border {
                        radius: BorderRadius::all(8.0),
                        ..Default::default()
                    },
                    on_mouse_up: Some(Arc::new(move |state: &mut Renderer<App>| {
                        state.app_state.save_layout();
                    })),
                    ..Default::default()
                },
            )
            .unwrap();

        tree.add_child(root, modal).unwrap();
        tree.add_child(modal, title).unwrap();
        tree.add_child(modal, save_layout_btn).unwrap();

        RenderLayout {
            tree,
            root,
            desired_size: Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            },
            root_pos: Vector::zero(),
            anchor: Anchor::TopLeft,
            scissor: false,
        }
    }
}
