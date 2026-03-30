//! Modulino LED Matrix driver.
//!
//! The Modulino LED Matrix module uses an IS31FL3733 LED driver
//! for controlling a 6x16 LED matrix.
//!
//! > [!WARNING]
//! > **EXPERIMENTAL**: This driver is a work-in-progress and has only been verified via
//! > unit tests using I2C mocks. It has NOT yet been tested on physical Modulino hardware.
//!
//! Note: This is an internal implementation because the existing `is31fl3733` crate (v0.5.0)
//! has a visibility issue where the required `Blocking` and `Async` mode types are not
//! exported, making the driver struct impossible to name in external code.
//!
//! If the `is31fl3733` crate fixes its exports, this can be moved to an external wrapper.

use crate::{addresses, Error, Result};
use embedded_hal::i2c::I2c;

/// Driver for the Modulino LED Matrix module (IS31FL3733 driver).
///
/// This is a custom `no_std` driver implemented specifically for Modulino's
/// 6x16 / 12x8 matrix configuration.
pub struct LedMatrix<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> LedMatrix<I2C>
where
    I2C: I2c<Error = E>,
{
    const REG_COMMAND: u8 = 0xFD;
    const REG_WRITE_LOCK: u8 = 0xFE;

    const PAGE_LED_CONTROL: u8 = 0x00;
    const PAGE_PWM_CONTROL: u8 = 0x01;
    const PAGE_FUNCTION: u8 = 0x03;

    const REG_FUNCTION_CONFIG: u8 = 0x00;
    const REG_FUNCTION_GCC: u8 = 0x01;
    const REG_FUNCTION_RESET: u8 = 0x11;

    /// Create a new LedMatrix instance.
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: addresses::LED_MATRIX,
        }
    }

    /// Initialize the LED matrix.
    pub fn init(&mut self) -> Result<(), E> {
        self.unlock()?;

        // Reset device
        self.select_page(Self::PAGE_FUNCTION)?;
        let mut _reset = [0u8; 1];
        self.i2c
            .write_read(self.address, &[Self::REG_FUNCTION_RESET], &mut _reset)
            .map_err(Error::I2c)?;

        // Enable Normal Operation (wake up)
        self.i2c
            .write(self.address, &[Self::REG_FUNCTION_CONFIG, 0x01])
            .map_err(Error::I2c)?;

        // Set Global Control Current
        self.i2c
            .write(self.address, &[Self::REG_FUNCTION_GCC, 0x40])
            .map_err(Error::I2c)?;

        // Clear all (turn off all LEDs)
        self.clear()?;

        Ok(())
    }

    /// Turn off all LEDs and set PWM values to zero.
    pub fn clear(&mut self) -> Result<(), E> {
        // Clear LED control registers (off)
        self.select_page(Self::PAGE_LED_CONTROL)?;
        let mut off = [0u8; 25];
        off[0] = 0x00; // Starting register
        for i in 1..25 {
            off[i] = 0x00;
        }
        self.i2c.write(self.address, &off).map_err(Error::I2c)?;

        // Clear PWM registers (zero brightness)
        self.select_page(Self::PAGE_PWM_CONTROL)?;
        let mut pwm = [0u8; 193];
        pwm[0] = 0x00; // Starting register
        for i in 1..193 {
            pwm[i] = 0x00;
        }
        self.i2c.write(self.address, &pwm).map_err(Error::I2c)?;

        Ok(())
    }

    /// Set the brightness of a specific LED.
    ///
    /// # Arguments
    /// * `x` - Column index (0-15)
    /// * `y` - Row index (0-5)
    /// * `brightness` - PWM value (0-255)
    pub fn set_pixel(&mut self, x: u8, y: u8, brightness: u8) -> Result<(), E> {
        if x >= 16 || y >= 6 {
            return Err(Error::InvalidParameter);
        }

        // 1. Enable the LED (Page 0)
        self.select_page(Self::PAGE_LED_CONTROL)?;
        let byte_offset = (y << 1) + (x >> 3); // 2 bytes per row
        let bit_offset = x % 8;

        let mut current_state = [0u8; 1];
        self.i2c
            .write_read(self.address, &[byte_offset], &mut current_state)
            .map_err(Error::I2c)?;

        if brightness > 0 {
            current_state[0] |= 1 << bit_offset;
        } else {
            current_state[0] &= !(1 << bit_offset);
        }
        self.i2c
            .write(self.address, &[byte_offset, current_state[0]])
            .map_err(Error::I2c)?;

        // 2. Set PWM (Page 1)
        self.select_page(Self::PAGE_PWM_CONTROL)?;
        let pwm_offset = (y << 4) + x;
        self.i2c
            .write(self.address, &[pwm_offset, brightness])
            .map_err(Error::I2c)?;

        Ok(())
    }

    /// Update the display (placeholder as some chips use a 'show' trigger,
    /// but IS31FL3733 updates immediately upon I2C write).
    pub fn show(&mut self) -> Result<(), E> {
        Ok(())
    }

    fn unlock(&mut self) -> Result<(), E> {
        self.i2c
            .write(self.address, &[Self::REG_WRITE_LOCK, 0xC5])
            .map_err(Error::I2c)
    }

    fn select_page(&mut self, page: u8) -> Result<(), E> {
        self.i2c
            .write(self.address, &[Self::REG_COMMAND, page])
            .map_err(Error::I2c)
    }

    /// Set global current control (overall brightness limit).
    pub fn set_global_current(&mut self, current: u8) -> Result<(), E> {
        self.select_page(Self::PAGE_FUNCTION)?;
        self.i2c
            .write(self.address, &[Self::REG_FUNCTION_GCC, current])
            .map_err(Error::I2c)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
