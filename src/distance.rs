//! Modulino Distance driver.
//!
//! The Modulino Distance module uses a VL53L4CD Time-of-Flight sensor
//! to measure distance to objects.
//!
//! Note: This driver provides a simplified interface. For full VL53L4CD
//! functionality, consider using a dedicated VL53L4CD driver crate.

use embedded_hal::i2c::I2c;
use crate::{addresses, Error, Result};

// VL53L4CD register addresses
const VL53L4CD_SYSTEM_START: u16 = 0x0087;
const VL53L4CD_RESULT_RANGE_STATUS: u16 = 0x0089;
const VL53L4CD_RESULT_FINAL_CROSSTALK_CORRECTED_RANGE_MM_SD0: u16 = 0x0096;
const VL53L4CD_SYSTEM_INTERRUPT_CLEAR: u16 = 0x0086;
const VL53L4CD_GPIO_HV_MUX_CTRL: u16 = 0x0030;
const VL53L4CD_GPIO_TIO_HV_STATUS: u16 = 0x0031;
const VL53L4CD_RANGE_CONFIG_A: u16 = 0x005E;
const VL53L4CD_RANGE_CONFIG_B: u16 = 0x0061;
const VL53L4CD_INTERMEASUREMENT_MS: u16 = 0x006C;

/// Driver for the Modulino Distance module (VL53L4CD ToF sensor).
///
/// # Example
///
/// ```rust,ignore
/// use modulino::Distance;
///
/// let mut distance = Distance::new(i2c)?;
///
/// // Start ranging
/// distance.start_ranging()?;
///
/// // Read distance
/// if let Some(mm) = distance.read_distance()? {
///     println!("Distance: {} mm", mm);
/// }
/// ```
pub struct Distance<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Distance<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Distance instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::DISTANCE)
    }

    /// Create a new Distance instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut distance = Self { i2c, address };
        
        // Initialize sensor with default settings
        distance.init()?;
        
        Ok(distance)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Write a byte to a register.
    fn write_register(&mut self, reg: u16, value: u8) -> Result<(), E> {
        let reg_bytes = reg.to_be_bytes();
        let data = [reg_bytes[0], reg_bytes[1], value];
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Write a 16-bit value to a register.
    fn write_register_16(&mut self, reg: u16, value: u16) -> Result<(), E> {
        let reg_bytes = reg.to_be_bytes();
        let val_bytes = value.to_be_bytes();
        let data = [reg_bytes[0], reg_bytes[1], val_bytes[0], val_bytes[1]];
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Write a 32-bit value to a register.
    fn write_register_32(&mut self, reg: u16, value: u32) -> Result<(), E> {
        let reg_bytes = reg.to_be_bytes();
        let val_bytes = value.to_be_bytes();
        let data = [reg_bytes[0], reg_bytes[1], val_bytes[0], val_bytes[1], val_bytes[2], val_bytes[3]];
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Read a byte from a register.
    fn read_register(&mut self, reg: u16) -> Result<u8, E> {
        let reg_bytes = reg.to_be_bytes();
        self.i2c.write(self.address, &reg_bytes)?;
        
        let mut buf = [0u8; 1];
        self.i2c.read(self.address, &mut buf)?;
        Ok(buf[0])
    }

    /// Read a 16-bit value from a register.
    fn read_register_16(&mut self, reg: u16) -> Result<u16, E> {
        let reg_bytes = reg.to_be_bytes();
        self.i2c.write(self.address, &reg_bytes)?;
        
        let mut buf = [0u8; 2];
        self.i2c.read(self.address, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Initialize the sensor with default settings.
    fn init(&mut self) -> Result<(), E> {
        // Set timing budget to 20ms
        self.set_timing_budget(20)?;
        
        // Set inter-measurement period to 0 (continuous)
        self.set_inter_measurement(0)?;
        
        Ok(())
    }

    /// Set the timing budget in milliseconds.
    ///
    /// Valid values: 10, 15, 20, 33, 50, 100, 200, 500
    pub fn set_timing_budget(&mut self, budget_ms: u16) -> Result<(), E> {
        let (osc_freq, macro_period_us) = (64000u32, 2304u32);
        
        let timing_budget_us = budget_ms as u32 * 1000;
        let macro_period = macro_period_us;
        
        // Simplified calculation - using preset values for common budgets
        let (range_config_a, range_config_b) = match budget_ms {
            10 => (0x0001, 0x0001),
            15 => (0x0002, 0x0002),
            20 => (0x0005, 0x0005),
            33 => (0x000B, 0x000B),
            50 => (0x0013, 0x0013),
            100 => (0x0029, 0x0029),
            200 => (0x0055, 0x0055),
            500 => (0x00D6, 0x00D6),
            _ => (0x0005, 0x0005), // Default to 20ms
        };
        
        self.write_register_16(VL53L4CD_RANGE_CONFIG_A, range_config_a)?;
        self.write_register_16(VL53L4CD_RANGE_CONFIG_B, range_config_b)?;
        
        Ok(())
    }

    /// Set the inter-measurement period in milliseconds.
    ///
    /// Set to 0 for continuous ranging.
    pub fn set_inter_measurement(&mut self, period_ms: u32) -> Result<(), E> {
        let osc_freq = 64000u32;
        let clock_pll = (period_ms as f32 * osc_freq as f32 / 1000.0) as u32;
        self.write_register_32(VL53L4CD_INTERMEASUREMENT_MS, clock_pll)?;
        Ok(())
    }

    /// Start continuous ranging.
    pub fn start_ranging(&mut self) -> Result<(), E> {
        self.write_register(VL53L4CD_SYSTEM_START, 0x40)?;
        Ok(())
    }

    /// Stop ranging.
    pub fn stop_ranging(&mut self) -> Result<(), E> {
        self.write_register(VL53L4CD_SYSTEM_START, 0x00)?;
        Ok(())
    }

    /// Check if new data is ready.
    pub fn data_ready(&mut self) -> Result<bool, E> {
        let polarity = (self.read_register(VL53L4CD_GPIO_HV_MUX_CTRL)? & 0x10) >> 4;
        let status = self.read_register(VL53L4CD_GPIO_TIO_HV_STATUS)? & 0x01;
        Ok(status != polarity)
    }

    /// Clear the interrupt flag.
    pub fn clear_interrupt(&mut self) -> Result<(), E> {
        self.write_register(VL53L4CD_SYSTEM_INTERRUPT_CLEAR, 0x01)?;
        Ok(())
    }

    /// Read the distance measurement.
    ///
    /// Returns `None` if the measurement is invalid.
    pub fn read_distance(&mut self) -> Result<Option<u16>, E> {
        // Check range status
        let status = self.read_register(VL53L4CD_RESULT_RANGE_STATUS)?;
        let range_status = status & 0x1F;
        
        // Read distance
        let distance = self.read_register_16(VL53L4CD_RESULT_FINAL_CROSSTALK_CORRECTED_RANGE_MM_SD0)?;
        
        // Clear interrupt for next measurement
        self.clear_interrupt()?;
        
        // Check if measurement is valid (status 0 or 4 are typically valid)
        if range_status == 0 || range_status == 4 {
            Ok(Some(distance))
        } else {
            Ok(None)
        }
    }

    /// Read distance, waiting for data to be ready.
    ///
    /// This is a blocking call that waits for new measurement data.
    /// Returns the distance in millimeters.
    pub fn read_distance_blocking(&mut self) -> Result<u16, E> {
        // Wait for data ready (simple polling)
        loop {
            if self.data_ready()? {
                break;
            }
        }
        
        match self.read_distance()? {
            Some(d) if d > 0 => Ok(d),
            _ => self.read_distance_blocking(), // Retry on invalid reading
        }
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
