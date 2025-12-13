use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use keybinds::{KeyInput, Keybind, Keybinds};
use modes::{Config, MouseInput};
use strum::EnumString;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, Default, serde::Serialize, serde::Deserialize,
)]
pub enum AppMouseAction {
    #[default]
    Pan,
    Orbit,
    PlacePoint,
}

// TODO(Next): Confirm message, which places line
#[derive(Debug, EnumString, Clone, Copy, PartialEq, Eq)]
pub enum AppBindableMessage {
    PopMode,
    ToggleSettings,
    ToggleProjection,
    ToggleDebugDraw,
    DumpDebugPick,
    TogglePerformanceOverlay,
    SplitAreaHorizontally,
    SplitAreaVertically,
    CollapseBoundary,
    ActivatePointMode,
    Confirm,
}

#[derive(Debug, EnumString, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum AppMode {
    Base,
    Sketch,
    Point,
    Line,
    Circle,
}

pub fn default_config() -> Config<AppMode, AppBindableMessage, AppMouseAction> {
    let base_keybinds = vec![
        Keybind::new(
            KeyInput::from_str("Escape").unwrap(),
            AppBindableMessage::PopMode,
        ),
        Keybind::new(
            KeyInput::from_str("F8").unwrap(),
            AppBindableMessage::ToggleSettings,
        ),
        Keybind::new(
            KeyInput::from_str("F9").unwrap(),
            AppBindableMessage::ToggleProjection,
        ),
        Keybind::new(
            KeyInput::from_str("F10").unwrap(),
            AppBindableMessage::ToggleDebugDraw,
        ),
        Keybind::new(
            KeyInput::from_str("F11").unwrap(),
            AppBindableMessage::DumpDebugPick,
        ),
        Keybind::new(
            KeyInput::from_str("F12").unwrap(),
            AppBindableMessage::TogglePerformanceOverlay,
        ),
        Keybind::new(
            KeyInput::from_str("h").unwrap(),
            AppBindableMessage::SplitAreaHorizontally,
        ),
        Keybind::new(
            KeyInput::from_str("v").unwrap(),
            AppBindableMessage::SplitAreaVertically,
        ),
        Keybind::new(
            KeyInput::from_str("d").unwrap(),
            AppBindableMessage::CollapseBoundary,
        ),
    ];
    let base_bindings = Keybinds::new(base_keybinds);

    let sketch_keybinds = vec![Keybind::new(
        KeyInput::from_str("p").unwrap(),
        AppBindableMessage::ActivatePointMode,
    )];
    let sketch_bindings = Keybinds::new(sketch_keybinds);
    let line_keybinds = vec![Keybind::new(
        KeyInput::from_str("Enter").unwrap(),
        AppBindableMessage::Confirm,
    )];
    let line_bindings = Keybinds::new(line_keybinds);

    let circle_keybinds = vec![Keybind::new(
        KeyInput::from_str("Enter").unwrap(),
        AppBindableMessage::Confirm,
    )];
    let circle_bindings = Keybinds::new(circle_keybinds);

    let mut bindings = HashMap::new();
    bindings.insert(AppMode::Base, base_bindings);
    bindings.insert(AppMode::Sketch, sketch_bindings);
    bindings.insert(AppMode::Line, line_bindings);
    bindings.insert(AppMode::Circle, circle_bindings);

    let mut mouse = HashMap::new();
    mouse.insert(
        AppMode::Base,
        vec![
            (MouseInput::from_str("Middle").unwrap(), AppMouseAction::Pan),
            (
                MouseInput::from_str("Shift+Middle").unwrap(),
                AppMouseAction::Orbit,
            ),
        ],
    );
    mouse.insert(
        AppMode::Point,
        vec![(
            MouseInput::from_str("Left").unwrap(),
            AppMouseAction::PlacePoint,
        )],
    );

    Config { bindings, mouse }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_parse_config_file() {
        let contents = include_str!("../assets/default.conf");
        let config = Config::from_str(contents).unwrap();
        let default_cfg = default_config();

        assert_eq!(config.bindings, default_cfg.bindings);
        assert_eq!(config.mouse, default_cfg.mouse);
    }
}
