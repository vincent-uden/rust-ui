use std::{marker::PhantomData, time::Duration};

use crate::{
    geometry::Vector,
    render::{
        Color, Text,
        renderer::{Anchor, AppState, NodeContext, RenderLayout, flags},
    },
};
use taffy::{AvailableSpace, Dimension, FlexDirection, Size, Style, TaffyTree, prelude::length};

pub struct PerformanceOverlay<T>
where
    T: AppState,
{
    phantom: PhantomData<T>,
    pub max_frame_time: Duration,
    pub visible: bool,
    pub avg_sleep_ms: f64,
    pub ram_usage: u64,
}

impl<T> Default for PerformanceOverlay<T>
where
    T: AppState,
{
    fn default() -> Self {
        Self::new_60_fps()
    }
}

impl<T> PerformanceOverlay<T>
where
    T: AppState,
{
    pub fn new_60_fps() -> Self {
        Self {
            phantom: PhantomData::default(),
            max_frame_time: Duration::from_nanos(1_000_000_000 / 60),
            visible: false,
            avg_sleep_ms: 0.0,
            ram_usage: 0,
        }
    }

    pub fn update(&mut self, avg_sleep_ms: f64, ram_usage: u64) {
        self.avg_sleep_ms = avg_sleep_ms;
        self.ram_usage = ram_usage;
    }

    pub fn generate_layout(&mut self, size: crate::geometry::Vector<f32>) -> RenderLayout<T> {
        let mut tree = TaffyTree::new();

        let title = tree
            .new_leaf_with_context(
                Style {
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: "Performance stats".into(),
                        font_size: 18,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let frame_time = tree
            .new_leaf_with_context(
                Style::default(),
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: format!(
                            "Frame time: {:.2} ms",
                            self.max_frame_time.as_millis() as f64 - self.avg_sleep_ms
                        ),
                        font_size: 14,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let ram_usage = tree
            .new_leaf_with_context(
                Style::default(),
                NodeContext {
                    flags: flags::TEXT,
                    text: Text {
                        text: format!("RAM: {:.2} MB", self.ram_usage / 1_000_000,),
                        font_size: 14,
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        let root = tree
            .new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: Dimension::percent(1.0),
                        height: Dimension::percent(1.0),
                    },
                    gap: Size {
                        width: length(0.0),
                        height: length(8.0),
                    },
                    max_size: size.into(),
                    padding: taffy::Rect::length(12.0),
                    ..Default::default()
                },
                NodeContext {
                    bg_color: Color::new(0.0, 0.0, 0.0, 0.5),
                    ..Default::default()
                },
            )
            .unwrap();

        tree.add_child(root, title).unwrap();
        tree.add_child(root, frame_time).unwrap();
        tree.add_child(root, ram_usage).unwrap();

        RenderLayout {
            tree,
            root,
            delayed_markers: Default::default(),
            desired_size: Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MinContent,
            },
            root_pos: Vector::zero(),
            anchor: Anchor::BottomRight,
            scissor: false,
        }
    }
}
