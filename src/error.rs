//! Error types for Modulino operations.

use core::fmt;

/// Error type for Modulino operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// I2C communication error
    I2c(E),
    /// Device not found on the I2C bus
    DeviceNotFound,
    /// Invalid address provided
    InvalidAddress,
    /// Invalid parameter value
    InvalidParameter,
    /// Value out of range
    OutOfRange,
    /// Operation timed out
    Timeout,
    /// Data transmission error
    DataError,
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::I2c(e)
    }
}

impl<E: fmt::Debug> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::I2c(e) => write!(f, "I2C error: {:?}", e),
            Error::DeviceNotFound => write!(f, "Device not found on I2C bus"),
            Error::InvalidAddress => write!(f, "Invalid I2C address"),
            Error::InvalidParameter => write!(f, "Invalid parameter value"),
            Error::OutOfRange => write!(f, "Value out of range"),
            Error::Timeout => write!(f, "Operation timed out"),
            Error::DataError => write!(f, "Data transmission error"),
        }
    }
}

/// Result type alias for Modulino operations.
pub type Result<T, E> = core::result::Result<T, Error<E>>;
