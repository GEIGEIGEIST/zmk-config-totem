#![no_main]
#![no_std]

// Dongle: split central, USB to the host, drives the SSD1306 OLED.
use rmk::macros::rmk_central;

#[rmk_central]
mod keyboard_central {}
