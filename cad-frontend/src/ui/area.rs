use std::{cell::RefCell, f32::consts::PI, sync::Arc, time::Instant};

use cad::{entity::GuidedEntity, registry::RegId};
use glfw::{Action, Key, Modifiers, Scancode};
use rust_ui::{
    geometry::{Rect, Vector},
    render::{
        COLOR_BLACK, COLOR_LIGHT, Color, NORD1, NORD3, NORD9, NORD11, NORD14, Text,
        renderer::{Anchor, NodeContext, RenderLayout, Renderer, UiBuilder, flags},
    },
};
use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, Dimension, FlexDirection, Size, Style, TaffyTree, prelude::length};

use crate::{
    app::{self, App, AppMutableState},
    modes::{AppMode, BindableMessage, ModeStack},
    ui::{
        modes, scene_explorer,
        viewport::{self, ViewportData},
    },
};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum AreaType {
    Red,
    Green,
    Blue,
    Viewport,
    SceneExplorer,
    Modes,
}

impl AreaType {
    pub fn all() -> [AreaType; 6] {
        [
            AreaType::Red,
            AreaType::Green,
            AreaType::Blue,
            AreaType::Viewport,
            AreaType::SceneExplorer,
            AreaType::Modes,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AreaType::Red => "Red",
            AreaType::Green => "Green",
            AreaType::Blue => "Blue",
            AreaType::Viewport => "Viewport",
            AreaType::SceneExplorer => "Scene Explorer",
            AreaType::Modes => "Modes",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize)]
pub struct AreaId(pub i64);

impl RegId for AreaId {
    fn new() -> Self {
        Self(0)
    }

    fn increment(self) -> Self {
        let AreaId(id) = self;
        Self(id + 1)
    }
}

impl Default for AreaId {
    fn default() -> Self {
        AreaId(-1)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum AreaData {
    #[default]
    None,
    Viewport(ViewportData),
}

impl TryFrom<AreaData> for ViewportData {
    type Error = AreaData;

    fn try_from(value: AreaData) -> Result<Self, Self::Error> {
        match value {
            AreaData::Viewport(viewport_data) => Ok(viewport_data),
            _ => Err(value),
        }
    }
}

impl<'a> TryFrom<&'a AreaData> for &'a ViewportData {
    type Error = &'a AreaData;

    fn try_from(value: &'a AreaData) -> Result<Self, Self::Error> {
        match value {
            AreaData::Viewport(viewport_data) => Ok(viewport_data),
            _ => Err(value),
        }
    }
}

impl<'a> TryFrom<&'a mut AreaData> for &'a mut ViewportData {
    type Error = &'a mut AreaData;

    fn try_from(value: &'a mut AreaData) -> Result<Self, Self::Error> {
        match value {
            AreaData::Viewport(viewport_data) => Ok(viewport_data),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Area {
    pub id: AreaId,
    pub area_type: AreaType,
    pub area_data: AreaData,
    pub bbox: Rect<f32>,
    pub hovered: Option<usize>,
    pub expanded: Option<usize>,
    pub expand_hovered: Option<usize>,
    pub mouse_pos: Vector<f32>,
}

impl Serialize for Area {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Area", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("area_type", &self.area_type)?;
        state.serialize_field("bbox", &self.bbox)?;
        state.serialize_field("hovered", &self.hovered)?;
        state.serialize_field("expanded", &self.expanded)?;
        state.serialize_field("expand_hovered", &self.expand_hovered)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Area {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AreaHelper {
            id: AreaId,
            area_type: AreaType,
            bbox: Rect<f32>,
            hovered: Option<usize>,
            expanded: Option<usize>,
            expand_hovered: Option<usize>,
        }

        let helper = AreaHelper::deserialize(deserializer)?;

        Ok(Area {
            id: helper.id,
            area_type: helper.area_type,
            area_data: match helper.area_type {
                AreaType::Viewport => AreaData::Viewport(ViewportData::default()),
                _ => AreaData::None,
            },
            bbox: helper.bbox,
            hovered: helper.hovered,
            expanded: helper.expanded,
            expand_hovered: helper.expand_hovered,
            mouse_pos: Vector { x: 0.0, y: 0.0 },
        })
    }
}

impl Area {
    pub fn new(id: AreaId, area_type: AreaType, bbox: Rect<f32>) -> Self {
        Self {
            id,
            area_type,
            area_data: match area_type {
                AreaType::Viewport => AreaData::Viewport(ViewportData::default()),
                _ => AreaData::None,
            },
            bbox,
            hovered: None,
            expanded: None,
            expand_hovered: None,
            mouse_pos: Vector { x: 0.0, y: 0.0 },
        }
    }

    pub fn generate_layout(
        &mut self,
        state: &AppMutableState,
        mode_stack: &ModeStack<AppMode, BindableMessage>,
    ) -> RenderLayout<App> {
        let tree = TaffyTree::new();
        let reftree = RefCell::new(tree);

        let id = self.id;

        let root = reftree
            .borrow_mut()
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: Dimension::percent(1.0),
                        height: Dimension::percent(1.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    bg_color: match self.area_type {
                        AreaType::Red => NORD11,
                        AreaType::Green => NORD14,
                        AreaType::Blue => NORD9,
                        AreaType::Viewport => Color::new(0.0, 0.0, 0.0, 0.0),
                        AreaType::SceneExplorer => NORD1,
                        AreaType::Modes => NORD1,
                    },
                    on_mouse_exit: Some(Arc::new(move |state: &mut Renderer<App>| {
                        // Might not exist if we exit on the same frame an area is deleted
                        if let Some(area) = state.app_state.area_map.get_mut(&id) {
                            area.expanded = None;
                            area.expand_hovered = None;
                            area.hovered = None;
                        }
                    })),
                    ..Default::default()
                },
            )
            .unwrap();

        match self.area_type {
            AreaType::Red | AreaType::Blue | AreaType::Green => {}
            AreaType::Viewport => {
                viewport::Viewport::generate_layout(
                    &reftree,
                    root,
                    &self.area_data.try_into().unwrap(),
                );
            }
            AreaType::SceneExplorer => {
                scene_explorer::SceneExplorer::generate_layout(&reftree, root, state, mode_stack);
            }
            AreaType::Modes => {
                modes::Modes::generate_layout(&reftree, root, mode_stack);
            }
        }

        let tree = UiBuilder::extract_tree(reftree);
        RenderLayout {
            tree,
            root,
            desired_size: Size {
                width: AvailableSpace::Definite(self.bbox.width()),
                height: AvailableSpace::Definite(self.bbox.height()),
            },
            root_pos: self.bbox.x0,
            anchor: Anchor::TopLeft,
            scissor: true,
        }
    }

    pub fn area_kind_picker_layout(&mut self) -> RenderLayout<App> {
        let id = self.id;
        let mut tree = TaffyTree::new();
        let root = tree
            .new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                NodeContext {
                    bg_color: Color::default(),
                    ..Default::default()
                },
            )
            .unwrap();

        let bar = tree
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: Dimension::length(self.bbox.width()),
                        height: Dimension::auto(),
                    },
                    padding: taffy::Rect {
                        left: length(4.0),
                        right: length(4.0),
                        top: length(4.0),
                        bottom: length(4.0),
                    },
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT,
                    // Hack to get make the bar text height
                    text: Text {
                        text: "M".into(),
                        font_size: 18,
                        color: Color::default(),
                    },
                    bg_color: COLOR_BLACK,
                    ..Default::default()
                },
            )
            .unwrap();

        let kind_tab = tree
            .new_leaf_with_context(
                Style {
                    padding: taffy::Rect {
                        left: length(4.0),
                        right: length(4.0),
                        top: length(4.0),
                        bottom: length(4.0),
                    },
                    ..Default::default()
                },
                NodeContext {
                    flags: flags::TEXT | flags::HOVER_BG,
                    text: Text {
                        text: "Area type".into(),
                        font_size: 18,
                        color: COLOR_LIGHT,
                    },
                    bg_color: COLOR_BLACK,
                    bg_color_hover: NORD3,
                    on_left_mouse_down: Some(Arc::new(move |state: &mut Renderer<App>| {
                        let area = &mut state.app_state.area_map[id];
                        if let Some(_) = area.expanded {
                            area.expanded = None;
                        } else {
                            area.expanded = Some(0);
                        }
                    })),
                    ..Default::default()
                },
            )
            .unwrap();

        let expanded = tree
            .new_leaf_with_context(
                Style {
                    padding: taffy::Rect {
                        left: length(0.0),
                        right: length(0.0),
                        top: length(-(18.0 + 4.0 + 4.0)),
                        bottom: length(0.0),
                    },
                    flex_direction: FlexDirection::Column,
                    align_self: Some(taffy::AlignItems::Start),
                    ..Default::default()
                },
                NodeContext {
                    bg_color: COLOR_BLACK,
                    ..Default::default()
                },
            )
            .unwrap();

        tree.add_child(expanded, kind_tab).unwrap();

        if self.expanded.is_some() {
            for (_i, kind) in AreaType::all().into_iter().enumerate() {
                let id = self.id;
                let node = tree
                    .new_leaf_with_context(
                        Style {
                            size: Size::auto(),
                            padding: taffy::Rect {
                                left: length(4.0),
                                right: length(4.0),
                                top: length(4.0),
                                bottom: length(4.0),
                            },
                            ..Default::default()
                        },
                        NodeContext {
                            flags: flags::TEXT | flags::HOVER_BG,
                            text: Text {
                                text: kind.name().into(),
                                font_size: 18,
                                color: COLOR_LIGHT,
                            },
                            bg_color: COLOR_BLACK,
                            bg_color_hover: NORD3,
                            on_left_mouse_up: Some(Arc::new(move |state| {
                                state.app_state.area_map[id].area_type = kind;
                                let area = &mut state.app_state.area_map[id];
                                match kind {
                                    AreaType::Viewport => {
                                        area.area_data =
                                            AreaData::Viewport(ViewportData::default());
                                    }
                                    _ => {}
                                }
                                area.expanded = None;
                            })),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                tree.add_child(expanded, node).unwrap();
            }
        }

        tree.add_child(root, bar).unwrap();
        tree.add_child(root, expanded).unwrap();

        RenderLayout {
            tree,
            root,
            desired_size: Size {
                width: AvailableSpace::Definite(self.bbox.width()),
                height: AvailableSpace::Definite(self.bbox.height()),
            },
            root_pos: self.bbox.x0,
            anchor: Anchor::TopLeft,
            scissor: true,
        }
    }

    pub fn handle_key(
        &mut self,
        _state: &mut AppMutableState,
        _key: Key,
        _scancode: Scancode,
        _action: Action,
        _modifiers: Modifiers,
    ) {
    }

    /// Position is in window coordinates, the area has to decide on its own if it cares about
    /// out-of-bounds events or not.
    pub fn handle_mouse_position(
        &mut self,
        state: &mut AppMutableState,
        mode_stack: &ModeStack<AppMode, BindableMessage>,
        position: Vector<f32>,
        delta: Vector<f32>,
    ) {
        match &mut self.area_data {
            AreaData::Viewport(data) => match data.interaction_state {
                viewport::InteractionState::Orbit => {
                    data.polar_angle -= delta.x * 0.01;
                    data.azimuthal_angle -= delta.y * 0.01;
                    data.azimuthal_angle = data.azimuthal_angle.clamp(0.00001, PI - 0.00001);
                }
                viewport::InteractionState::Pan => {
                    let pan_speed = data.distance / data.size.y * 0.5;
                    let right = data.right_vector();
                    let up = data.up_vector();
                    data.looking_at -= right * delta.x * pan_speed;
                    data.looking_at += up * delta.y * pan_speed;
                }
                _ => {
                    if mode_stack.is_active(&AppMode::Point) {
                        let mouse_in_viewport = self.mouse_pos - self.bbox.x0;
                        if let Some(sketch_info) = state
                            .scene
                            .sketches
                            .iter_mut()
                            .find(|s| s.id == state.sketch_mode_data.sketch_id)
                        {
                            state.point_mode_data.pending =
                                data.screen_to_sketch_coords(mouse_in_viewport, &sketch_info.plane);
                        }
                    }
                    if mode_stack.is_active(&AppMode::Line) {
                        let mouse_in_viewport = self.mouse_pos - self.bbox.x0;
                        if let Some(sketch_info) = state
                            .scene
                            .sketches
                            .iter_mut()
                            .find(|s| s.id == state.sketch_mode_data.sketch_id)
                            && let Some(sketch_coords) =
                                data.screen_to_sketch_coords(mouse_in_viewport, &sketch_info.plane)
                        {
                            if let Some(last) = state.line_mode_data.points.last_mut() {
                                *last = sketch_coords;
                            } else {
                                state.line_mode_data.points.push(sketch_coords);
                            }
                        }
                    }
                    if mode_stack.is_active(&AppMode::Circle) {
                        let mouse_in_viewport = self.mouse_pos - self.bbox.x0;
                        if let Some(sketch_info) = state
                            .scene
                            .sketches
                            .iter_mut()
                            .find(|s| s.id == state.sketch_mode_data.sketch_id)
                        {
                            if state.circle_mode_data.boundary.is_none() {
                                state.circle_mode_data.center = data
                                    .screen_to_sketch_coords(mouse_in_viewport, &sketch_info.plane);
                            } else {
                                state.circle_mode_data.boundary = data
                                    .screen_to_sketch_coords(mouse_in_viewport, &sketch_info.plane);
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        self.mouse_pos = position;
    }

    pub fn handle_mouse_button(
        &mut self,
        state: &mut AppMutableState,
        mode_stack: &ModeStack<AppMode, BindableMessage>,
        button: glfw::MouseButton,
        action: Action,
        modifiers: Modifiers,
    ) {
        match &mut self.area_data {
            AreaData::Viewport(viewport_data) => match action {
                Action::Release => match button {
                    glfw::MouseButton::Button1
                    | glfw::MouseButton::Button2
                    | glfw::MouseButton::Button3 => match viewport_data.interaction_state {
                        viewport::InteractionState::Orbit | viewport::InteractionState::Pan => {
                            viewport_data.interaction_state = viewport::InteractionState::None;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Action::Press => match button {
                    glfw::MouseButton::Button1 => {
                        if self.bbox.contains(self.mouse_pos) {
                            let mouse_in_viewport = self.mouse_pos - self.bbox.x0;
                            if let Some(sketch_info) = state
                                .scene
                                .sketches
                                .iter_mut()
                                .find(|s| s.id == state.sketch_mode_data.sketch_id)
                            {
                                if let Some(sketch_coords) = viewport_data
                                    .screen_to_sketch_coords(mouse_in_viewport, &sketch_info.plane)
                                {
                                    match mode_stack.outermost().unwrap() {
                                        AppMode::Point => {
                                            sketch_info.sketch.insert_point(sketch_coords);
                                        }
                                        AppMode::Line => {
                                            state.line_mode_data.points.push(sketch_coords);
                                        }
                                        AppMode::Circle => {
                                            if state.circle_mode_data.boundary.is_none() {
                                                state.circle_mode_data.boundary =
                                                    Some(sketch_coords);
                                            } else {
                                                sketch_info.sketch.insert_circle(
                                                    state.circle_mode_data.center.unwrap(),
                                                    (state.circle_mode_data.center.unwrap()
                                                        - state.circle_mode_data.boundary.unwrap())
                                                    .norm(),
                                                );
                                                state.circle_mode_data.center = None;
                                                state.circle_mode_data.boundary = None;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            } else {
                                viewport_data.interaction_state = viewport::InteractionState::Orbit;
                            }
                        }
                    }
                    glfw::MouseButton::Button3 => {
                        if self.bbox.contains(self.mouse_pos) {
                            if modifiers.contains(Modifiers::Shift) {
                                viewport_data.interaction_state = viewport::InteractionState::Orbit;
                            } else {
                                viewport_data.interaction_state = viewport::InteractionState::Pan;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_mouse_scroll(&mut self, _state: &mut AppMutableState, scroll_delta: Vector<f32>) {
        match &mut self.area_data {
            AreaData::Viewport(viewport_data) => {
                if self.bbox.contains(self.mouse_pos) {
                    if scroll_delta.y < 0.0 {
                        viewport_data.distance *= 1.15;
                    } else {
                        viewport_data.distance /= 1.15;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        match &mut self.area_data {
            AreaData::None => {}
            AreaData::Viewport(data) => match data.interaction_state {
                viewport::InteractionState::AutoMoving => {
                    let passed = Instant::now().duration_since(data.auto_move_start);
                    data.azimuthal_angle = ease_in_out(
                        data.start_azimuthal_angle,
                        data.target_azimuthal_angle,
                        passed.as_secs_f32() / data.auto_move_duration.as_secs_f32(),
                    );
                    data.polar_angle = ease_in_out(
                        data.start_polar_angle,
                        data.target_polar_angle,
                        passed.as_secs_f32() / data.auto_move_duration.as_secs_f32(),
                    );
                    if -(passed.as_secs_f32() - data.auto_move_duration.as_secs_f32()) < 0.05 {
                        data.polar_angle = data.target_polar_angle;
                        data.azimuthal_angle =
                            data.target_azimuthal_angle.clamp(0.00001, PI - 0.00001);
                        data.interaction_state = viewport::InteractionState::None;
                    }
                }
                _ => {}
            },
        }
    }
}

pub fn ease_in_out(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let eased = if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    };
    start + (end - start) * eased
}
