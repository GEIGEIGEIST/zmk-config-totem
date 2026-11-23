#![no_main]
#![no_std]

// left half: split peripheral 0.
use rmk::macros::rmk_peripheral;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {}
