//! Modulino Movement driver.
//!
//! The Modulino Movement module uses an LSM6DSOX IMU for accelerometer
//! and gyroscope measurements.
//!
//! Note: This driver provides a simplified interface. For full LSM6DSOX
//! functionality, consider using a dedicated LSM6DSOX driver crate.

use crate::{addresses, Error, I2cDevice, Result};
use embedded_hal::i2c::I2c;

// LSM6DSOX register addresses
const LSM6DSOX_CTRL1_XL: u8 = 0x10;
const LSM6DSOX_CTRL2_G: u8 = 0x11;
const LSM6DSOX_CTRL3_C: u8 = 0x12;
const LSM6DSOX_STATUS_REG: u8 = 0x1E;
const LSM6DSOX_OUTX_L_G: u8 = 0x22;
const LSM6DSOX_OUTX_L_A: u8 = 0x28;
const LSM6DSOX_WHO_AM_I: u8 = 0x0F;

const LSM6DSOX_WHO_AM_I_VALUE: u8 = 0x6C;

/// 3-axis measurement values.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MovementValues {
    /// X-axis value
    pub x: f32,
    /// Y-axis value
    pub y: f32,
    /// Z-axis value
    pub z: f32,
}

impl MovementValues {
    /// Create new movement values.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculate the magnitude of the vector.
    pub fn magnitude(&self) -> f32 {
        libm::sqrtf(self.x * self.x + self.y * self.y + self.z * self.z)
    }
}

impl From<(f32, f32, f32)> for MovementValues {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<MovementValues> for (f32, f32, f32) {
    fn from(v: MovementValues) -> Self {
        (v.x, v.y, v.z)
    }
}

/// Driver for the Modulino Movement module (LSM6DSOX IMU).
///
/// # Example
///
/// ```rust,ignore
/// use modulino::Movement;
///
/// let mut movement = Movement::new(i2c)?;
///
/// // Read acceleration
/// let accel = movement.acceleration()?;
/// println!("Accel: x={:.2}g, y={:.2}g, z={:.2}g", accel.x, accel.y, accel.z);
///
/// // Read angular velocity
/// let gyro = movement.angular_velocity()?;
/// println!("Gyro: x={:.2}dps, y={:.2}dps, z={:.2}dps", gyro.x, gyro.y, gyro.z);
/// ```
pub struct Movement<I2C> {
    device: I2cDevice<I2C>,
    accel_sensitivity: f32,
    gyro_sensitivity: f32,
}

impl<I2C, E> Movement<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Movement instance with the default address (0x6A).
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::MOVEMENT[0])
    }

    /// Create a new Movement instance with a custom address.
    ///
    /// Valid addresses are 0x6A or 0x6B depending on the SA0 pin configuration.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut movement = Self {
            device: I2cDevice::new(i2c, address),
            accel_sensitivity: 0.061, // mg/LSB at ±2g
            gyro_sensitivity: 8.75,   // mdps/LSB at ±250dps
        };

        // Verify device identity
        let who_am_i = movement.device.read_reg(LSM6DSOX_WHO_AM_I)?;
        if who_am_i != LSM6DSOX_WHO_AM_I_VALUE {
            return Err(Error::DeviceNotFound);
        }

        // Initialize with default settings
        movement.init()?;

        Ok(movement)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Initialize the sensor with default settings.
    fn init(&mut self) -> Result<(), E> {
        // Software reset
        self.device.write_reg(LSM6DSOX_CTRL3_C, 0x01)?;

        // Wait for reset (in a real implementation, add delay here)

        // Configure accelerometer: 104 Hz, ±2g
        self.device.write_reg(LSM6DSOX_CTRL1_XL, 0x40)?;
        self.accel_sensitivity = 0.061; // mg/LSB at ±2g

        // Configure gyroscope: 104 Hz, ±250 dps
        self.device.write_reg(LSM6DSOX_CTRL2_G, 0x40)?;
        self.gyro_sensitivity = 8.75; // mdps/LSB at ±250dps

        // Enable BDU (Block Data Update)
        self.device.write_reg(LSM6DSOX_CTRL3_C, 0x44)?;

        Ok(())
    }

    /// Read acceleration values.
    ///
    /// Returns acceleration in g (gravitational units).
    pub fn acceleration(&mut self) -> Result<MovementValues, E> {
        let mut buf = [0u8; 6];
        self.device.read_regs(LSM6DSOX_OUTX_L_A, &mut buf)?;

        let x_raw = i16::from_le_bytes([buf[0], buf[1]]);
        let y_raw = i16::from_le_bytes([buf[2], buf[3]]);
        let z_raw = i16::from_le_bytes([buf[4], buf[5]]);

        // Convert to g
        let scale = self.accel_sensitivity / 1000.0;
        Ok(MovementValues {
            x: x_raw as f32 * scale,
            y: y_raw as f32 * scale,
            z: z_raw as f32 * scale,
        })
    }

    /// Get the magnitude of acceleration.
    ///
    /// When at rest on Earth, this should be approximately 1.0g due to gravity.
    pub fn acceleration_magnitude(&mut self) -> Result<f32, E> {
        Ok(self.acceleration()?.magnitude())
    }

    /// Read angular velocity (gyroscope) values.
    ///
    /// Returns angular velocity in degrees per second (dps).
    pub fn angular_velocity(&mut self) -> Result<MovementValues, E> {
        let mut buf = [0u8; 6];
        self.device.read_regs(LSM6DSOX_OUTX_L_G, &mut buf)?;

        let x_raw = i16::from_le_bytes([buf[0], buf[1]]);
        let y_raw = i16::from_le_bytes([buf[2], buf[3]]);
        let z_raw = i16::from_le_bytes([buf[4], buf[5]]);

        // Convert to dps
        let scale = self.gyro_sensitivity / 1000.0;
        Ok(MovementValues {
            x: x_raw as f32 * scale,
            y: y_raw as f32 * scale,
            z: z_raw as f32 * scale,
        })
    }

    /// Alias for `angular_velocity()`.
    pub fn gyro(&mut self) -> Result<MovementValues, E> {
        self.angular_velocity()
    }

    /// Check if new data is available.
    pub fn data_ready(&mut self) -> Result<bool, E> {
        let status = self.device.read_reg(LSM6DSOX_STATUS_REG)?;
        // Check XLDA (bit 0) or GDA (bit 1)
        Ok((status & 0x03) != 0)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
