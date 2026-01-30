//! # Modulino
//!
//! A hardware-agnostic, `no_std` Rust driver for Arduino Modulino breakout boards.
//!
//! This crate provides drivers for the following Modulino devices:
//!
//! - [`Buttons`] - Three-button module with LEDs
//! - [`Buzzer`] - Piezo speaker module
//! - [`Pixels`] - 8 RGB LED module (APA102-compatible)
//! - [`Distance`] - Time-of-Flight distance sensor (VL53L4CD)
//! - [`Movement`] - IMU module (LSM6DSOX accelerometer/gyroscope)
//! - [`Knob`] - Rotary encoder with button
//! - [`Thermo`] - Temperature and humidity sensor (wraps [`hs3003`](https://crates.io/crates/hs3003) crate)
//! - [`Joystick`] - Analog joystick with button
//! - [`LatchRelay`] - Latching relay module
//! - [`Vibro`] - Vibration motor module
//!
//! ## Example
//!
//! ```rust,ignore
//! use modulino::{Pixels, Color};
//!
//! // Create a Pixels instance with your I2C bus
//! let mut pixels = Pixels::new(i2c)?;
//!
//! // Set the first LED to red
//! pixels.set_color(0, Color::RED, 50)?;
//!
//! // Apply the changes
//! pixels.show()?;
//! ```
//!
//! ## Features
//!
//! - `defmt`: Enable `defmt` debug formatting for error types
//!
//! ## Hardware Requirements
//!
//! All Modulino devices communicate over I2C at 100kHz. They use the Qwiic/STEMMA QT
//! connector standard for easy daisy-chaining.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(unsafe_code)]

mod buttons;
mod buzzer;
mod color;
mod distance;
mod error;
mod i2c_device;
mod joystick;
mod knob;
mod latch_relay;
mod movement;
mod pixels;
mod thermo;
mod vibro;

pub use buttons::{ButtonLed, ButtonState, Buttons};
pub use buzzer::{Buzzer, Note};
pub use color::Color;
pub use distance::Distance;
pub use error::{Error, Result};
pub use i2c_device::I2cDevice;
pub use joystick::Joystick;
pub use knob::Knob;
pub use latch_relay::LatchRelay;
pub use movement::{Movement, MovementValues};
pub use pixels::Pixels;
pub use thermo::{Hs3003Error, Thermo, ThermoMeasurement};
pub use vibro::{PowerLevel, Vibro};

/// Default I2C addresses for Modulino devices.
///
/// These are the factory-default 7-bit I2C addresses. Some modules support
/// multiple addresses via hardware configuration (solder jumpers).
pub mod addresses {
    /// Default address for Modulino Buttons (0x7C >> 1 = 0x3E)
    pub const BUTTONS: u8 = 0x3E;

    /// Default address for Modulino Buzzer (0x3C >> 1 = 0x1E)
    pub const BUZZER: u8 = 0x1E;

    /// Default address for Modulino Pixels (0x6C >> 1 = 0x36)
    pub const PIXELS: u8 = 0x36;

    /// Default address for Modulino Distance
    pub const DISTANCE: u8 = 0x29;

    /// Default addresses for Modulino Movement (configurable via solder jumper)
    pub const MOVEMENT: [u8; 2] = [0x6A, 0x6B];

    /// Default addresses for Modulino Knob (two possible addresses)
    pub const KNOB: [u8; 2] = [0x3A, 0x3B]; // 0x74 >> 1, 0x76 >> 1

    /// Default address for Modulino Thermo
    pub const THERMO: u8 = 0x44;

    /// Default address for Modulino Joystick (0x58 >> 1 = 0x2C)
    pub const JOYSTICK: u8 = 0x2C;

    /// Default address for Modulino Latch Relay (0x04 >> 1 = 0x02)
    pub const LATCH_RELAY: u8 = 0x02;

    /// Default address for Modulino Vibro (0x70 >> 1 = 0x38)
    pub const VIBRO: u8 = 0x38;
}

/// Pinstrap address map for device type detection.
///
/// When reading from a Modulino with a microcontroller, the first byte
/// returned is always the pinstrap address, which identifies the device type.
pub mod pinstrap {
    /// Pinstrap address for Buzzer
    pub const BUZZER: u8 = 0x3C;

    /// Pinstrap address for Buttons
    pub const BUTTONS: u8 = 0x7C;

    /// Pinstrap addresses for Knob
    pub const KNOB: [u8; 2] = [0x76, 0x74];

    /// Pinstrap address for Pixels
    pub const PIXELS: u8 = 0x6C;

    /// Pinstrap address for Joystick
    pub const JOYSTICK: u8 = 0x58;

    /// Pinstrap address for Latch Relay
    pub const LATCH_RELAY: u8 = 0x04;

    /// Pinstrap address for Vibro
    pub const VIBRO: u8 = 0x70;
}
