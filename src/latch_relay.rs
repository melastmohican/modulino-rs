//! Modulino Latch Relay driver.
//!
//! The Modulino Latch Relay module is a latching relay that maintains its state
//! even when power is removed.

use embedded_hal::i2c::I2c;
use crate::{addresses, Result};

/// Driver for the Modulino Latch Relay module.
///
/// A latching relay maintains its state (on/off) even when power is removed.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::LatchRelay;
///
/// let mut relay = LatchRelay::new(i2c)?;
///
/// // Turn on the relay
/// relay.on()?;
///
/// // Check state
/// if relay.is_on()? == Some(true) {
///     println!("Relay is ON");
/// }
///
/// // Turn off the relay
/// relay.off()?;
/// ```
pub struct LatchRelay<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> LatchRelay<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new LatchRelay instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::LATCH_RELAY)
    }

    /// Create a new LatchRelay instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let relay = Self { i2c, address };
        Ok(relay)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Turn the relay on.
    pub fn on(&mut self) -> Result<(), E> {
        let data = [1u8, 0, 0];
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Turn the relay off.
    pub fn off(&mut self) -> Result<(), E> {
        let data = [0u8, 0, 0];
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Set the relay state.
    pub fn set(&mut self, on: bool) -> Result<(), E> {
        if on {
            self.on()
        } else {
            self.off()
        }
    }

    /// Toggle the relay state.
    pub fn toggle(&mut self) -> Result<(), E> {
        match self.is_on()? {
            Some(true) => self.off(),
            _ => self.on(),
        }
    }

    /// Check if the relay is currently on.
    ///
    /// Returns:
    /// - `Some(true)` if the relay is on
    /// - `Some(false)` if the relay is off
    /// - `None` if the state is unknown (e.g., after power cycle before first command)
    pub fn is_on(&mut self) -> Result<Option<bool>, E> {
        let mut buf = [0u8; 4]; // 1 pinstrap + 3 status
        self.i2c.read(self.address, &mut buf)?;
        
        // Skip first byte (pinstrap address)
        let status0 = buf[1];
        let status1 = buf[2];
        
        // If both are 0, state is unknown (maintained from before power off)
        if status0 == 0 && status1 == 0 {
            Ok(None)
        } else if status0 == 1 {
            Ok(Some(false)) // Relay is off
        } else {
            Ok(Some(true)) // Relay is on
        }
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
