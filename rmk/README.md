# TOTEM – RMK v0.9.0 firmware

Same setup as the ZMK config: a XIAO nRF52840 dongle is the split central (USB to the
computer, with an OLED), and the two halves are BLE peripherals.

| Firmware | Board | Role |
| --- | --- | --- |
| `totem_dongle.uf2` | dongle XIAO | split central, USB, OLED |
| `totem_left.uf2` | left half XIAO | peripheral 0 |
| `totem_right.uf2` | right half XIAO | peripheral 1 |

Everything (pins, display, keymap, combos, tap-hold timing) is in `keyboard.toml`.

## OLED wiring (DEBO OLED2 0.96 / SSD1306, I2C)

| OLED | Dongle XIAO |
| --- | --- |
| VCC | 3V3 |
| GND | GND |
| SDA | D4 |
| SCL | D5 |

The old nice!view (SPI) wiring is not used anymore. The built-in `OledRenderer` shows
layer, modifiers, Caps Lock, battery and split-connection state.

## Build

GitHub Actions (`.github/workflows/rmk.yml`) builds on every push that touches `rmk/`;
download `totem-rmk-firmware` from the run's artifacts.

Locally:

```sh
sudo apt install gcc-arm-none-eabi libnewlib-arm-none-eabi   # needed by a BLE crypto crate
cargo install --locked flip-link cargo-make cargo-binutils cargo-hex-to-uf2
cd rmk
cargo make uf2 --release
```

## Flashing (first time, coming from ZMK)

1. In `keyboard.toml` set `[storage] clear_storage = true`, build, and flash all three
   boards (double-tap reset → drag the `.uf2` onto the drive). This wipes old ZMK data.
2. Set `clear_storage = false`, build again, flash all three again.
3. Power the halves and plug in the dongle. The dongle scans for both halves and pairs
   them on its own.

Going back to ZMK later: RMK uses the XIAO's SoftDevice flash area, so ZMK must then be
built with the `nrf52840-nosd` snippet (also for `settings_reset`), see the
[RMK FAQ](https://github.com/rmk-rs/rmk/blob/rmk-v0.9.0/docs/docs/main/docs/getting_started/faq.md).

## Rynk (live keymap editing)

The firmware uses Rynk instead of Vial. Rynk is experimental in v0.9.0: there is no
desktop app yet, and the browser demo (`rynk-wasm`) only connects over Bluetooth, not
USB. Because the dongle talks to the computer over USB, this repo has its own native
tool, [`viewer/`](../viewer), built on the `rynk` + `rynk-usb` crates. For now it
shows the active layer live (read-only). To change keys, edit `keyboard.toml` and
reflash.

The physical key positions the viewer draws come from `[layout]` / `[layout.shapes]`
in `keyboard.toml` and are baked into the **dongle** firmware.

Locked operations (bootloader, settings reset, key tester) are unlocked by holding
**Q + the top-right key** (`[host].unlock_keys`).

To switch to Vial instead: `vial_enabled = true` / `rynk_enabled = false` in `[host]`,
swap `rynk` for `vial` in `Cargo.toml`, and add a `vial.json`
(`rmkit layout convert --to-vial keyboard.toml`).

## Differences from the ZMK keymap

- Combos match key actions, not positions, and share one timeout (50 ms).
- Caps Lock combo: the right outer pinky key on the base layer is now `RShift`
  (was `LShift`) so the two keys of the combo differ.
- The 8-key "back to base" combo and the empty ä/ö/ü combos were dropped; every layer
  already has `TO(0)` on the same key (right half, bottom row, 2nd from the outside).
- `&kt LEFT_GUI` (key toggle) has no RMK equivalent: it's a plain `LGui` for now.
- `&mt INSERT RIGHT_GUI` on the qwerty layer became `TH(RGui, Insert)` (same behavior).
- The dongle has a 1×1 dummy matrix on D0/D1 because `keyboard.toml` needs a matrix on
  every split board. Leave it unwired.
