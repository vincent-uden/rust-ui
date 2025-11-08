use glfw::{Action, Key, Modifiers, MouseButton};

use keybinds::{Key as KeybindsKey, KeyInput, Mods};

use crate::modes::{MouseButton as MyMouseButton, MouseInput, MouseModifiers};

/// Converts a GLFW key and modifiers to a KeyInput from the keybinds crate.
/// Returns None if the key is not supported.
pub fn glfw_key_to_key_input(key: Key, modifiers: Modifiers) -> Option<KeyInput> {
    let base_key = match key {
        Key::A => KeybindsKey::Char('a'),
        Key::B => KeybindsKey::Char('b'),
        Key::C => KeybindsKey::Char('c'),
        Key::D => KeybindsKey::Char('d'),
        Key::E => KeybindsKey::Char('e'),
        Key::F => KeybindsKey::Char('f'),
        Key::G => KeybindsKey::Char('g'),
        Key::H => KeybindsKey::Char('h'),
        Key::I => KeybindsKey::Char('i'),
        Key::J => KeybindsKey::Char('j'),
        Key::K => KeybindsKey::Char('k'),
        Key::L => KeybindsKey::Char('l'),
        Key::M => KeybindsKey::Char('m'),
        Key::N => KeybindsKey::Char('n'),
        Key::O => KeybindsKey::Char('o'),
        Key::P => KeybindsKey::Char('p'),
        Key::Q => KeybindsKey::Char('q'),
        Key::R => KeybindsKey::Char('r'),
        Key::S => KeybindsKey::Char('s'),
        Key::T => KeybindsKey::Char('t'),
        Key::U => KeybindsKey::Char('u'),
        Key::V => KeybindsKey::Char('v'),
        Key::W => KeybindsKey::Char('w'),
        Key::X => KeybindsKey::Char('x'),
        Key::Y => KeybindsKey::Char('y'),
        Key::Z => KeybindsKey::Char('z'),
        Key::Num0 => KeybindsKey::Char('0'),
        Key::Num1 => KeybindsKey::Char('1'),
        Key::Num2 => KeybindsKey::Char('2'),
        Key::Num3 => KeybindsKey::Char('3'),
        Key::Num4 => KeybindsKey::Char('4'),
        Key::Num5 => KeybindsKey::Char('5'),
        Key::Num6 => KeybindsKey::Char('6'),
        Key::Num7 => KeybindsKey::Char('7'),
        Key::Num8 => KeybindsKey::Char('8'),
        Key::Num9 => KeybindsKey::Char('9'),
        Key::Escape => KeybindsKey::Esc,
        Key::F1 => KeybindsKey::F1,
        Key::F2 => KeybindsKey::F2,
        Key::F3 => KeybindsKey::F3,
        Key::F4 => KeybindsKey::F4,
        Key::F5 => KeybindsKey::F5,
        Key::F6 => KeybindsKey::F6,
        Key::F7 => KeybindsKey::F7,
        Key::F8 => KeybindsKey::F8,
        Key::F9 => KeybindsKey::F9,
        Key::F10 => KeybindsKey::F10,
        Key::F11 => KeybindsKey::F11,
        Key::F12 => KeybindsKey::F12,
        Key::Backspace => KeybindsKey::Backspace,
        Key::Tab => KeybindsKey::Tab,
        Key::Enter => KeybindsKey::Enter,
        Key::Space => KeybindsKey::Char(' '),
        Key::Left => KeybindsKey::Left,
        Key::Right => KeybindsKey::Right,
        Key::Up => KeybindsKey::Up,
        Key::Down => KeybindsKey::Down,
        _ => return None,
    };

    let mut mods = Mods::empty();
    if modifiers.contains(Modifiers::Control) {
        mods |= Mods::CTRL;
    }
    if modifiers.contains(Modifiers::Shift) {
        mods |= Mods::SHIFT;
    }
    if modifiers.contains(Modifiers::Alt) {
        mods |= Mods::ALT;
    }

    Some(KeyInput::new(base_key, mods))
}

/// Converts a GLFW mouse button and modifiers to a MouseInput.
/// Returns None if the button is not supported or action is not press/release.
pub fn glfw_mouse_to_mouse_input(
    button: MouseButton,
    modifiers: Modifiers,
    action: Action,
) -> Option<MouseInput> {
    if action != Action::Press && action != Action::Release {
        return None;
    }

    let mouse_button = match button {
        MouseButton::Button1 => MyMouseButton::Left,
        MouseButton::Button2 => MyMouseButton::Middle,
        MouseButton::Button3 => MyMouseButton::Right,
        MouseButton::Button4 => MyMouseButton::Back,
        MouseButton::Button5 => MyMouseButton::Forward,
        _ => return None,
    };

    let modifiers = MouseModifiers {
        ctrl: modifiers.contains(Modifiers::Control),
        shift: modifiers.contains(Modifiers::Shift),
    };

    Some(MouseInput {
        button: mouse_button,
        modifiers,
    })
}

/// Converts GLFW scroll events to a MouseInput.
/// Returns None if no scroll occurred.
pub fn glfw_scroll_to_mouse_input(x: f64, y: f64, modifiers: Modifiers) -> Option<MouseInput> {
    if x.abs() < 0.01 && y.abs() < 0.01 {
        return None;
    }

    let button = if y > 0.0 {
        MyMouseButton::ScrollUp
    } else if y < 0.0 {
        MyMouseButton::ScrollDown
    } else {
        return None;
    };

    let modifiers = MouseModifiers {
        ctrl: modifiers.contains(Modifiers::Control),
        shift: modifiers.contains(Modifiers::Shift),
    };

    Some(MouseInput {
        button,
        modifiers,
    })
}