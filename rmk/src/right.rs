#![no_main]
#![no_std]

// right half: split peripheral 1.
use rmk::macros::rmk_peripheral;

#[rmk_peripheral(id = 1)]
mod keyboard_peripheral {}
