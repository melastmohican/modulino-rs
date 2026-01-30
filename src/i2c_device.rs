//! I2C Device helper.
//!
//! This module provides a helper struct `I2cDevice` that wraps an I2C bus
//! and a device address, providing common methods for register-based communication.

use embedded_hal::i2c::I2c;

/// Helper struct for I2C operations.
pub struct I2cDevice<I2C> {
    /// The I2C bus.
    pub i2c: I2C,
    /// The device address.
    pub address: u8,
}

impl<I2C, E> I2cDevice<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new I2cDevice.
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Write bytes to the device.
    pub fn write(&mut self, data: &[u8]) -> Result<(), E> {
        self.i2c.write(self.address, data)
    }

    /// Read bytes from the device.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<(), E> {
        self.i2c.read(self.address, buf)
    }

    /// Write bytes and then read bytes (Repeated Start).
    pub fn write_read(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), E> {
        self.i2c.write_read(self.address, write, read)
    }

    /// Write a byte to an 8-bit register.
    pub fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), E> {
        self.write(&[reg, value])
    }

    /// Read a byte from an 8-bit register.
    pub fn read_reg(&mut self, reg: u8) -> Result<u8, E> {
        let mut buf = [0u8; 1];
        self.write_read(&[reg], &mut buf)?;
        Ok(buf[0])
    }

    /// Read multiple bytes starting from an 8-bit register.
    pub fn read_regs(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), E> {
        self.write_read(&[reg], buf)
    }

    /// Write a byte to a 16-bit register (Big Endian address).
    pub fn write_reg16_u8(&mut self, reg: u16, value: u8) -> Result<(), E> {
        let reg_bytes = reg.to_be_bytes();
        self.write(&[reg_bytes[0], reg_bytes[1], value])
    }

    /// Write a 16-bit value to a 16-bit register (Big Endian address and value).
    pub fn write_reg16_u16(&mut self, reg: u16, value: u16) -> Result<(), E> {
        let reg_bytes = reg.to_be_bytes();
        let val_bytes = value.to_be_bytes();
        self.write(&[reg_bytes[0], reg_bytes[1], val_bytes[0], val_bytes[1]])
    }

    /// Write a 32-bit value to a 16-bit register (Big Endian address and value).
    pub fn write_reg16_u32(&mut self, reg: u16, value: u32) -> Result<(), E> {
        let reg_bytes = reg.to_be_bytes();
        let val_bytes = value.to_be_bytes();
        self.write(&[
            reg_bytes[0],
            reg_bytes[1],
            val_bytes[0],
            val_bytes[1],
            val_bytes[2],
            val_bytes[3],
        ])
    }

    /// Read a byte from a 16-bit register (Big Endian address).
    pub fn read_reg16_u8(&mut self, reg: u16) -> Result<u8, E> {
        let reg_bytes = reg.to_be_bytes();
        // Use write_read (Repeated Start) if possible,
        // but some devices might prefer write then read.
        // We'll stick to write_read as it's more standard for register reading.
        let mut buf = [0u8; 1];
        self.write_read(&reg_bytes, &mut buf)?;
        Ok(buf[0])
    }

    /// Read a 16-bit value from a 16-bit register (Big Endian address and value).
    pub fn read_reg16_u16(&mut self, reg: u16) -> Result<u16, E> {
        let reg_bytes = reg.to_be_bytes();
        let mut buf = [0u8; 2];
        self.write_read(&reg_bytes, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
