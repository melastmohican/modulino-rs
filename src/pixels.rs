//! Modulino Pixels driver.
//!
//! The Modulino Pixels module has 8 RGB LEDs (APA102-compatible).

use crate::{addresses, Color, Error, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Number of LEDs on the Modulino Pixels.
pub const NUM_LEDS: usize = 8;

/// Driver for the Modulino Pixels module.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::{Pixels, Color};
///
/// let mut pixels = Pixels::new(i2c)?;
///
/// // Set individual LED
/// pixels.set_color(0, Color::RED, 50)?;
///
/// // Set all LEDs to the same color
/// pixels.set_all_color(Color::BLUE, 25)?;
///
/// // Apply changes
/// pixels.show()?;
///
/// // Clear all LEDs
/// pixels.clear_all()?;
/// pixels.show()?;
/// ```
pub struct Pixels<I2C> {
    device: I2cDevice<I2C>,
    data: [u8; NUM_LEDS * 4],
}

impl<I2C, E> Pixels<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Pixels instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::PIXELS)
    }

    /// Create a new Pixels instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut pixels = Self {
            device: I2cDevice::new(i2c, address),
            data: [0xE0; NUM_LEDS * 4], // Initialize with brightness bits set, LEDs off
        };

        // Clear all LEDs on init
        pixels.clear_all();

        Ok(pixels)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Map brightness from 0-100 to 0-31 (5-bit brightness for APA102).
    fn map_brightness(brightness: u8) -> u8 {
        let clamped = if brightness > 100 { 100 } else { brightness };
        ((clamped as u16 * 31) / 100) as u8
    }

    /// Set the color of a specific LED.
    ///
    /// # Arguments
    ///
    /// * `index` - LED index (0-7)
    /// * `color` - The color to set
    /// * `brightness` - Brightness level (0-100)
    pub fn set_color(
        &mut self,
        index: usize,
        color: Color,
        brightness: u8,
    ) -> Result<&mut Self, E> {
        if index >= NUM_LEDS {
            return Err(Error::OutOfRange);
        }

        let byte_index = index * 4;
        let mapped_brightness = Self::map_brightness(brightness);
        let color_data = color.to_apa102_data() | (mapped_brightness as u32) | 0xE0;

        let bytes = color_data.to_le_bytes();
        self.data[byte_index..byte_index + 4].copy_from_slice(&bytes);

        Ok(self)
    }

    /// Set the color of a specific LED using RGB values.
    ///
    /// # Arguments
    ///
    /// * `index` - LED index (0-7)
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `brightness` - Brightness level (0-100)
    pub fn set_rgb(
        &mut self,
        index: usize,
        r: u8,
        g: u8,
        b: u8,
        brightness: u8,
    ) -> Result<&mut Self, E> {
        self.set_color(index, Color::new(r, g, b), brightness)
    }

    /// Set the brightness of a specific LED without changing its color.
    ///
    /// # Arguments
    ///
    /// * `index` - LED index (0-7)
    /// * `brightness` - Brightness level (0-100)
    pub fn set_brightness(&mut self, index: usize, brightness: u8) -> Result<&mut Self, E> {
        if index >= NUM_LEDS {
            return Err(Error::OutOfRange);
        }

        let byte_index = index * 4;
        let mapped_brightness = Self::map_brightness(brightness);
        self.data[byte_index] = mapped_brightness | 0xE0;

        Ok(self)
    }

    /// Set the color of all LEDs.
    pub fn set_all_color(&mut self, color: Color, brightness: u8) -> &mut Self {
        for i in 0..NUM_LEDS {
            let _ = self.set_color(i, color, brightness);
        }
        self
    }

    /// Set all LEDs to the same RGB color.
    pub fn set_all_rgb(&mut self, r: u8, g: u8, b: u8, brightness: u8) -> &mut Self {
        self.set_all_color(Color::new(r, g, b), brightness)
    }

    /// Set the color of a range of LEDs.
    ///
    /// # Arguments
    ///
    /// * `from` - Starting index (inclusive)
    /// * `to` - Ending index (inclusive)
    /// * `color` - The color to set
    /// * `brightness` - Brightness level (0-100)
    pub fn set_range_color(
        &mut self,
        from: usize,
        to: usize,
        color: Color,
        brightness: u8,
    ) -> &mut Self {
        let end = if to >= NUM_LEDS { NUM_LEDS - 1 } else { to };
        for i in from..=end {
            let _ = self.set_color(i, color, brightness);
        }
        self
    }

    /// Set the brightness of all LEDs without changing their colors.
    pub fn set_all_brightness(&mut self, brightness: u8) -> &mut Self {
        for i in 0..NUM_LEDS {
            let _ = self.set_brightness(i, brightness);
        }
        self
    }

    /// Clear (turn off) a specific LED.
    pub fn clear(&mut self, index: usize) -> Result<&mut Self, E> {
        self.set_color(index, Color::BLACK, 0)
    }

    /// Clear a range of LEDs.
    pub fn clear_range(&mut self, from: usize, to: usize) -> &mut Self {
        let end = if to >= NUM_LEDS { NUM_LEDS - 1 } else { to };
        for i in from..=end {
            let _ = self.clear(i);
        }
        self
    }

    /// Clear all LEDs.
    pub fn clear_all(&mut self) -> &mut Self {
        for i in 0..NUM_LEDS {
            // Brightness 0 maps to 0 | 0xE0 = 0xE0.
            // Color Black is 0.
            // We want [0xE0, 0, 0, 0] in memory (LE).
            let _ = self.set_color(i, Color::BLACK, 0);
        }
        self
    }

    /// Apply the current LED states to the hardware.
    ///
    /// This must be called after setting colors for changes to take effect.
    pub fn show(&mut self) -> Result<(), E> {
        self.device.write(&self.data)?;
        Ok(())
    }

    /// Set a color and immediately show it.
    pub fn set_color_show(&mut self, index: usize, color: Color, brightness: u8) -> Result<(), E> {
        self.set_color(index, color, brightness)?;
        self.show()
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
