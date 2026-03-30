//! Modulino Pressure driver.
//!
//! The Modulino Pressure module uses an LPS22HB sensor for barometric
//! pressure and temperature measurements.
//!
//! > [!WARNING]
//! > **EXPERIMENTAL**: This driver is a work-in-progress and has only been verified via
//! > unit tests using I2C mocks. It has NOT yet been tested on physical Modulino hardware.
//!
//! Note: This is an internal implementation because the existing `lps22hb` crate (v0.1.0)
//! is built for `embedded-hal` 0.2 and is incompatible with the `embedded-hal` 1.0
//! traits used by this crate.
//!
//! If the `lps22hb` crate is updated to EH 1.0, this can be moved to an external wrapper.

use crate::{addresses, Error, Result};
use embedded_hal::i2c::I2c;

/// Driver for the Modulino Pressure module (LPS22HB sensor).
pub struct Pressure<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Pressure<I2C>
where
    I2C: I2c<Error = E>,
{
    const REG_WHO_AM_I: u8 = 0x0F;
    const REG_CTRL_REG1: u8 = 0x10;
    const REG_CTRL_REG2: u8 = 0x11;
    const _REG_STATUS_REG: u8 = 0x27;
    const REG_OUT_P_XL: u8 = 0x28;
    const REG_OUT_T_L: u8 = 0x2B;

    const WHO_AM_I_VALUE: u8 = 0xB1;

    /// Create a new Pressure instance.
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: addresses::PRESSURE,
        }
    }

    /// Initialize the sensor.
    pub fn init(&mut self) -> Result<(), E> {
        let mut id = [0u8; 1];
        self.i2c
            .write_read(self.address, &[Self::REG_WHO_AM_I], &mut id)
            .map_err(Error::I2c)?;
        if id[0] != Self::WHO_AM_I_VALUE {
            return Err(Error::DeviceNotFound);
        }

        // Set ODR to 10Hz, enable block data update
        self.i2c
            .write(self.address, &[Self::REG_CTRL_REG1, 0x22])
            .map_err(Error::I2c)?;

        Ok(())
    }

    /// Read atmospheric pressure in hPa.
    pub fn pressure(&mut self) -> Result<f32, E> {
        let mut buf = [0u8; 3];
        self.i2c
            .write_read(self.address, &[Self::REG_OUT_P_XL], &mut buf)
            .map_err(Error::I2c)?;

        let raw = (buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16);
        // Sensitivity is 4096 LSB/hPa
        Ok(raw as f32 / 4096.0)
    }

    /// Read ambient temperature in degrees Celsius.
    pub fn temperature(&mut self) -> Result<f32, E> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[Self::REG_OUT_T_L], &mut buf)
            .map_err(Error::I2c)?;

        let raw = (buf[0] as u16 | ((buf[1] as u16) << 8)) as i16;
        // Sensitivity is 100 LSB/°C
        Ok(raw as f32 / 100.0)
    }

    /// Perform a software reset.
    pub fn reset(&mut self) -> Result<(), E> {
        self.i2c
            .write(self.address, &[Self::REG_CTRL_REG2, 0x04])
            .map_err(Error::I2c)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
