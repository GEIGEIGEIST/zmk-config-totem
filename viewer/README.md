# totem-viewer

A small always-on-top window that shows the TOTEM's **active layer live**,
with every key's tap and hold action. It talks to the RMK firmware over
**Rynk** on the dongle's USB connection, so there's nothing to configure:
plug in the dongle, start the viewer.

- Follows the keyboard: hold a layer key and the window switches with you
  (the firmware pushes layer changes, no polling).
- Real TOTEM geometry (column stagger and thumb fan), read from the firmware.
- Home-row mods and layer-taps show the tap label large and the hold
  (modifier or target layer, by name) small underneath.
- Transparent keys (▽, dimmed) show what they fall through to.
- Click a layer chip to preview another layer; click it again to follow the
  keyboard. Hover a key for its matrix position and raw action.
- Header shows Caps Lock, whether each half is connected and its battery,
  and a ⟳ button that re-reads the keymap.
- Reconnects on its own when the dongle is unplugged and plugged back in.

Pure Rust: [`eframe`/`egui`](https://github.com/emilk/egui) for the window,
RMK's own [`rynk`](https://crates.io/crates/rynk) + [`rynk-usb`](https://crates.io/crates/rynk-usb)
(on top of the pure-Rust [`nusb`](https://crates.io/crates/nusb)) for the
protocol, `tokio`, `serde` + `toml`. No libusb, no C dependencies.

## Get it

**Download:** every push that touches `viewer/` builds Linux, Windows and
macOS binaries. Open the latest *Build viewer* run under Actions and download
`totem-viewer-<os>`.

**Build it yourself:**

```sh
cd viewer
cargo run --release
```

## Platform setup

**Linux:** USB devices are root-only by default. Add a udev rule once:

```sh
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="4c4b", ATTR{idProduct}=="4654", MODE="0660", TAG+="uaccess"' \
  | sudo tee /etc/udev/rules.d/70-totem-rynk.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Then unplug and replug the dongle. (VID/PID come from `rmk/keyboard.toml`.)

**Windows:** nothing to do. The firmware tells Windows to use the built-in
WinUSB driver for the Rynk interface.

**macOS:** nothing to do.

## Options

```text
totem-viewer [--config <keyboard.toml>] [--no-top]

--config <path>   read layer names from this keyboard.toml instead of the one
                  bundled at build time
--no-top          don't keep the window above other windows
```

Rynk doesn't send layer names, so they come from `rmk/keyboard.toml`. It's
baked into the binary at build time. Rebuild the viewer (or pass `--config`) after
renaming layers. Everything else (keys, layout, active layer) is read live
from the keyboard.

## Firmware requirements

- RMK `0.9.0` built with the `rynk` feature and `[host] rynk_enabled = true`
  (already the case on the `rmk` branch).
- The physical layout comes from `[layout]` in `keyboard.toml`. If the dongle
  runs a build from before the `[layout.shapes]` were added, keys show up in a
  plain grid. Reflash the **dongle** to get the real shape; the halves don't
  need it.

The `rynk` crates are pinned to `=0.3.0`, which matches `rmk 0.9.0`. When
upgrading RMK, bump them to the matching release.

## Code layout

| File | What |
| --- | --- |
| `src/device.rs` | Connection thread: discover → connect → read layout/keymap → follow topics → reconnect. `Command` is where editor actions will go. |
| `src/app.rs` | The egui window and keyboard drawing. |
| `src/labels.rs` | `KeyAction` → key-cap labels (with unit tests). |
| `src/config.rs` | Layer names from `keyboard.toml`. |

### Toward an editor

`rynk::Client` already has `set_key`, `set_keymap_bulk`, `set_combo`,
`set_morse`, `set_behavior`, … The plan: add variants to `device::Command`
(e.g. `SetKey { layer, row, col, action }`), handle them in `run_session` next
to `Reload`, and make the keys in `app.rs` clickable.
