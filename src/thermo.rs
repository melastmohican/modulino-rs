//! Modulino Thermo driver.
//!
//! The Modulino Thermo module uses an HS3003 sensor for temperature
//! and humidity measurements.
//!
//! This module wraps the [`hs3003`](https://crates.io/crates/hs3003) crate
//! to provide a consistent API with other Modulino devices.

use embedded_hal::i2c::I2c;
use embedded_hal::delay::DelayNs;
use hs3003::{Hs3003, Measurement};
pub use hs3003::Error as Hs3003Error;

use crate::{addresses, Result, Error};

/// Temperature and humidity measurement.
///
/// This is a re-export wrapper around the measurement from the `hs3003` crate.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ThermoMeasurement {
    /// Temperature in degrees Celsius
    pub temperature: f32,
    /// Relative humidity in percentage (0-100)
    pub humidity: f32,
}

impl ThermoMeasurement {
    /// Create a new measurement.
    pub const fn new(temperature: f32, humidity: f32) -> Self {
        Self { temperature, humidity }
    }

    /// Check if the measurement is valid.
    pub fn is_valid(&self) -> bool {
        self.temperature > -40.0 && self.temperature < 125.0 &&
        self.humidity >= 0.0 && self.humidity <= 100.0
    }
}

impl From<Measurement> for ThermoMeasurement {
    fn from(m: Measurement) -> Self {
        Self {
            temperature: m.temperature,
            humidity: m.humidity,
        }
    }
}

/// Driver for the Modulino Thermo module (HS3003 sensor).
///
/// This driver wraps the [`hs3003`](https://crates.io/crates/hs3003) crate
/// to provide temperature and humidity measurements.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::Thermo;
///
/// let mut thermo = Thermo::new(i2c);
///
/// // Read temperature and humidity (requires a delay provider)
/// let measurement = thermo.read(&mut delay)?;
/// println!("Temperature: {:.1}°C", measurement.temperature);
/// println!("Humidity: {:.1}%", measurement.humidity);
/// ```
pub struct Thermo<I2C> {
    sensor: Hs3003<I2C>,
}

impl<I2C, E> Thermo<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Thermo instance.
    ///
    /// The HS3003 sensor has a fixed I2C address of 0x44.
    pub fn new(i2c: I2C) -> Self {
        Self {
            sensor: Hs3003::new(i2c),
        }
    }

    /// Get the I2C address.
    ///
    /// The HS3003 has a fixed address of 0x44.
    pub fn address(&self) -> u8 {
        addresses::THERMO
    }

    /// Read temperature and humidity.
    ///
    /// This method triggers a measurement and waits for the result.
    /// It requires a delay provider that implements `DelayNs`.
    ///
    /// # Arguments
    ///
    /// * `delay` - A delay provider for waiting during measurement
    ///
    /// # Returns
    ///
    /// A `ThermoMeasurement` containing temperature (°C) and humidity (% RH).
    pub fn read<D: DelayNs>(&mut self, delay: &mut D) -> Result<ThermoMeasurement, E> {
        match self.sensor.read(delay) {
            Ok(measurement) => Ok(measurement.into()),
            Err(hs3003::Error::I2c(e)) => Err(Error::I2c(e)),
            Err(hs3003::Error::StaleData) => Err(Error::DataError),
        }
    }

    /// Read temperature only.
    ///
    /// Convenience method that reads a full measurement and returns
    /// only the temperature value.
    pub fn temperature<D: DelayNs>(&mut self, delay: &mut D) -> Result<f32, E> {
        Ok(self.read(delay)?.temperature)
    }

    /// Read humidity only.
    ///
    /// Convenience method that reads a full measurement and returns
    /// only the humidity value.
    pub fn humidity<D: DelayNs>(&mut self, delay: &mut D) -> Result<f32, E> {
        Ok(self.read(delay)?.humidity)
    }

    /// Release the I2C bus, returning the underlying `Hs3003` driver.
    pub fn release(self) -> Hs3003<I2C> {
        self.sensor
    }

    /// Get a reference to the underlying `Hs3003` driver.
    ///
    /// This allows access to any additional functionality provided
    /// by the `hs3003` crate directly.
    pub fn inner(&mut self) -> &mut Hs3003<I2C> {
        &mut self.sensor
    }
}
