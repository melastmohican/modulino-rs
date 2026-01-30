//! Modulino Distance driver.
//!
//! The Modulino Distance module uses a VL53L4CD Time-of-Flight sensor.

use crate::{addresses, I2cDevice, Result};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

// VL53L4CD Register Addresses
const VL53L4CD_SYSTEM_START: u16 = 0x0087;
const VL53L4CD_RESULT_RANGE_STATUS: u16 = 0x0089;
const VL53L4CD_RESULT_FINAL_CROSSTALK_CORRECTED_RANGE_MM_SD0: u16 = 0x0096;
const VL53L4CD_SYSTEM_INTERRUPT_CLEAR: u16 = 0x0086;
const VL53L4CD_GPIO_HV_MUX_CTRL: u16 = 0x0030;
const VL53L4CD_GPIO_TIO_HV_STATUS: u16 = 0x0031;
const VL53L4CD_RANGE_CONFIG_A: u16 = 0x005E;
const VL53L4CD_RANGE_CONFIG_B: u16 = 0x0061;
const VL53L4CD_INTERMEASUREMENT_MS: u16 = 0x006C;
const VL53L4CD_FIRMWARE_SYSTEM_STATUS: u16 = 0x00E5;
const VL53L4CD_VHV_CONFIG_TIMEOUT_MACROP_LOOP_BOUND: u16 = 0x0008;

// Tuning blob from ST Ultra Low Power Driver
const VL53L4CD_DEFAULT_CONFIGURATION: [u8; 91] = [
    0x00, 0x00, 0x00, 0x11, 0x02, 0x00, 0x02, 0x08, 0x00, 0x08, 0x10, 0x01, 0x01, 0x00, 0x00, 0x00,
    0x00, 0xff, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x0b, 0x00, 0x00, 0x02, 0x14, 0x21,
    0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0xc8, 0x00, 0x00, 0x38, 0xff, 0x01, 0x00, 0x08, 0x00,
    0x00, 0x00, 0x01, 0x07, 0x00, 0x02, 0x05, 0x00, 0xb4, 0x00, 0xbb, 0x08, 0x38, 0x00, 0x00, 0x00,
    0x00, 0x0f, 0x89, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x07, 0x05, 0x06, 0x06, 0x00,
    0x00, 0x02, 0xc7, 0xff, 0x9B, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
];

/// Driver for the Modulino Distance module.
pub struct Distance<I2C> {
    device: I2cDevice<I2C>,
}

impl<I2C, E> Distance<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Distance instance.
    pub fn new(i2c: I2C) -> Self {
        Self::new_with_address(i2c, addresses::DISTANCE)
    }

    /// Create a new Distance instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Self {
        Self {
            device: I2cDevice::new(i2c, address),
        }
    }

    /// Initialize the sensor.
    /// This performs the firmware loading and tuning required by the VL53L4CD.
    pub fn init<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), E> {
        // 1. Wait for boot
        let mut attempts = 0;
        loop {
            let status = self.device.read_reg16_u8(VL53L4CD_FIRMWARE_SYSTEM_STATUS)?;
            if status == 0x03 {
                break;
            }
            attempts += 1;
            if attempts > 1000 {
                break;
            }
            delay.delay_ms(1);
        }

        // 2. Load default configuration
        for (i, &byte) in VL53L4CD_DEFAULT_CONFIGURATION.iter().enumerate() {
            self.device.write_reg16_u8(0x2D + i as u16, byte)?;
        }

        // 3. Start VHV
        self.device.write_reg16_u8(VL53L4CD_SYSTEM_START, 0x40)?;

        // 4. Wait for data ready
        attempts = 0;
        loop {
            if self.data_ready()? {
                break;
            }
            attempts += 1;
            if attempts > 1000 {
                break;
            }
            delay.delay_ms(1);
        }

        self.clear_interrupt()?;
        self.stop_ranging()?;

        // 5. Apply specific settings
        self.device
            .write_reg16_u8(VL53L4CD_VHV_CONFIG_TIMEOUT_MACROP_LOOP_BOUND, 0x09)?;
        self.device.write_reg16_u8(0x000B, 0x00)?;
        self.device.write_reg16_u16(0x0024, 0x0500)?;
        self.device.write_reg16_u8(0x0081, 0x8A)?; // 0b1000_1010
        self.device.write_reg16_u8(0x004B, 0x03)?;

        // Set defaults
        self.set_timing_budget(20)?;
        // Use 50ms inter-measurement instead of 0 to ensure stability
        self.set_inter_measurement(50)?;

        Ok(())
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Set the timing budget in milliseconds.
    pub fn set_timing_budget(&mut self, budget_ms: u16) -> Result<(), E> {
        let (range_config_a, range_config_b) = match budget_ms {
            10 => (0x0001, 0x0001),
            15 => (0x0002, 0x0002),
            20 => (0x0005, 0x0005),
            33 => (0x000B, 0x000B),
            50 => (0x0013, 0x0013),
            100 => (0x0029, 0x0029),
            200 => (0x0055, 0x0055),
            500 => (0x00D6, 0x00D6),
            _ => (0x0005, 0x0005),
        };
        self.device
            .write_reg16_u16(VL53L4CD_RANGE_CONFIG_A, range_config_a)?;
        self.device
            .write_reg16_u16(VL53L4CD_RANGE_CONFIG_B, range_config_b)?;
        Ok(())
    }

    /// Set the inter-measurement period in milliseconds.
    pub fn set_inter_measurement(&mut self, period_ms: u32) -> Result<(), E> {
        // Simplified calculation for demo
        let osc_freq = 64000u32;
        if period_ms == 0 {
            self.device
                .write_reg16_u32(VL53L4CD_INTERMEASUREMENT_MS, 0)?;
        } else {
            let clock_pll = (period_ms as f32 * osc_freq as f32 / 1000.0) as u32;
            self.device
                .write_reg16_u32(VL53L4CD_INTERMEASUREMENT_MS, clock_pll)?;
        }
        Ok(())
    }

    /// Start continuous ranging.
    pub fn start_ranging(&mut self) -> Result<(), E> {
        self.device.write_reg16_u8(VL53L4CD_SYSTEM_START, 0x40)?;
        Ok(())
    }

    /// Stop ranging.
    pub fn stop_ranging(&mut self) -> Result<(), E> {
        self.device.write_reg16_u8(VL53L4CD_SYSTEM_START, 0x00)?;
        Ok(())
    }

    /// Check if new data is ready.
    pub fn data_ready(&mut self) -> Result<bool, E> {
        let polarity = (self.device.read_reg16_u8(VL53L4CD_GPIO_HV_MUX_CTRL)? & 0x10) >> 4;
        let status = self.device.read_reg16_u8(VL53L4CD_GPIO_TIO_HV_STATUS)? & 0x01;
        Ok(status != polarity)
    }

    /// Clear the interrupt flag.
    pub fn clear_interrupt(&mut self) -> Result<(), E> {
        self.device
            .write_reg16_u8(VL53L4CD_SYSTEM_INTERRUPT_CLEAR, 0x01)?;
        Ok(())
    }

    /// Read the distance measurement.
    pub fn read_distance(&mut self) -> Result<Option<u16>, E> {
        let status = self.device.read_reg16_u8(VL53L4CD_RESULT_RANGE_STATUS)?;
        let _range_status = status & 0x1F;
        let distance = self
            .device
            .read_reg16_u16(VL53L4CD_RESULT_FINAL_CROSSTALK_CORRECTED_RANGE_MM_SD0)?;
        self.clear_interrupt()?;

        // Return measurement regardless of status for debug visibility
        Ok(Some(distance))
    }

    /// Publicly expose range status for debugging.
    pub fn read_range_status(&mut self) -> Result<u8, E> {
        let status = self.device.read_reg16_u8(VL53L4CD_RESULT_RANGE_STATUS)?;
        Ok(status & 0x1F)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
