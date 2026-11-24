//! # Wheel Keyboard
//!
//! A two-thumb keyboard for touchscreens. Each thumb gets a wheel with eight
//! directions and two rings: touch, slide towards the letter, lift. An
//! on-device model learns how you write, highlights the likely next letter,
//! fixes taps that land right on the border between two letters, and suggests
//! words. Nothing leaves the device.
//!
//! This crate is the engine only: no drawing, no platform code, no
//! dependencies. You feed it touches and it tells you what to type.
//!
//! ```
//! use wheel_keyboard::{Action, Keyboard, Side};
//!
//! let mut kb = Keyboard::default();
//! // Left thumb (consonants): touch, slide up 40 px on a 100 px wheel, lift.
//! kb.touch_down(Side::Left, 200.0, 300.0, 100.0, 0);
//! kb.touch_move(Side::Left, 200.0, 260.0, 16);
//! assert_eq!(kb.touch_up(Side::Left, 32), vec![Action::Text("S".into())]);
//! ```
//!
//! Licensed under Apache-2.0. If you use it, keep the `NOTICE` file and credit
//! **Francesco Catania (@sekaibuilder)**.

pub mod geometry;
mod key;
mod keyboard;
mod layout;
mod model;

pub use key::{Key, Modifier, Modifiers};
pub use keyboard::{
    ACCENT_DELAY_MS, Action, Button, CHORD_DELAY_MS, Config, DOUBLE_TAP_MS, Keyboard, Layer,
    ModState, MoveFeedback, Preview, REPEAT_DELAY_MS, REPEAT_INTERVAL_MS, Selection, Side,
};
pub use layout::{Context, Layout, Ring, RingId, RingPair, Rings};
pub use model::{ALPHABET, Model, Prediction, fold};
