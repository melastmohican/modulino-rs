//! Modulino Joystick driver.
//!
//! The Modulino Joystick module is an analog joystick with a push button.

use crate::{addresses, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Driver for the Modulino Joystick module.
///
/// The joystick reports X and Y values in the range -128 to 127,
/// where (0, 0) is the center position.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::Joystick;
///
/// let mut joystick = Joystick::new(i2c)?;
///
/// loop {
///     joystick.update()?;
///     
///     let x = joystick.x();
///     let y = joystick.y();
///     
///     if joystick.button_pressed() {
///         println!("Button pressed at position ({}, {})", x, y);
///     }
/// }
/// ```
pub struct Joystick<I2C> {
    device: I2cDevice<I2C>,
    x: i8,
    y: i8,
    button_pressed: bool,
    deadzone: u8,
}

impl<I2C, E> Joystick<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Default deadzone threshold.
    pub const DEFAULT_DEADZONE: u8 = 10;

    /// Create a new Joystick instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::JOYSTICK)
    }

    /// Create a new Joystick instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut joystick = Self {
            device: I2cDevice::new(i2c, address),
            x: 0,
            y: 0,
            button_pressed: false,
            deadzone: Self::DEFAULT_DEADZONE,
        };

        // Read initial state
        joystick.update()?;

        Ok(joystick)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Apply deadzone logic to normalize coordinates.
    fn normalize_coordinate(&self, raw: u8) -> i8 {
        // Convert from 0-255 range to -128 to 127 range
        let centered = (raw as i16) - 128;

        // Apply deadzone
        if centered.abs() < self.deadzone as i16 {
            0
        } else {
            centered as i8
        }
    }

    /// Update the joystick state.
    ///
    /// This should be called periodically to read the latest values.
    /// Returns `true` if the state has changed.
    pub fn update(&mut self) -> Result<bool, E> {
        let previous_x = self.x;
        let previous_y = self.y;
        let previous_button = self.button_pressed;

        let mut buf = [0u8; 4]; // 1 pinstrap + 2 axes + 1 button
        self.device.read(&mut buf)?;

        // Skip first byte (pinstrap address)
        let raw_x = buf[1];
        let raw_y = buf[2];

        self.x = self.normalize_coordinate(raw_x);
        self.y = self.normalize_coordinate(raw_y);
        self.button_pressed = buf[3] != 0;

        Ok(self.x != previous_x || self.y != previous_y || self.button_pressed != previous_button)
    }

    /// Get the X-axis value (-128 to 127).
    pub fn x(&self) -> i8 {
        self.x
    }

    /// Get the Y-axis value (-128 to 127).
    pub fn y(&self) -> i8 {
        self.y
    }

    /// Get both axis values as a tuple.
    pub fn position(&self) -> (i8, i8) {
        (self.x, self.y)
    }

    /// Check if the button is pressed.
    pub fn button_pressed(&self) -> bool {
        self.button_pressed
    }

    /// Get the deadzone threshold.
    pub fn deadzone(&self) -> u8 {
        self.deadzone
    }

    /// Set the deadzone threshold.
    ///
    /// Values within this distance from center (0, 0) will be reported as 0.
    pub fn set_deadzone(&mut self, deadzone: u8) {
        self.deadzone = deadzone;
    }

    /// Check if the joystick is in the center position (within deadzone).
    pub fn is_centered(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    /// Get the magnitude of joystick displacement from center.
    pub fn magnitude(&self) -> f32 {
        let x = self.x as f32;
        let y = self.y as f32;
        libm::sqrtf(x * x + y * y)
    }

    /// Get the angle of joystick displacement in radians.
    ///
    /// Returns 0 when centered. Angle is measured counter-clockwise from the positive X-axis.
    pub fn angle(&self) -> f32 {
        if self.is_centered() {
            0.0
        } else {
            libm::atan2f(self.y as f32, self.x as f32)
        }
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
