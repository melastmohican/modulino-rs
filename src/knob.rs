//! Modulino Knob driver.
//!
//! The Modulino Knob module is a rotary encoder with a push button.

use crate::{addresses, Error, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Driver for the Modulino Knob module (rotary encoder).
///
/// # Example
///
/// ```rust,ignore
/// use modulino::Knob;
///
/// let mut knob = Knob::new(i2c)?;
///
/// // Set a range for the encoder value
/// knob.set_range(-100, 100);
///
/// loop {
///     knob.update()?;
///     
///     if knob.pressed() {
///         println!("Button pressed!");
///     }
///     
///     println!("Value: {}", knob.value());
/// }
/// ```
pub struct Knob<I2C> {
    device: I2cDevice<I2C>,
    value: i16,
    pressed: bool,
    range: Option<(i16, i16)>,
}

impl<I2C, E> Knob<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Knob instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::KNOB[0])
    }

    /// Create a new Knob instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut knob = Self {
            device: I2cDevice::new(i2c, address),
            value: 0,
            pressed: false,
            range: None,
        };

        // Read initial state
        knob.update()?;

        Ok(knob)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Read the current encoder state from the device.
    fn read_data(&mut self) -> Result<(i16, bool), E> {
        let mut buf = [0u8; 4]; // 1 pinstrap + 2 encoder + 1 button
        self.device.read(&mut buf)?;

        // Skip first byte (pinstrap address)
        let raw_value = i16::from_le_bytes([buf[1], buf[2]]);
        let pressed = buf[3] != 0;

        Ok((raw_value, pressed))
    }

    /// Update the encoder state.
    ///
    /// This should be called periodically to read the latest values.
    /// Returns `true` if the state has changed.
    pub fn update(&mut self) -> Result<bool, E> {
        let previous_value = self.value;
        let previous_pressed = self.pressed;

        let (mut new_value, new_pressed) = self.read_data()?;

        // Apply range constraint if set
        if let Some((min, max)) = self.range {
            if new_value < min {
                new_value = min;
                self.set_value_internal(min)?;
            } else if new_value > max {
                new_value = max;
                self.set_value_internal(max)?;
            }
        }

        self.value = new_value;
        self.pressed = new_pressed;

        Ok(self.value != previous_value || self.pressed != previous_pressed)
    }

    /// Get the current encoder value.
    pub fn value(&self) -> i16 {
        self.value
    }

    /// Set the encoder value.
    pub fn set_value(&mut self, value: i16) -> Result<(), E> {
        // Check range if set
        if let Some((min, max)) = self.range {
            if value < min || value > max {
                return Err(Error::OutOfRange);
            }
        }

        self.set_value_internal(value)?;
        self.value = value;
        Ok(())
    }

    /// Internal method to set the encoder value on the device.
    fn set_value_internal(&mut self, value: i16) -> Result<(), E> {
        let bytes = value.to_le_bytes();
        let data = [bytes[0], bytes[1], 0, 0];
        self.device.write(&data)?;
        Ok(())
    }

    /// Reset the encoder value to 0.
    pub fn reset(&mut self) -> Result<(), E> {
        self.set_value(0)
    }

    /// Check if the button is currently pressed.
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    /// Set the value range for the encoder.
    ///
    /// The encoder value will be constrained to this range.
    /// Pass `None` to remove the range constraint.
    pub fn set_range(&mut self, min: i16, max: i16) {
        self.range = Some((min, max));

        // Constrain current value to new range
        if self.value < min {
            self.value = min;
        } else if self.value > max {
            self.value = max;
        }
    }

    /// Clear the range constraint.
    pub fn clear_range(&mut self) {
        self.range = None;
    }

    /// Get the current range, if set.
    pub fn range(&self) -> Option<(i16, i16)> {
        self.range
    }

    /// Get the rotation direction since the last update.
    ///
    /// Returns:
    /// - Positive value for clockwise rotation
    /// - Negative value for counter-clockwise rotation
    /// - 0 for no rotation
    pub fn rotation_delta(&self, previous_value: i16) -> i16 {
        // Handle wraparound
        let diff = self.value.wrapping_sub(previous_value);

        // Check for wraparound (if diff is too large, it wrapped)
        if !(-16384..=16384).contains(&diff) {
            diff.wrapping_add(i16::MIN)
        } else {
            diff
        }
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
