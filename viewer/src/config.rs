//! Layer names. Rynk doesn't send them, so they come from `keyboard.toml`:
//! the copy in this repo is baked in at compile time, and `--config <path>`
//! reads another one at runtime.

use serde::Deserialize;

/// The firmware config this viewer was built next to.
const BUNDLED_KEYBOARD_TOML: &str = include_str!("../../rmk/keyboard.toml");

#[derive(Deserialize)]
struct KeyboardToml {
    #[serde(default)]
    keymap: Keymap,
}

#[derive(Deserialize, Default)]
struct Keymap {
    #[serde(default)]
    layer: Vec<Layer>,
}

#[derive(Deserialize)]
struct Layer {
    name: Option<String>,
}

/// Layer names from a `keyboard.toml` string. Unnamed layers get their index.
pub fn layer_names_from_toml(text: &str) -> Result<Vec<String>, String> {
    let parsed: KeyboardToml = toml::from_str(text).map_err(|e| e.to_string())?;
    Ok(parsed
        .keymap
        .layer
        .into_iter()
        .enumerate()
        .map(|(i, l)| l.name.unwrap_or_else(|| i.to_string()))
        .collect())
}

/// Layer names from `path`, or from the bundled config when `path` is `None`.
pub fn load_layer_names(path: Option<&str>) -> Vec<String> {
    let text = match path {
        Some(p) => match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("could not read {p}: {e}; using the bundled keyboard.toml");
                BUNDLED_KEYBOARD_TOML.to_string()
            }
        },
        None => BUNDLED_KEYBOARD_TOML.to_string(),
    };
    layer_names_from_toml(&text).unwrap_or_else(|e| {
        eprintln!("could not parse layer names: {e}");
        Vec::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_config_has_named_layers() {
        let names = layer_names_from_toml(BUNDLED_KEYBOARD_TOML).unwrap();
        assert_eq!(names.first().map(String::as_str), Some("c_base"));
        assert!(names.len() >= 2);
    }
}
