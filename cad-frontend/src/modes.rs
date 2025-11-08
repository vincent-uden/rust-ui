use anyhow::{Result, anyhow};
use colored::Colorize as _;
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    hash::Hash,
    str::FromStr,
};

use keybinds::{KeyInput, Keybind, Keybinds};
use strum::EnumString;

pub trait Mode: Debug + PartialEq + Hash {}

#[derive(Debug)]
pub struct ModeStack<M>
where
    M: Mode,
{
    stack: Vec<M>,
}

impl<M> ModeStack<M>
where
    M: Mode,
{
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn is_active(&self, mode: &M) -> bool {
        self.stack.contains(mode)
    }

    pub fn is_outermost(&self, mode: &M) -> bool {
        self.stack.last().map(|m| m == mode).unwrap_or(false)
    }

    pub fn pop(&mut self) -> Option<M> {
        self.stack.pop()
    }

    pub fn pop_until(&mut self, stop_at: &M) {
        while self.stack.last() != Some(stop_at) {
            self.stack.pop();
        }
    }

    pub fn push(&mut self, mode: M) {
        self.stack.push(mode);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MouseModifiers {
    pub ctrl: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MouseInput {
    pub button: MouseButton,
    pub modifiers: MouseModifiers,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, Default, serde::Serialize, serde::Deserialize,
)]
pub enum MouseAction {
    #[default]
    Pan,
    Orbit,
    PlacePoint,
}

pub type MouseBinding = (MouseInput, MouseAction);

impl FromStr for MouseInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();

        let mut modifiers = MouseModifiers::default();
        let mut button_str = s;

        if parts.len() > 1 {
            button_str = parts.last().unwrap();
            for part in &parts[..parts.len() - 1] {
                match *part {
                    "Ctrl" => modifiers.ctrl = true,
                    "Shift" => modifiers.shift = true,
                    _ => return Err(anyhow!("Unknown modifier: {}", part)),
                }
            }
        }

        // Strip "Mouse" prefix if present
        let button_name = if let Some(stripped) = button_str.strip_prefix("Mouse") {
            stripped
        } else {
            button_str
        };

        let button = MouseButton::from_str(button_name)?;

        Ok(MouseInput { button, modifiers })
    }
}

#[derive(Debug, EnumString, Clone, Copy, PartialEq, Eq)]
pub enum BindableMessage {
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
}

#[derive(Debug, EnumString, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMode {
    Base,
    Sketch,
    Point,
}

impl Mode for AppMode {}

#[derive(Debug, EnumString)]
enum Command {
    Bind,
    MouseBind,
    Set,
}

#[derive(Debug, Clone)]
pub struct ConfigError {
    pub line_number: usize,
    pub message: String,
}

impl ConfigError {
    pub fn new(line_number: usize, message: String) -> Self {
        Self {
            line_number,
            message,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {}",
            "Line".bright_blue(),
            self.line_number.to_string().bright_yellow(),
            self.message.bright_red()
        )
    }
}

#[derive(Debug)]
pub struct ConfigParseResult {
    pub config: Config,
    pub errors: Vec<ConfigError>,
}

impl ConfigParseResult {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, line_number: usize, message: String) {
        self.errors.push(ConfigError::new(line_number, message));
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn format_errors(&self) -> String {
        if self.errors.is_empty() {
            return String::new();
        }

        let mut output = format!("{}\n", "Configuration parsing errors:".bright_red().bold());
        for error in &self.errors {
            output.push_str(&format!("  {error}\n"));
        }
        output
    }
}

#[derive(Debug)]
pub struct Config {
    bindings: HashMap<AppMode, Keybinds<BindableMessage>>,
    pub mouse: HashMap<AppMode, Vec<MouseBinding>>,
}

impl Config {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            mouse: HashMap::new(),
        }
    }

    /// Splits a line from the config file into parts, taking into quoted strings into accound since they are needed for multi key bindings
    fn parse_line_parts(line: &str) -> Result<Vec<String>, String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;
        let chars = line.chars();

        for ch in chars {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' | '\t' if !in_quotes => {
                    if !current_part.is_empty() {
                        parts.push(current_part.clone());
                        current_part.clear();
                    }
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if in_quotes {
            return Err("Unterminated quoted string".to_string());
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        Ok(parts)
    }

    fn parse_line(line: &str, config: &mut Config) -> Result<(), String> {
        let parts = Self::parse_line_parts(line)?;
        if parts.is_empty() {
            return Ok(());
        }

        let command = match Command::from_str(&parts[0]) {
            Ok(cmd) => cmd,
            Err(_) => return Err(format!("Unknown command: {}", parts[0])),
        };

        match command {
            Command::Bind => {
                if parts.len() != 4 {
                    return Err(
                        "Bind command requires exactly 3 arguments: <key> <mode> <action>"
                            .to_string(),
                    );
                }

                let mode_str = &parts[1];
                let key_str = &parts[2];
                let action_str = &parts[3];

                let mode =
                    AppMode::from_str(mode_str).map_err(|_| format!("Unknown mode: {mode_str}"))?;
                let action = BindableMessage::from_str(action_str)
                    .map_err(|_| format!("Unknown action: {action_str}"))?;

                match config.bindings.get_mut(&mode) {
                    Some(keyboard) => {
                        keyboard
                            .bind(key_str, action)
                            .map_err(|e| format!("Failed to bind key '{key_str}': {e}"))?;
                    }
                    None => {
                        let mut new_kb = Keybinds::new(vec![]);
                        new_kb
                            .bind(key_str, action)
                            .map_err(|e| format!("Failed to bind key '{key_str}': {e}"))?;
                        config.bindings.insert(mode, new_kb);
                    }
                }
            }
            Command::MouseBind => {
                if parts.len() != 4 {
                    return Err(
                        "MouseBind command requires exactly 3 arguments: <mouse_input> <mode> <action>"
                            .to_string(),
                    );
                }

                let mode_str = &parts[1];
                let mouse_input_str = &parts[2];
                let action_str = &parts[3];

                let mode =
                    AppMode::from_str(mode_str).map_err(|_| format!("Unknown mode: {mode_str}"))?;
                let mouse_input = MouseInput::from_str(mouse_input_str)
                    .map_err(|e| format!("Invalid mouse input '{mouse_input_str}': {e}"))?;
                let mouse_action = MouseAction::from_str(action_str)
                    .map_err(|_| format!("Unknown mouse action: {action_str}"))?;

                match config.mouse.get_mut(&mode) {
                    Some(mouse_list) => {
                        mouse_list.push((mouse_input, mouse_action));
                    }
                    None => {
                        let mut new_ml = vec![];
                        new_ml.push((mouse_input, mouse_action));
                        config.mouse.insert(mode, new_ml);
                    }
                }
            }
            Command::Set => {
                if parts.len() != 3 {
                    return Err(
                        "Set command requires exactly 2 arguments: <setting> <value>".to_string(),
                    );
                }

                let setting = &parts[1];
                let _value = &parts[2];

                match setting.as_str() {
                    _ => return Err(format!("Unknown setting: {setting}")),
                }
            }
        }

        Ok(())
    }

    pub fn parse_with_errors(s: &str) -> ConfigParseResult {
        let mut result = ConfigParseResult::new();
        let lines: Vec<&str> = s.lines().collect();

        for (line_number, line) in lines.iter().enumerate() {
            let line_num = line_number + 1; // 1-based line numbers
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Err(error) = Self::parse_line(trimmed, &mut result.config) {
                result.add_error(line_num, error);
            }
        }

        result
    }
}

impl Default for Config {
    fn default() -> Self {
        let base_keybinds = vec![
            Keybind::new(KeyInput::from_str("Escape").unwrap(), BindableMessage::PopMode),
            Keybind::new(KeyInput::from_str("F8").unwrap(), BindableMessage::ToggleSettings),
            Keybind::new(KeyInput::from_str("F9").unwrap(), BindableMessage::ToggleProjection),
            Keybind::new(KeyInput::from_str("F10").unwrap(), BindableMessage::ToggleDebugDraw),
            Keybind::new(KeyInput::from_str("F11").unwrap(), BindableMessage::DumpDebugPick),
            Keybind::new(KeyInput::from_str("F12").unwrap(), BindableMessage::TogglePerformanceOverlay),
            Keybind::new(KeyInput::from_str("h").unwrap(), BindableMessage::SplitAreaHorizontally),
            Keybind::new(KeyInput::from_str("v").unwrap(), BindableMessage::SplitAreaVertically),
            Keybind::new(KeyInput::from_str("d").unwrap(), BindableMessage::CollapseBoundary),
        ];
        let base_bindings = Keybinds::new(base_keybinds);

        let sketch_keybinds = vec![
            Keybind::new(KeyInput::from_str("p").unwrap(), BindableMessage::ActivatePointMode),
        ];
        let sketch_bindings = Keybinds::new(sketch_keybinds);

        let mut bindings = HashMap::new();
        bindings.insert(AppMode::Base, base_bindings);
        bindings.insert(AppMode::Sketch, sketch_bindings);

        let mut mouse = HashMap::new();
        mouse.insert(AppMode::Base, vec![
            (MouseInput::from_str("Middle").unwrap(), MouseAction::Pan),
            (MouseInput::from_str("Shift+Middle").unwrap(), MouseAction::Orbit),
        ]);
        mouse.insert(AppMode::Point, vec![
            (MouseInput::from_str("Left").unwrap(), MouseAction::PlacePoint),
        ]);

        Self { bindings, mouse }
    }
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_result = Self::parse_with_errors(s);

        if parse_result.has_errors() {
            return Err(anyhow!("{}", parse_result.format_errors()));
        }

        Ok(parse_result.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_parse_config_file() {
        let contents = include_str!("../assets/default.conf");
        let config = Config::from_str(contents).unwrap();
        let default_cfg = Config::default();

        assert_eq!(config.bindings, default_cfg.bindings);
        assert_eq!(config.mouse, default_cfg.mouse);
    }
}
