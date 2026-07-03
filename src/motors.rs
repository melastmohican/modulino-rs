//! Modulino Motors driver.
//!
//! The Modulino Motors module provides an interface to control DC and stepper motors.
//!
//! > [!WARNING]
//! > **EXPERIMENTAL**: This driver is a work-in-progress and has only been verified via
//! > unit tests using I2C mocks. It has NOT yet been tested on physical Modulino hardware.

use crate::{addresses, Error, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Supported motor current decay modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DecayMode {
    /// Slow decay (smoother and quieter motor operation)
    #[default]
    Slow = 0,
    /// Mixed decay (30% Fast, 70% Slow)
    Mixed30Fast70Slow = 1,
    /// Mixed decay (60% Fast, 40% Slow)
    Mixed60Fast40Slow = 2,
    /// Fast decay (quick response, but more noise/vibration)
    Fast = 3,
}

impl From<u8> for DecayMode {
    fn from(value: u8) -> Self {
        match value {
            1 => DecayMode::Mixed30Fast70Slow,
            2 => DecayMode::Mixed60Fast40Slow,
            3 => DecayMode::Fast,
            _ => DecayMode::Slow,
        }
    }
}

/// Driver for the Modulino Motors module.
pub struct Motors<I2C> {
    device: I2cDevice<I2C>,
    steps_per_revolution: i16,
    speed_a: u8,
    invert_a: bool,
    speed_b: u8,
    invert_b: bool,
    stepper_direction_inverted: bool,
    frequency_hz: u16,
    mode_stepper: bool,
    half_step_enabled: bool,
    decay_mode: DecayMode,
    hfs_enabled: bool,
    release_on_complete: bool,
    busy: bool,
    sense_raw_a: u16,
    sense_raw_b: u16,
}

impl<I2C, E> Motors<I2C>
where
    I2C: I2c<Error = E>,
{
    const CMD_MODE: u8 = b'M';
    const CMD_SPEED_DC: u8 = b'S';
    const CMD_STEPPER: u8 = b'G';
    const CMD_DECAY: u8 = b'T';
    const CMD_STEP_MODE: u8 = b'H';
    const CMD_FREQ_DC: u8 = b'F';
    const CMD_HFS: u8 = b'X';

    const FLAG_BUSY: u8 = 0x01;
    const FLAG_MODE: u8 = 0x02;
    const FLAG_STEP_MODE: u8 = 0x04;
    const FLAG_HFS: u8 = 0x08;
    const FLAG_DECAY_MASK: u8 = 0x30;
    const FLAG_DECAY_SHIFT: u8 = 4;
    const FLAG_RELEASE: u8 = 0x40;

    /// Max raw speed limit for DC motor channels
    pub const MAX_SPEED: i16 = 32767;

    /// Create a new Motors instance with the default address and auto-discover steps.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::MOTORS)
    }

    /// Discover if a Motors module is connected.
    pub fn discover(i2c: &mut I2C) -> Result<u8, E> {
        let addresses = [addresses::MOTORS];
        for &addr in &addresses {
            if i2c.write(addr, &[]).is_ok() {
                return Ok(addr);
            }
        }
        i2c.write(addresses[0], &[])
            .map(|_| addresses[0])
            .map_err(Error::I2c)
    }

    /// Create a new Motors instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut motors = Self {
            device: I2cDevice::new(i2c, address),
            steps_per_revolution: -1,
            speed_a: 0,
            invert_a: false,
            speed_b: 0,
            invert_b: false,
            stepper_direction_inverted: false,
            frequency_hz: 20000,
            mode_stepper: false,
            half_step_enabled: false,
            decay_mode: DecayMode::Slow,
            hfs_enabled: false,
            release_on_complete: false,
            busy: false,
            sense_raw_a: 0,
            sense_raw_b: 0,
        };

        // Query initial state
        motors.update()?;

        Ok(motors)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Stop both DC outputs.
    pub fn stop(&mut self) -> Result<(), E> {
        self.set_dc_speed_raw(0, 0)
    }

    /// Release stepper coils immediately.
    pub fn release_coils(&mut self) -> Result<(), E> {
        let cmd = [Self::CMD_STEPPER, 0, 0, 0, 0, 1, 0, 1];
        self.send_command(&cmd)
    }

    /// Energize and hold stepper coils immediately.
    pub fn hold_coils(&mut self) -> Result<(), E> {
        let cmd = [Self::CMD_STEPPER, 0, 0, 0, 0, 1, 0, 0];
        self.send_command(&cmd)
    }

    /// Set motor A speed as percentage of full scale (0..100).
    pub fn set_speed_a(&mut self, percent: u8) -> Result<(), E> {
        if percent > 100 {
            return Err(Error::InvalidParameter);
        }
        self.speed_a = percent;
        self.update_dc_speed_from_percent()
    }

    /// Set motor B speed as percentage of full scale (0..100).
    pub fn set_speed_b(&mut self, percent: u8) -> Result<(), E> {
        if percent > 100 {
            return Err(Error::InvalidParameter);
        }
        self.speed_b = percent;
        self.update_dc_speed_from_percent()
    }

    /// Set inversion flag for motor A direction.
    pub fn set_invert_a(&mut self, invert: bool) -> Result<(), E> {
        self.invert_a = invert;
        self.update_dc_speed_from_percent()
    }

    /// Set inversion flag for motor B direction.
    pub fn set_invert_b(&mut self, invert: bool) -> Result<(), E> {
        self.invert_b = invert;
        self.update_dc_speed_from_percent()
    }

    /// Get configured speed percentage for motor A.
    pub fn speed_a(&self) -> u8 {
        self.speed_a
    }

    /// Get configured speed percentage for motor B.
    pub fn speed_b(&self) -> u8 {
        self.speed_b
    }

    /// Get inversion state for motor A.
    pub fn invert_a(&self) -> bool {
        self.invert_a
    }

    /// Get inversion state for motor B.
    pub fn invert_b(&self) -> bool {
        self.invert_b
    }

    /// Set inversion flag for stepper direction.
    pub fn set_stepper_direction_inverted(&mut self, invert: bool) {
        self.stepper_direction_inverted = invert;
    }

    /// Get inversion state for stepper direction.
    pub fn stepper_direction_inverted(&self) -> bool {
        self.stepper_direction_inverted
    }

    /// Set raw signed DC speeds for channels A and B (-32767..32767).
    pub fn set_dc_speed_raw(&mut self, speed_a: i16, speed_b: i16) -> Result<(), E> {
        if !(-Self::MAX_SPEED..=Self::MAX_SPEED).contains(&speed_a)
            || !(-Self::MAX_SPEED..=Self::MAX_SPEED).contains(&speed_b)
        {
            return Err(Error::InvalidParameter);
        }

        let sa_bytes = speed_a.to_le_bytes();
        let sb_bytes = speed_b.to_le_bytes();

        let cmd = [
            Self::CMD_SPEED_DC,
            sa_bytes[0],
            sa_bytes[1],
            sb_bytes[0],
            sb_bytes[1],
        ];
        self.send_command(&cmd)
    }

    /// Command a stepper move using period-based speed.
    ///
    /// # Arguments
    ///
    /// * `steps` - Signed number of steps.
    /// * `speed_period` - Step period in 0.1 ms timer ticks (1..65535).
    /// * `release_delay_ms` - Delay before releasing coils after move completion (0 to hold indefinitely).
    pub fn move_stepper(
        &mut self,
        mut steps: i32,
        speed_period: u16,
        release_delay_ms: u8,
    ) -> Result<(), E> {
        if speed_period < 1 {
            return Err(Error::InvalidParameter);
        }

        if self.stepper_direction_inverted {
            steps = -steps;
        }

        let steps_bytes = steps.to_le_bytes();
        let sp_bytes = speed_period.to_le_bytes();

        let cmd = [
            Self::CMD_STEPPER,
            steps_bytes[0],
            steps_bytes[1],
            steps_bytes[2],
            steps_bytes[3],
            sp_bytes[0],
            sp_bytes[1],
            release_delay_ms,
        ];
        self.send_command(&cmd)
    }

    /// Command a stepper move using target RPM.
    pub fn move_stepper_rpm(
        &mut self,
        steps: i32,
        rpm: f32,
        release_delay_ms: u8,
    ) -> Result<(), E> {
        if self.steps_per_revolution < 1 || rpm <= 0.0 {
            return Err(Error::InvalidParameter);
        }

        let steps_per_rev =
            self.steps_per_revolution as u32 * if self.half_step_enabled { 2 } else { 1 };
        let pulses_per_minute = rpm * steps_per_rev as f32;
        if pulses_per_minute <= 0.0 {
            return Err(Error::InvalidParameter);
        }

        let period_float = 600000.0 / pulses_per_minute;
        if !(1.0..=65535.0).contains(&period_float) {
            return Err(Error::InvalidParameter);
        }

        let period_ticks = period_float as u16;
        self.move_stepper(steps, period_ticks, release_delay_ms)
    }

    /// Set decay mode.
    pub fn set_decay(&mut self, decay_mode: DecayMode) -> Result<(), E> {
        let cmd = [Self::CMD_DECAY, decay_mode as u8];
        self.send_command(&cmd)?;
        self.decay_mode = decay_mode;
        Ok(())
    }

    /// Set DC PWM frequency (200..60000 Hz).
    pub fn set_frequency(&mut self, frequency_hz: u16) -> Result<(), E> {
        if !(200..=60000).contains(&frequency_hz) {
            return Err(Error::InvalidParameter);
        }
        let freq_bytes = frequency_hz.to_le_bytes();
        let cmd = [Self::CMD_FREQ_DC, freq_bytes[0], freq_bytes[1]];
        self.send_command(&cmd)?;
        self.frequency_hz = frequency_hz;
        Ok(())
    }

    /// Configure half-full-scale current sense mode.
    pub fn set_half_full_scale_enabled(&mut self, enabled: bool) -> Result<(), E> {
        let cmd = [Self::CMD_HFS, enabled as u8];
        self.send_command(&cmd)?;
        self.hfs_enabled = enabled;
        Ok(())
    }

    /// Switch operating mode between DC (false) and Stepper (true).
    pub fn set_stepper_mode_enabled(&mut self, enabled: bool) -> Result<(), E> {
        let mode = if enabled { 1 } else { 0 };
        let cmd = [Self::CMD_MODE, mode];
        self.send_command(&cmd)?;
        self.mode_stepper = enabled;
        Ok(())
    }

    /// Configure step mode.
    ///
    /// * `enabled` - True for half-step, false for full-step.
    pub fn set_half_step_enabled(&mut self, enabled: bool) -> Result<(), E> {
        let cmd = [Self::CMD_STEP_MODE, enabled as u8];
        self.send_command(&cmd)?;
        self.half_step_enabled = enabled;
        Ok(())
    }

    /// Read latest telemetry from the module.
    pub fn update(&mut self) -> Result<(), E> {
        let mut buf = [0u8; 6]; // 1 pinstrap + 5 data
        self.device.read(&mut buf)?;

        self.sense_raw_a = u16::from_le_bytes([buf[1], buf[2]]);
        self.sense_raw_b = u16::from_le_bytes([buf[3], buf[4]]);

        let flags = buf[5];
        self.busy = (flags & Self::FLAG_BUSY) != 0;
        self.mode_stepper = (flags & Self::FLAG_MODE) != 0;
        self.half_step_enabled = (flags & Self::FLAG_STEP_MODE) != 0;
        self.hfs_enabled = (flags & Self::FLAG_HFS) != 0;
        self.decay_mode =
            DecayMode::from((flags & Self::FLAG_DECAY_MASK) >> Self::FLAG_DECAY_SHIFT);
        self.release_on_complete = (flags & Self::FLAG_RELEASE) != 0;

        Ok(())
    }

    /// Get raw current-sense ADC reading for channel A.
    pub fn sensed_raw_a(&self) -> u16 {
        self.sense_raw_a
    }

    /// Get raw current-sense ADC reading for channel B.
    pub fn sensed_raw_b(&self) -> u16 {
        self.sense_raw_b
    }

    /// Get estimated channel A current in mA.
    pub fn sensed_current_a(&self) -> f32 {
        self.sense_raw_to_ma(self.sense_raw_a)
    }

    /// Get estimated channel B current in mA.
    pub fn sensed_current_b(&self) -> f32 {
        self.sense_raw_to_ma(self.sense_raw_b)
    }

    /// Get busy flag from last telemetry update.
    pub fn busy(&self) -> bool {
        self.busy
    }

    /// Get HFS state from last telemetry update.
    pub fn half_full_scale_enabled(&self) -> bool {
        self.hfs_enabled
    }

    /// Get current operating mode from last telemetry update.
    pub fn stepper_mode_enabled(&self) -> bool {
        self.mode_stepper
    }

    /// Get step mode from last telemetry update.
    pub fn half_step_enabled(&self) -> bool {
        self.half_step_enabled
    }

    /// Get the release-on-complete state from last telemetry update.
    pub fn release_on_complete(&self) -> bool {
        self.release_on_complete
    }

    /// Get decay mode from last telemetry update.
    pub fn decay_mode(&self) -> DecayMode {
        self.decay_mode
    }

    /// Get configured DC PWM frequency.
    pub fn frequency(&self) -> u16 {
        self.frequency_hz
    }

    /// Get configured motor full-step resolution.
    pub fn steps_per_revolution(&self) -> i16 {
        self.steps_per_revolution
    }

    /// Set motor full-step resolution.
    pub fn set_steps_per_revolution(&mut self, value: i16) -> Result<(), E> {
        if value < 1 {
            return Err(Error::InvalidParameter);
        }
        self.steps_per_revolution = value;
        Ok(())
    }

    /// Send a command padded to firmware wire length (8 bytes).
    fn send_command(&mut self, msg: &[u8]) -> Result<(), E> {
        if msg.len() > 8 {
            return Err(Error::InvalidParameter);
        }
        let mut padded = [0u8; 8];
        padded[..msg.len()].copy_from_slice(msg);
        self.device.write(&padded).map_err(Error::I2c)
    }

    /// Helper to recompute and push DC speeds from percentage and invert settings.
    fn update_dc_speed_from_percent(&mut self) -> Result<(), E> {
        let mut raw_a = ((self.speed_a as i32 * Self::MAX_SPEED as i32) / 100) as i16;
        let mut raw_b = ((self.speed_b as i32 * Self::MAX_SPEED as i32) / 100) as i16;
        if self.invert_a {
            raw_a = -raw_a;
        }
        if self.invert_b {
            raw_b = -raw_b;
        }
        self.set_dc_speed_raw(raw_a, raw_b)
    }

    /// Convert raw telemetry count to motor current estimate.
    fn sense_raw_to_ma(&self, raw: u16) -> f32 {
        let kisen = if self.hfs_enabled { 3750.0 } else { 7500.0 };
        (raw as f32 * 3300.0 * kisen) / (4095.0 * 4700.0)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
