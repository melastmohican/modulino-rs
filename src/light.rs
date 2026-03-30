//! Modulino Light sensor driver.
//!
//! The Modulino Light module uses an LTR-381RGB sensor for
//! measuring RGB color, infrared, and ambient light levels.
//!
//! > [!WARNING]
//! > **EXPERIMENTAL**: This driver is a work-in-progress and has only been verified via
//! > unit tests using I2C mocks. It has NOT yet been tested on physical Modulino hardware.
//!
//! Note: This is an internal implementation because no stable `no_std` Rust crate
//! currently exists for the LTR-381RGB sensor.

use crate::{addresses, Error, Result};
use embedded_hal::i2c::I2c;

/// Measurement result from the Light sensor.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LightMeasurement {
    /// Red channel value
    pub red: u32,
    /// Green channel value
    pub green: u32,
    /// Blue channel value
    pub blue: u32,
    /// Infrared channel value
    pub ir: u32,
}

/// Available gain settings for the LTR-381RGB sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Gain {
    /// 1x Gain
    Gain1x = 0x00,
    /// 3x Gain
    Gain3x = 0x01,
    /// 6x Gain
    Gain6x = 0x02,
    /// 9x Gain
    Gain9x = 0x03,
    /// 18x Gain
    Gain18x = 0x04,
}

/// Available ADC resolutions for the LTR-381RGB sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Resolution {
    /// 20-bit resolution (400ms integration time)
    Res20Bit = 0x00,
    /// 19-bit resolution (200ms integration time)
    Res19Bit = 0x01,
    /// 18-bit resolution (100ms integration time)
    Res18Bit = 0x02,
    /// 17-bit resolution (50ms integration time)
    Res17Bit = 0x03,
    /// 16-bit resolution (25ms integration time)
    Res16Bit = 0x04,
}

/// Available measurement rates for the LTR-381RGB sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MeasurementRate {
    /// 25ms
    Rate25ms = 0x00,
    /// 50ms
    Rate50ms = 0x01,
    /// 100ms
    Rate100ms = 0x02,
    /// 200ms
    Rate200ms = 0x03,
    /// 400ms
    Rate400ms = 0x04,
    /// 500ms
    Rate500ms = 0x05,
    /// 1000ms
    Rate1000ms = 0x06,
    /// 2000ms
    Rate2000ms = 0x07,
}

/// Driver for the Modulino Light module (LTR-381RGB sensor).
pub struct Light<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Light<I2C>
where
    I2C: I2c<Error = E>,
{
    const REG_MAIN_CTRL: u8 = 0x00;
    const REG_MEAS_RATE: u8 = 0x04;
    const REG_GAIN: u8 = 0x05;
    const REG_PART_ID: u8 = 0x06;
    const REG_MAIN_STATUS: u8 = 0x07;
    const REG_DATA_IR: u8 = 0x0A;
    const REG_DATA_GREEN: u8 = 0x0D;
    const REG_DATA_RED: u8 = 0x10;
    const REG_DATA_BLUE: u8 = 0x13;

    /// Create a new Light instance.
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: addresses::LIGHT,
        }
    }

    /// Initialize the sensor.
    pub fn init(&mut self) -> Result<(), E> {
        // Verify Part ID
        let mut part_id = [0u8; 1];
        self.i2c
            .write_read(self.address, &[Self::REG_PART_ID], &mut part_id)
            .map_err(Error::I2c)?;
        if (part_id[0] & 0xF0) != 0xC0 {
            return Err(Error::DeviceNotFound);
        }

        // Clear status
        let mut status = [0u8; 1];
        self.i2c
            .write_read(self.address, &[Self::REG_MAIN_STATUS], &mut status)
            .map_err(Error::I2c)?;

        // Default config: 18x gain, 16-bit res, 25ms rate
        self.set_gain(Gain::Gain18x)?;
        self.set_config(Resolution::Res16Bit, MeasurementRate::Rate25ms)?;

        // Enable RGB mode
        self.enable(true)?;

        Ok(())
    }

    /// Enable or disable the sensor measurements.
    pub fn enable(&mut self, enabled: bool) -> Result<(), E> {
        let val = if enabled { 0x06 } else { 0x00 }; // 0x06 = RGB + ALS
        self.i2c
            .write(self.address, &[Self::REG_MAIN_CTRL, val])
            .map_err(Error::I2c)
    }

    /// Set sensor gain.
    pub fn set_gain(&mut self, gain: Gain) -> Result<(), E> {
        self.i2c
            .write(self.address, &[Self::REG_GAIN, gain as u8])
            .map_err(Error::I2c)
    }

    /// Set ADC resolution and measurement rate.
    pub fn set_config(&mut self, res: Resolution, rate: MeasurementRate) -> Result<(), E> {
        let val = ((res as u8) << 4) | (rate as u8);
        self.i2c
            .write(self.address, &[Self::REG_MEAS_RATE, val])
            .map_err(Error::I2c)
    }

    /// Read all color channels.
    pub fn read(&mut self) -> Result<LightMeasurement, E> {
        Ok(LightMeasurement {
            ir: self.read_channel(Self::REG_DATA_IR)?,
            green: self.read_channel(Self::REG_DATA_GREEN)?,
            red: self.read_channel(Self::REG_DATA_RED)?,
            blue: self.read_channel(Self::REG_DATA_BLUE)?,
        })
    }

    fn read_channel(&mut self, reg: u8) -> Result<u32, E> {
        let mut buf = [0u8; 3];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(Error::I2c)?;

        // Combine 3 bytes (LSB first)
        let val = (buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16);
        Ok(val)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
