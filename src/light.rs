//! Modulino Light sensor driver.
//!
//! The Modulino Light module uses an LTR-381RGB sensor for
//! measuring RGB color, infrared, and ambient light levels.
//!
//! The Modulino Light module uses an LTR-381RGB sensor for
//! measuring RGB color, infrared, and ambient light levels.
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
    /// Raw ambient light (Green channel)
    pub raw_lux: u32,
    /// Calculated ambient light in Lux
    pub lux: f32,
}

impl LightMeasurement {
    /// Get the approximate color name for this measurement.
    pub fn color_name(&self) -> ColorName {
        ColorName::from_rgb(self.red, self.green, self.blue)
    }
}

/// A human-readable color description.
///
/// Implements `Display` to provide strings like "VERY PALE DARK BLUE".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorName {
    base: &'static str,
    lightness: Option<&'static str>,
    saturation: Option<&'static str>,
}

impl ColorName {
    fn from_rgb(r: u32, g: u32, b: u32) -> Self {
        let (h, s, l) = rgb_to_hsl(r, g, b);

        // Special cases
        if l > 90.0 {
            return Self::new("WHITE", None, None);
        }
        if l <= 0.20 {
            return Self::new("BLACK", None, None);
        }
        if s < 10.0 {
            return if l < 50.0 {
                Self::new("DARK GRAY", None, None)
            } else {
                Self::new("LIGHT GRAY", None, None)
            };
        }

        let base = match h {
            h if !(15.0..345.0).contains(&h) => "RED",
            h if h < 45.0 => "ORANGE",
            h if h < 75.0 => "YELLOW",
            h if h < 105.0 => "LIME",
            h if h < 135.0 => "GREEN",
            h if h < 165.0 => "SPRING GREEN",
            h if h < 195.0 => "CYAN",
            h if h < 225.0 => "AZURE",
            h if h < 255.0 => "BLUE",
            h if h < 285.0 => "VIOLET",
            h if h < 315.0 => "MAGENTA",
            _ => "ROSE",
        };

        let lightness = match l {
            l if l < 20.0 => Some("VERY DARK"),
            l if l < 40.0 => Some("DARK"),
            l if l > 80.0 => Some("VERY LIGHT"),
            l if l > 60.0 => Some("LIGHT"),
            _ => None,
        };

        let saturation = match s {
            s if s < 20.0 => Some("VERY PALE"),
            s if s < 40.0 => Some("PALE"),
            s if s > 95.0 => Some("VERY VIVID"),
            s if s > 80.0 => Some("VIVID"),
            _ => None,
        };

        Self::new(base, lightness, saturation)
    }

    const fn new(
        base: &'static str,
        lightness: Option<&'static str>,
        saturation: Option<&'static str>,
    ) -> Self {
        Self {
            base,
            lightness,
            saturation,
        }
    }
}

impl core::fmt::Display for ColorName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(s) = self.saturation {
            write!(f, "{} ", s)?;
        }
        if let Some(l) = self.lightness {
            write!(f, "{} ", l)?;
        }
        write!(f, "{}", self.base)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ColorName {
    fn format(&self, f: defmt::Formatter) {
        if let Some(s) = self.saturation {
            defmt::write!(f, "{} ", s);
        }
        if let Some(l) = self.lightness {
            defmt::write!(f, "{} ", l);
        }
        defmt::write!(f, "{}", self.base);
    }
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

impl Gain {
    /// Get the numeric gain factor.
    pub fn factor(&self) -> f32 {
        match self {
            Gain::Gain1x => 1.0,
            Gain::Gain3x => 3.0,
            Gain::Gain6x => 6.0,
            Gain::Gain9x => 9.0,
            Gain::Gain18x => 18.0,
        }
    }
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

impl Resolution {
    /// Get the timing factor (Integration Time / 100ms).
    pub fn factor(&self) -> f32 {
        match self {
            Resolution::Res20Bit => 4.0,  // 400ms
            Resolution::Res19Bit => 2.0,  // 200ms
            Resolution::Res18Bit => 1.0,  // 100ms
            Resolution::Res17Bit => 0.5,  // 50ms
            Resolution::Res16Bit => 0.25, // 25ms
        }
    }

    /// Get the maximum ADC value for this resolution.
    pub fn max_value(&self) -> u32 {
        match self {
            Resolution::Res20Bit => 1048575,
            Resolution::Res19Bit => 524287,
            Resolution::Res18Bit => 262143,
            Resolution::Res17Bit => 131071,
            Resolution::Res16Bit => 65534, // As used in Arduino library
        }
    }
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
    current_gain: Gain,
    current_res: Resolution,
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
            current_gain: Gain::Gain18x,
            current_res: Resolution::Res16Bit,
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

        // Default config: 18x gain, 16-bit res, 25ms rate (Standard Arduino Modulino)
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
            .map_err(Error::I2c)?;
        self.current_gain = gain;
        Ok(())
    }

    /// Set ADC resolution and measurement rate.
    pub fn set_config(&mut self, res: Resolution, rate: MeasurementRate) -> Result<(), E> {
        let val = ((res as u8) << 4) | (rate as u8);
        self.i2c
            .write(self.address, &[Self::REG_MEAS_RATE, val])
            .map_err(Error::I2c)?;
        self.current_res = res;
        Ok(())
    }

    /// Read the infrared channel value.
    pub fn ir(&mut self) -> Result<u32, E> {
        self.read_channel(Self::REG_DATA_IR)
    }

    /// Read the raw ambient light level (Green channel).
    pub fn raw_lux(&mut self) -> Result<u32, E> {
        self.read_channel(Self::REG_DATA_GREEN)
    }

    /// Read the calculated ambient light level in Lux.
    pub fn lux(&mut self) -> Result<f32, E> {
        let ir = self.ir()?;
        let raw_lux = self.raw_lux()?;
        Ok(calculate_lux(
            ir,
            raw_lux,
            self.current_gain,
            self.current_res,
        ))
    }

    /// Read all sensor values including calculated Lux.
    pub fn read(&mut self) -> Result<LightMeasurement, E> {
        let ir = self.ir()?;
        let green = self.read_channel(Self::REG_DATA_GREEN)?;
        let red = self.read_channel(Self::REG_DATA_RED)?;
        let blue = self.read_channel(Self::REG_DATA_BLUE)?;

        Ok(LightMeasurement {
            ir,
            green,
            red,
            blue,
            raw_lux: green,
            lux: calculate_lux(ir, green, self.current_gain, self.current_res),
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

fn calculate_lux(ir: u32, raw_lux: u32, gain: Gain, res: Resolution) -> f32 {
    if raw_lux == 0 {
        return 0.0;
    }
    let gain_factor = gain.factor();
    let int_factor = res.factor();

    // Basic lux formula: (0.8 * raw_lux) / (gain * int_time)
    let lux = (0.8 * raw_lux as f32) / (gain_factor * int_factor);

    // IR compensation factor: 1 - 0.033 * (IR / Green)
    let ratio = ir as f32 / raw_lux as f32;
    let ratio_clamped = if ratio > 30.0 { 30.0 } else { ratio };
    let factor = 1.0 - 0.033 * ratio_clamped;

    if factor < 0.0 {
        0.0
    } else {
        lux * factor
    }
}

fn rgb_to_hsl(r: u32, g: u32, b: u32) -> (f32, f32, f32) {
    // Normalization based on 16-bit resolution (default for Color readings)
    const MAX_VAL: f32 = 65534.0;

    let r_n = (r as f32).min(MAX_VAL) / MAX_VAL;
    let g_n = (g as f32).min(MAX_VAL) / MAX_VAL;
    let b_n = (b as f32).min(MAX_VAL) / MAX_VAL;

    let max = r_n.max(g_n).max(b_n);
    let min = r_n.min(g_n).min(b_n);
    let delta = max - min;

    let l = (max + min) / 2.0;
    let mut s = 0.0;
    let mut h = 0.0;

    if delta != 0.0 {
        s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        if max == r_n {
            h = (g_n - b_n) / delta + (if g_n < b_n { 6.0 } else { 0.0 });
        } else if max == g_n {
            h = (b_n - r_n) / delta + 2.0;
        } else {
            h = (r_n - g_n) / delta + 4.0;
        }
        h *= 60.0;
    }

    // Output scales: H(0-360), S(0-100), L(0-100)
    (h, s * 100.0, l * 100.0)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::i2c::Mock as I2cMock;
    use embedded_hal_mock::eh1::i2c::Transaction;
    use std::vec;

    #[test]
    fn test_calculate_lux() {
        // 18x gain, 16-bit (0.25 int_factor)
        // (0.8 * 1000) / (18 * 0.25) = 800 / 4.5 = 177.77
        let raw = 1000;
        let ir = 500;
        let lux = calculate_lux(ir, raw, Gain::Gain18x, Resolution::Res16Bit);
        // Ratio = 0.5. Factor = 1 - 0.033 * 0.5 = 0.9835
        // 177.77 * 0.9835 = 174.83
        assert!((lux - 174.83).abs() < 0.1);
    }

    #[test]
    fn test_color_name_basic() {
        // Pure Red
        let cn = ColorName::from_rgb(65534, 0, 0);
        assert_eq!(cn.base, "RED");
        // S=100.0, L=50.0 -> VIVID RED
        assert_eq!(cn.saturation, Some("VERY VIVID"));
        assert_eq!(cn.lightness, None);

        // Pure Green
        let cn = ColorName::from_rgb(0, 65534, 0);
        assert_eq!(cn.base, "GREEN");

        // Gray
        let cn = ColorName::from_rgb(40000, 40000, 40000);
        assert_eq!(cn.base, "LIGHT GRAY");

        // Black
        let cn = ColorName::from_rgb(10, 10, 10);
        assert_eq!(cn.base, "BLACK");
    }

    #[test]
    fn test_light_read() {
        let expectations = [
            Transaction::write_read(addresses::LIGHT, vec![0x0A], vec![0x01, 0x02, 0x00]),
            Transaction::write_read(addresses::LIGHT, vec![0x0D], vec![0x03, 0x04, 0x00]),
            Transaction::write_read(addresses::LIGHT, vec![0x10], vec![0x05, 0x06, 0x00]),
            Transaction::write_read(addresses::LIGHT, vec![0x13], vec![0x07, 0x08, 0x00]),
        ];
        let mut i2c = I2cMock::new(&expectations);
        let mut light = Light::new(i2c.clone());

        let meas = light.read().unwrap();
        assert_eq!(meas.ir, 0x0201);
        assert_eq!(meas.green, 0x0403);
        assert_eq!(meas.red, 0x0605);
        assert_eq!(meas.blue, 0x0807);
        assert_eq!(meas.raw_lux, 0x0403);

        i2c.done();
    }
}
