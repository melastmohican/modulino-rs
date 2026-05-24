//! Modulino Opto Relay driver.
//!
//! The Modulino Opto Relay module contains an optically isolated solid state relay.
//!
//! > [!WARNING]
//! > **EXPERIMENTAL**: This driver is a work-in-progress and has only been verified via
//! > unit tests using I2C mocks. It has NOT yet been tested on physical Modulino hardware.

use crate::{addresses, Error, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Driver for the Modulino Opto Relay module.
pub struct OptoRelay<I2C> {
    device: I2cDevice<I2C>,
    is_on: bool,
}

impl<I2C, E> OptoRelay<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new OptoRelay instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::OPTO_RELAY)
    }

    /// Discover if an OptoRelay module is connected.
    ///
    /// Probes the default/match addresses and returns the first one that ACKs.
    ///
    /// > [!WARNING]
    /// > **EXPERIMENTAL**: This feature is a work-in-progress and has NOT yet been tested on physical hardware.
    pub fn discover(i2c: &mut I2C) -> Result<u8, E> {
        let addresses = [addresses::OPTO_RELAY];
        for &addr in &addresses {
            if i2c.write(addr, &[]).is_ok() {
                return Ok(addr);
            }
        }
        i2c.write(addresses[0], &[]).map(|_| addresses[0]).map_err(Error::I2c)
    }

    /// Create a new OptoRelay instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut relay = Self {
            device: I2cDevice::new(i2c, address),
            is_on: false,
        };

        // Read initial state
        relay.update()?;

        Ok(relay)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Turn on the opto relay.
    pub fn on(&mut self) -> Result<(), E> {
        let data = [1u8, 0, 0];
        self.device.write(&data)?;
        self.is_on = true;
        Ok(())
    }

    /// Turn off the opto relay.
    pub fn off(&mut self) -> Result<(), E> {
        let data = [0u8, 0, 0];
        self.device.write(&data)?;
        self.is_on = false;
        Ok(())
    }

    /// Set the opto relay state.
    pub fn set(&mut self, on: bool) -> Result<(), E> {
        if on {
            self.on()
        } else {
            self.off()
        }
    }

    /// Toggle the opto relay state.
    pub fn toggle(&mut self) -> Result<(), E> {
        self.set(!self.is_on)
    }

    /// Update the opto relay state from the device.
    ///
    /// Returns `true` if the state has changed.
    pub fn update(&mut self) -> Result<bool, E> {
        let previous = self.is_on;
        let mut buf = [0u8; 4]; // 1 pinstrap + 3 status
        self.device.read(&mut buf)?;

        // Skip first byte (pinstrap address)
        self.is_on = buf[1] != 0;

        Ok(self.is_on != previous)
    }

    /// Check if the relay is currently on.
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
