//! Turn RMK key actions into short, human-readable key-cap labels.

use rynk::rmk_types::action::{Action, KeyAction, KeyboardAction};
use rynk::rmk_types::keycode::{HidKeyCode, KeyCode, SpecialKey};
use rynk::rmk_types::modifier::ModifierCombination;

/// What a key cap shows: the tap label, an optional hold label, and a kind
/// used for coloring.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyLabel {
    pub tap: String,
    pub hold: Option<String>,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Ordinary key (letters, symbols, F-keys, ...).
    Normal,
    /// Plain modifier key.
    Modifier,
    /// A key whose main job is switching layers (MO, TO, TG, ...).
    Layer,
    /// Keyboard control: bootloader, caps word, BLE output, ...
    Control,
    /// `_` – falls through to the layer below.
    Transparent,
    /// `No` – does nothing.
    Empty,
}

/// Look up a layer's display name; falls back to the index.
pub trait LayerNames {
    fn layer_name(&self, layer: u8) -> String;
}

impl LayerNames for [String] {
    fn layer_name(&self, layer: u8) -> String {
        self.get(layer as usize)
            .cloned()
            .unwrap_or_else(|| layer.to_string())
    }
}

pub fn key_label(action: &KeyAction, names: &[String]) -> KeyLabel {
    match action {
        KeyAction::No => KeyLabel {
            tap: String::new(),
            hold: None,
            kind: Kind::Empty,
        },
        KeyAction::Transparent => KeyLabel {
            tap: "▽".into(),
            hold: None,
            kind: Kind::Transparent,
        },
        KeyAction::Single(a) | KeyAction::Tap(a) => {
            let (tap, kind) = action_label(a, names);
            KeyLabel { tap, hold: None, kind }
        }
        KeyAction::TapHold(tap, hold, _profile) => {
            let (tap_text, tap_kind) = action_label(tap, names);
            let (hold_text, hold_kind) = action_label(hold, names);
            KeyLabel {
                tap: tap_text,
                hold: Some(hold_text),
                // Layer-taps are colored as layer keys so they stand out.
                kind: if hold_kind == Kind::Layer { Kind::Layer } else { tap_kind },
            }
        }
        KeyAction::Morse(i) => KeyLabel {
            tap: format!("Morse {i}"),
            hold: None,
            kind: Kind::Control,
        },
        // `KeyAction` is non-exhaustive.
        #[allow(unreachable_patterns)]
        other => KeyLabel {
            tap: format!("{other:?}"),
            hold: None,
            kind: Kind::Normal,
        },
    }
}

pub fn action_label(action: &Action, names: &[String]) -> (String, Kind) {
    use Action as A;
    let layer = |n: &u8| names.layer_name(*n);
    match action {
        A::No => (String::new(), Kind::Empty),
        A::Key(KeyCode::Hid(h)) => {
            let kind = if is_modifier_key(*h) { Kind::Modifier } else { Kind::Normal };
            (hid_label(*h), kind)
        }
        A::Key(KeyCode::Consumer(c)) => (format!("{c:?}"), Kind::Normal),
        A::Key(KeyCode::SystemControl(s)) => (format!("{s:?}"), Kind::Control),
        A::Modifier(m) => (mods_label(*m), Kind::Modifier),
        A::KeyWithModifier(h, m) => (key_with_mods_label(*h, *m), Kind::Normal),
        A::LayerOn(n) => (layer(n), Kind::Layer),
        A::LayerOnWithModifier(n, m) => (format!("{}+{}", layer(n), mods_label(*m)), Kind::Layer),
        A::LayerOff(n) => (format!("off {}", layer(n)), Kind::Layer),
        A::LayerToggle(n) => (format!("TG {}", layer(n)), Kind::Layer),
        A::DefaultLayer(n) | A::PersistentDefaultLayer(n) => (format!("DF {}", layer(n)), Kind::Layer),
        A::LayerToggleOnly(n) => (format!("→ {}", layer(n)), Kind::Layer),
        A::TriLayerLower => ("Lower".into(), Kind::Layer),
        A::TriLayerUpper => ("Upper".into(), Kind::Layer),
        A::OneShotLayer(n) => (format!("OS {}", layer(n)), Kind::Layer),
        A::OneShotModifier(m) => (format!("OS {}", mods_label(*m)), Kind::Modifier),
        A::OneShotKey(h) => (format!("OS {}", hid_label(*h)), Kind::Normal),
        A::TriggerMacro(i) => (format!("Macro {i}"), Kind::Control),
        A::KeyboardControl(k) => (keyboard_action_label(*k), Kind::Control),
        A::Special(SpecialKey::GraveEscape) => ("Esc/`".into(), Kind::Normal),
        A::Special(SpecialKey::Repeat) => ("Repeat".into(), Kind::Normal),
        A::User(i) => (format!("User {i}"), Kind::Control),
        A::Light(l) => (format!("{l:?}"), Kind::Control),
        #[allow(unreachable_patterns)]
        other => (format!("{other:?}"), Kind::Normal),
    }
}

fn keyboard_action_label(k: KeyboardAction) -> String {
    match k {
        KeyboardAction::Bootloader => "Boot".into(),
        KeyboardAction::Reboot => "Reboot".into(),
        KeyboardAction::CapsWordToggle => "Caps Word".into(),
        KeyboardAction::OutputUsb => "USB".into(),
        KeyboardAction::OutputBluetooth => "BT".into(),
        KeyboardAction::OutputAuto => "Out Auto".into(),
        KeyboardAction::ComboToggle => "Combo ⇄".into(),
        other => format!("{other:?}"),
    }
}

fn is_modifier_key(h: HidKeyCode) -> bool {
    use HidKeyCode as H;
    matches!(
        h,
        H::LCtrl | H::LShift | H::LAlt | H::LGui | H::RCtrl | H::RShift | H::RAlt | H::RGui
    )
}

/// Modifier combination as text, e.g. `LShift`, `Ctrl+Alt`.
pub fn mods_label(m: ModifierCombination) -> String {
    let pairs = [
        ("Ctrl", m.left_ctrl(), m.right_ctrl()),
        ("Shift", m.left_shift(), m.right_shift()),
        ("Alt", m.left_alt(), m.right_alt()),
        ("Gui", m.left_gui(), m.right_gui()),
    ];
    let active: Vec<_> = pairs.iter().filter(|(_, l, r)| *l || *r).collect();
    match active.as_slice() {
        [] => String::new(),
        // A single modifier keeps its side, which matters for home-row mods.
        [(name, l, r)] => match (l, r) {
            (true, false) => format!("L{name}"),
            (false, true) => format!("R{name}"),
            _ => name.to_string(),
        },
        many => many.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join("+"),
    }
}

fn key_with_mods_label(h: HidKeyCode, m: ModifierCombination) -> String {
    let only_shift = (m.left_shift() || m.right_shift())
        && !(m.left_ctrl() || m.right_ctrl() || m.left_alt() || m.right_alt() || m.left_gui() || m.right_gui());
    if only_shift {
        if let Some(s) = shifted_symbol(h) {
            return s.into();
        }
        return format!("⇧{}", hid_label(h));
    }
    format!("{}+{}", mods_label(m), hid_label(h))
}

/// What a key produces with Shift held, on a US layout.
fn shifted_symbol(h: HidKeyCode) -> Option<&'static str> {
    use HidKeyCode as H;
    Some(match h {
        H::Kc1 => "!",
        H::Kc2 => "@",
        H::Kc3 => "#",
        H::Kc4 => "$",
        H::Kc5 => "%",
        H::Kc6 => "^",
        H::Kc7 => "&",
        H::Kc8 => "*",
        H::Kc9 => "(",
        H::Kc0 => ")",
        H::Minus => "_",
        H::Equal => "+",
        H::LeftBracket => "{",
        H::RightBracket => "}",
        H::Backslash => "|",
        H::Semicolon => ":",
        H::Quote => "\"",
        H::Grave => "~",
        H::Comma => "<",
        H::Dot => ">",
        H::Slash => "?",
        _ => return None,
    })
}

/// Short label for a plain HID key code.
pub fn hid_label(h: HidKeyCode) -> String {
    use HidKeyCode as H;
    let s = match h {
        H::No => "",
        H::Kc1 => "1",
        H::Kc2 => "2",
        H::Kc3 => "3",
        H::Kc4 => "4",
        H::Kc5 => "5",
        H::Kc6 => "6",
        H::Kc7 => "7",
        H::Kc8 => "8",
        H::Kc9 => "9",
        H::Kc0 => "0",
        H::Enter => "Enter",
        H::Escape => "Esc",
        H::Backspace => "Bksp",
        H::Tab => "Tab",
        H::Space => "Space",
        H::Minus => "-",
        H::Equal => "=",
        H::LeftBracket => "[",
        H::RightBracket => "]",
        H::Backslash => "\\",
        H::NonusHash => "#",
        H::Semicolon => ";",
        H::Quote => "'",
        H::Grave => "`",
        H::Comma => ",",
        H::Dot => ".",
        H::Slash => "/",
        H::CapsLock => "Caps",
        H::PrintScreen => "PrtSc",
        H::ScrollLock => "ScrLk",
        H::Pause => "Pause",
        H::Insert => "Ins",
        H::Home => "Home",
        H::PageUp => "PgUp",
        H::Delete => "Del",
        H::End => "End",
        H::PageDown => "PgDn",
        H::Right => "→",
        H::Left => "←",
        H::Down => "↓",
        H::Up => "↑",
        H::NumLock => "NumLk",
        H::KpSlash => "kp /",
        H::KpAsterisk => "kp *",
        H::KpMinus => "kp -",
        H::KpPlus => "kp +",
        H::KpEnter => "kp ⏎",
        H::Kp1 => "kp 1",
        H::Kp2 => "kp 2",
        H::Kp3 => "kp 3",
        H::Kp4 => "kp 4",
        H::Kp5 => "kp 5",
        H::Kp6 => "kp 6",
        H::Kp7 => "kp 7",
        H::Kp8 => "kp 8",
        H::Kp9 => "kp 9",
        H::Kp0 => "kp 0",
        H::KpDot => "kp .",
        H::KpEqual => "kp =",
        H::NonusBackslash => "\\",
        H::Application => "Menu",
        H::AudioMute | H::KbMute => "Mute",
        H::AudioVolUp | H::KbVolumeUp => "Vol+",
        H::AudioVolDown | H::KbVolumeDown => "Vol-",
        H::MediaNextTrack => "Next",
        H::MediaPrevTrack => "Prev",
        H::MediaPlayPause => "Play",
        H::MediaStop => "Stop",
        H::BrightnessUp => "Bri+",
        H::BrightnessDown => "Bri-",
        H::LCtrl => "LCtrl",
        H::LShift => "LShift",
        H::LAlt => "LAlt",
        H::LGui => "LGui",
        H::RCtrl => "RCtrl",
        H::RShift => "RShift",
        H::RAlt => "RAlt",
        H::RGui => "RGui",
        other => return format!("{other:?}"),
    };
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["base".into(), "nav".into()]
    }

    #[test]
    fn home_row_mod() {
        let a = KeyAction::TapHold(
            Action::Key(KeyCode::Hid(HidKeyCode::A)),
            Action::Modifier(ModifierCombination::LSHIFT),
            0,
        );
        let l = key_label(&a, &names());
        assert_eq!(l.tap, "A");
        assert_eq!(l.hold.as_deref(), Some("LShift"));
        assert_eq!(l.kind, Kind::Normal);
    }

    #[test]
    fn layer_tap_uses_layer_name() {
        let a = KeyAction::TapHold(Action::Key(KeyCode::Hid(HidKeyCode::Space)), Action::LayerOn(1), 0);
        let l = key_label(&a, &names());
        assert_eq!(l.tap, "Space");
        assert_eq!(l.hold.as_deref(), Some("nav"));
        assert_eq!(l.kind, Kind::Layer);
    }

    #[test]
    fn shifted_keys_show_symbol() {
        let a = KeyAction::Single(Action::KeyWithModifier(HidKeyCode::LeftBracket, ModifierCombination::LSHIFT));
        assert_eq!(key_label(&a, &names()).tap, "{");
        let a = KeyAction::Single(Action::KeyWithModifier(HidKeyCode::A, ModifierCombination::LCTRL));
        assert_eq!(key_label(&a, &names()).tap, "LCtrl+A");
    }

    #[test]
    fn to_and_unknown_layer() {
        let a = KeyAction::Single(Action::LayerToggleOnly(0));
        assert_eq!(key_label(&a, &names()).tap, "→ base");
        let a = KeyAction::Single(Action::LayerOn(7));
        assert_eq!(key_label(&a, &names()).tap, "7");
    }
}
