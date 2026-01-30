//! Modulino Vibro driver.
//!
//! The Modulino Vibro module contains a vibration motor.

use crate::{addresses, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Predefined power levels for the vibration motor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PowerLevel {
    /// Motor stopped
    Stop = 0,
    /// Gentle vibration
    Gentle = 25,
    /// Moderate vibration
    Moderate = 35,
    /// Medium vibration
    #[default]
    Medium = 45,
    /// Intense vibration
    Intense = 55,
    /// Powerful vibration
    Powerful = 65,
    /// Maximum vibration
    Maximum = 75,
}

impl PowerLevel {
    /// Get the numeric power value.
    pub const fn value(&self) -> u8 {
        *self as u8
    }
}

impl From<PowerLevel> for u8 {
    fn from(level: PowerLevel) -> Self {
        level.value()
    }
}

/// Driver for the Modulino Vibro module.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::{Vibro, PowerLevel};
///
/// let mut vibro = Vibro::new(i2c)?;
///
/// // Vibrate at medium power for 500ms
/// vibro.on(500, PowerLevel::Medium)?;
///
/// // Vibrate with custom power level
/// vibro.on_with_power(1000, 50)?;
///
/// // Stop vibration
/// vibro.off()?;
/// ```
pub struct Vibro<I2C> {
    device: I2cDevice<I2C>,
    frequency: u32,
}

impl<I2C, E> Vibro<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Default vibration frequency in Hz.
    pub const DEFAULT_FREQUENCY: u32 = 1000;

    /// Create a new Vibro instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::VIBRO)
    }

    /// Create a new Vibro instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut vibro = Self {
            device: I2cDevice::new(i2c, address),
            frequency: Self::DEFAULT_FREQUENCY,
        };

        // Ensure motor is off on init
        vibro.off()?;

        Ok(vibro)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Get the current frequency setting.
    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    /// Set the vibration frequency.
    pub fn set_frequency(&mut self, frequency: u32) {
        self.frequency = frequency;
    }

    /// Turn on the vibration motor.
    ///
    /// # Arguments
    ///
    /// * `duration_ms` - Duration in milliseconds (0xFFFF for indefinite)
    /// * `power` - Power level
    pub fn on(&mut self, duration_ms: u16, power: PowerLevel) -> Result<(), E> {
        self.on_with_power(duration_ms, power.value())
    }

    /// Turn on the vibration motor with a custom power level.
    ///
    /// # Arguments
    ///
    /// * `duration_ms` - Duration in milliseconds (0xFFFF for indefinite)
    /// * `power` - Power level (0-100)
    pub fn on_with_power(&mut self, duration_ms: u16, power: u8) -> Result<(), E> {
        let freq_bytes = self.frequency.to_le_bytes();
        let duration_bytes = (duration_ms as u32).to_le_bytes();
        let power_bytes = (power as u32).to_le_bytes();

        let data = [
            freq_bytes[0],
            freq_bytes[1],
            freq_bytes[2],
            freq_bytes[3],
            duration_bytes[0],
            duration_bytes[1],
            duration_bytes[2],
            duration_bytes[3],
            power_bytes[0],
            power_bytes[1],
            power_bytes[2],
            power_bytes[3],
        ];

        self.device.write(&data)?;
        Ok(())
    }

    /// Turn on the vibration motor indefinitely.
    pub fn on_continuous(&mut self, power: PowerLevel) -> Result<(), E> {
        self.on(0xFFFF, power)
    }

    /// Turn off the vibration motor.
    pub fn off(&mut self) -> Result<(), E> {
        let data = [0u8; 12];
        self.device.write(&data)?;
        Ok(())
    }

    /// Alias for `off()`.
    pub fn stop(&mut self) -> Result<(), E> {
        self.off()
    }

    /// Vibrate in a pattern (pulse).
    ///
    /// # Arguments
    ///
    /// * `on_ms` - Vibration duration in milliseconds
    /// * `power` - Power level
    pub fn pulse(&mut self, on_ms: u16, power: PowerLevel) -> Result<(), E> {
        self.on(on_ms, power)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
