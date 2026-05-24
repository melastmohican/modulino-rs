//! Modulino LED Matrix driver.
//!
//! The Modulino LED Matrix module consists of a 12x8 LED matrix display (96 LEDs)
//! driven by a firmware coprocessor over I2C.

use crate::{addresses, Error, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Display mode for the LED Matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DisplayMode {
    /// Monochromatic Vertical (column-major) mode (default).
    #[default]
    MonochromaticVertical,
    /// Monochromatic Horizontal (row-major) mode.
    MonochromaticHorizontal,
    /// Grayscale mode (4-bit per pixel).
    Grayscale,
}

/// Driver for the Modulino LED Matrix module.
pub struct LedMatrix<I2C> {
    device: I2cDevice<I2C>,
    mode: DisplayMode,
    buffer: [u8; 48],
}

impl<I2C, E> LedMatrix<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new LedMatrix instance with the default address.
    pub fn new(i2c: I2C) -> Self {
        Self::new_with_address(i2c, addresses::LED_MATRIX)
    }

    /// Discover if an LedMatrix module is connected.
    ///
    /// Probes the default/match addresses and returns the first one that ACKs.
    ///
    /// > [!WARNING]
    /// > **EXPERIMENTAL**: This feature is a work-in-progress and has NOT yet been tested on physical hardware.
    pub fn discover(i2c: &mut I2C) -> Result<u8, E> {
        let addresses = [addresses::LED_MATRIX];
        for &addr in &addresses {
            if i2c.write(addr, &[]).is_ok() {
                return Ok(addr);
            }
        }
        i2c.write(addresses[0], &[])
            .map(|_| addresses[0])
            .map_err(Error::I2c)
    }

    /// Create a new LedMatrix instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Self {
        Self {
            device: I2cDevice::new(i2c, address),
            mode: DisplayMode::default(),
            buffer: [0u8; 48],
        }
    }

    /// Initialize the LED matrix and set it to default Monochromatic Vertical mode.
    pub fn init(&mut self) -> Result<(), E> {
        self.set_mode(DisplayMode::MonochromaticVertical)?;
        Ok(())
    }

    /// Set the display mode for the LED matrix.
    ///
    /// This queries the current mode on the device and performs the appropriate
    /// mode-switch payload write.
    pub fn set_mode(&mut self, mode: DisplayMode) -> Result<(), E> {
        // Query the current mode from the device by reading 4 bytes (1 pinstrap + 3 identifier)
        let mut query_buf = [0u8; 4];
        self.device.read(&mut query_buf)?;

        let device_is_grayscale = &query_buf[1..4] == b"GS4";

        // To switch the mode, the write transaction must match the size expected by
        // the *current* device mode (48 bytes if currently in grayscale, 12 bytes if monochromatic).
        if device_is_grayscale {
            let mut payload = [0u8; 48];
            match mode {
                DisplayMode::Grayscale => {
                    payload[0..3].copy_from_slice(b"GS4");
                }
                _ => {
                    payload[0..3].copy_from_slice(b"MON");
                }
            }
            self.device.write(&payload)?;
        } else {
            let mut payload = [0u8; 12];
            match mode {
                DisplayMode::Grayscale => {
                    payload[0..3].copy_from_slice(b"GS4");
                }
                _ => {
                    payload[0..3].copy_from_slice(b"MON");
                }
            }
            self.device.write(&payload)?;
        }

        self.mode = mode;
        self.clear_buffer();
        Ok(())
    }

    /// Get the current display mode.
    pub fn mode(&self) -> DisplayMode {
        self.mode
    }

    /// Clear the local frame buffer.
    fn clear_buffer(&mut self) {
        self.buffer = [0u8; 48];
    }

    /// Turn off all LEDs and update the display.
    pub fn clear(&mut self) -> Result<(), E> {
        self.clear_buffer();
        self.show()
    }

    /// Set the brightness of a specific pixel (0-255).
    ///
    /// If the display is in Grayscale mode, this maps the 0-255 brightness value
    /// to the 4-bit grayscale range (0-15). In monochromatic modes, a non-zero
    /// brightness turns the LED on.
    pub fn set_pixel(&mut self, x: u8, y: u8, brightness: u8) -> Result<(), E> {
        match self.mode {
            DisplayMode::Grayscale => {
                let val_4bit = brightness / 17; // scale 0..255 to 0..15
                self.set_grayscale_pixel(x, y, val_4bit)?;
            }
            DisplayMode::MonochromaticVertical => {
                if x >= 12 || y >= 8 {
                    return Err(Error::InvalidParameter);
                }
                let byte_idx = x as usize;
                if brightness > 0 {
                    self.buffer[byte_idx] |= 1 << y;
                } else {
                    self.buffer[byte_idx] &= !(1 << y);
                }
            }
            DisplayMode::MonochromaticHorizontal => {
                if x >= 12 || y >= 8 {
                    return Err(Error::InvalidParameter);
                }
                let pixel_idx = (y * 12 + x) as usize;
                let byte_idx = pixel_idx / 8;
                let bit_offset = 7 - (pixel_idx % 8);
                if brightness > 0 {
                    self.buffer[byte_idx] |= 1 << bit_offset;
                } else {
                    self.buffer[byte_idx] &= !(1 << bit_offset);
                }
            }
        }
        Ok(())
    }

    /// Set the 4-bit grayscale value (0-15) of a specific pixel (Grayscale mode only).
    pub fn set_grayscale_pixel(&mut self, x: u8, y: u8, value: u8) -> Result<(), E> {
        if x >= 12 || y >= 8 {
            return Err(Error::InvalidParameter);
        }
        let col_offset = (x as usize) * 4;
        let byte_idx = col_offset + (y as usize) / 2;
        let val_4bit = value & 0x0F;
        let is_upper = (y % 2) != 0;

        if is_upper {
            self.buffer[byte_idx] = (self.buffer[byte_idx] & 0x0F) | (val_4bit << 4);
        } else {
            self.buffer[byte_idx] = (self.buffer[byte_idx] & 0xF0) | val_4bit;
        }
        Ok(())
    }

    /// Set the entire active frame from a slice.
    ///
    /// Expects 12 bytes for monochromatic modes or 48 bytes for grayscale mode.
    pub fn set_frame(&mut self, frame: &[u8]) -> Result<(), E> {
        let expected_size = match self.mode {
            DisplayMode::Grayscale => 48,
            _ => 12,
        };
        if frame.len() < expected_size {
            return Err(Error::InvalidParameter);
        }
        self.buffer[0..expected_size].copy_from_slice(&frame[0..expected_size]);
        self.show()
    }

    /// Convert row-major monochromatic horizontal data to column-major vertical.
    fn convert_to_column_major(data: &mut [u8; 12]) {
        let mut col_major = [0u8; 12];
        for (col, col_val) in col_major.iter_mut().enumerate() {
            for row in 0..8 {
                let pixel_idx = row * 12 + col;
                let src_byte = pixel_idx / 8;
                let src_bit = 7 - (pixel_idx % 8);
                if ((data[src_byte] >> src_bit) & 1) != 0 {
                    *col_val |= 1 << row;
                }
            }
        }
        *data = col_major;
    }

    /// Render the current local buffer to the display.
    pub fn show(&mut self) -> Result<(), E> {
        match self.mode {
            DisplayMode::MonochromaticVertical => {
                self.device.write(&self.buffer[0..12])?;
            }
            DisplayMode::MonochromaticHorizontal => {
                let mut col_major = [0u8; 12];
                col_major.copy_from_slice(&self.buffer[0..12]);
                Self::convert_to_column_major(&mut col_major);
                self.device.write(&col_major)?;
            }
            DisplayMode::Grayscale => {
                self.device.write(&self.buffer[0..48])?;
            }
        }
        Ok(())
    }

    /// Get the current local buffer.
    pub fn buffer(&self) -> &[u8] {
        match self.mode {
            DisplayMode::Grayscale => &self.buffer[0..48],
            _ => &self.buffer[0..12],
        }
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
