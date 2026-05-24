//! Modulino Hub driver.
//!
//! > [!WARNING]
//! > **EXPERIMENTAL**: This driver is a work-in-progress and is based on the TCA9548A I2C
//! > multiplexer (such as the SparkFun Qwiic Mux Breakout). It has NOT yet been tested on physical
//! > Modulino Hub hardware, as the official Modulino Hub was never officially released.

use crate::Result;
use embedded_hal::i2c::I2c;

/// Default address for the TCA9548A multiplexer (0x70).
pub const DEFAULT_ADDRESS: u8 = 0x70;

/// Driver for the Modulino Hub (TCA9548A I2C multiplexer).
pub struct Hub<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Hub<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Hub instance with the default address (0x70).
    pub fn new(i2c: I2C) -> Self {
        Self::new_with_address(i2c, DEFAULT_ADDRESS)
    }

    /// Create a new Hub instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Select an active port channel (0 to 7).
    ///
    /// This enables the specified channel and disables all others.
    pub fn select(&mut self, port: u8) -> Result<(), E> {
        if port >= 8 {
            return Err(crate::Error::InvalidParameter);
        }
        let control_byte = 1 << port;
        self.i2c
            .write(self.address, &[control_byte])
            .map_err(crate::Error::I2c)
    }

    /// Clear/deselect all port channels.
    ///
    /// This disables all I2C channels on the multiplexer.
    pub fn clear(&mut self) -> Result<(), E> {
        self.i2c
            .write(self.address, &[0x00])
            .map_err(crate::Error::I2c)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}

/// A helper representing a specific port of the Hub.
pub struct HubPort<'a, I2C> {
    hub: &'a mut Hub<I2C>,
    port: u8,
}

impl<'a, I2C, E> HubPort<'a, I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new HubPort.
    pub fn new(hub: &'a mut Hub<I2C>, port: u8) -> Self {
        Self { hub, port }
    }

    /// Enable this port channel on the multiplexer.
    pub fn select(&mut self) -> Result<(), E> {
        self.hub.select(self.port)
    }

    /// Disable all port channels on the multiplexer.
    pub fn clear(&mut self) -> Result<(), E> {
        self.hub.clear()
    }
}
