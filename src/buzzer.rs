//! Modulino Buzzer driver.
//!
//! The Modulino Buzzer module contains a piezo speaker that can play tones
//! at specified frequencies.

use embedded_hal::i2c::I2c;
use crate::{addresses, Result};

/// Musical note frequencies in Hz.
///
/// Notes are named with the note letter, optional sharp (S), and octave number.
/// For example, `A4` is 440 Hz (standard tuning), `CS5` is C# in octave 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
#[allow(missing_docs)]
pub enum Note {
    // Octave 3
    FS3 = 185,
    G3 = 196,
    GS3 = 208,
    A3 = 220,
    AS3 = 233,
    B3 = 247,
    
    // Octave 4
    C4 = 262,
    CS4 = 277,
    D4 = 294,
    DS4 = 311,
    E4 = 330,
    F4 = 349,
    FS4 = 370,
    G4 = 392,
    GS4 = 415,
    A4 = 440,
    AS4 = 466,
    B4 = 494,
    
    // Octave 5
    C5 = 523,
    CS5 = 554,
    D5 = 587,
    DS5 = 622,
    E5 = 659,
    F5 = 698,
    FS5 = 740,
    G5 = 784,
    GS5 = 831,
    A5 = 880,
    AS5 = 932,
    B5 = 988,
    
    // Octave 6
    C6 = 1047,
    CS6 = 1109,
    D6 = 1175,
    DS6 = 1245,
    E6 = 1319,
    F6 = 1397,
    FS6 = 1480,
    G6 = 1568,
    GS6 = 1661,
    A6 = 1760,
    AS6 = 1865,
    B6 = 1976,
    
    // Octave 7
    C7 = 2093,
    CS7 = 2217,
    D7 = 2349,
    DS7 = 2489,
    E7 = 2637,
    F7 = 2794,
    FS7 = 2960,
    G7 = 3136,
    GS7 = 3322,
    A7 = 3520,
    AS7 = 3729,
    B7 = 3951,
    
    // Octave 8
    C8 = 4186,
    CS8 = 4435,
    D8 = 4699,
    DS8 = 4978,
    
    /// Silence (rest)
    Rest = 0,
}

impl Note {
    /// Get the frequency in Hz.
    pub const fn frequency(&self) -> u16 {
        *self as u16
    }
}

impl From<Note> for u16 {
    fn from(note: Note) -> Self {
        note.frequency()
    }
}

/// Driver for the Modulino Buzzer module.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::{Buzzer, Note};
///
/// let mut buzzer = Buzzer::new(i2c)?;
///
/// // Play a tone at 440 Hz for 500ms
/// buzzer.tone(440, 500)?;
///
/// // Play a note
/// buzzer.play_note(Note::C5, 1000)?;
///
/// // Stop the tone
/// buzzer.no_tone()?;
/// ```
pub struct Buzzer<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Buzzer<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Minimum supported frequency in Hz.
    pub const MIN_FREQUENCY: u16 = 180;

    /// Create a new Buzzer instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::BUZZER)
    }

    /// Create a new Buzzer instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut buzzer = Self { i2c, address };
        
        // Initialize with no tone
        buzzer.no_tone()?;
        
        Ok(buzzer)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Play a tone at the specified frequency.
    ///
    /// # Arguments
    ///
    /// * `frequency` - The frequency in Hz (minimum 180 Hz, or 0 for silence)
    /// * `duration_ms` - The duration in milliseconds (0xFFFF for indefinite)
    ///
    /// # Note
    ///
    /// Frequencies below 180 Hz (except 0) are not supported by the hardware.
    pub fn tone(&mut self, frequency: u16, duration_ms: u16) -> Result<(), E> {
        let freq_bytes = (frequency as u32).to_le_bytes();
        let duration_bytes = (duration_ms as u32).to_le_bytes();
        
        let data = [
            freq_bytes[0],
            freq_bytes[1],
            freq_bytes[2],
            freq_bytes[3],
            duration_bytes[0],
            duration_bytes[1],
            duration_bytes[2],
            duration_bytes[3],
        ];
        
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Play a tone indefinitely until stopped.
    pub fn tone_continuous(&mut self, frequency: u16) -> Result<(), E> {
        self.tone(frequency, 0xFFFF)
    }

    /// Play a musical note.
    ///
    /// # Arguments
    ///
    /// * `note` - The note to play
    /// * `duration_ms` - The duration in milliseconds
    pub fn play_note(&mut self, note: Note, duration_ms: u16) -> Result<(), E> {
        self.tone(note.frequency(), duration_ms)
    }

    /// Stop playing any tone.
    pub fn no_tone(&mut self) -> Result<(), E> {
        let data = [0u8; 8];
        self.i2c.write(self.address, &data)?;
        Ok(())
    }

    /// Alias for `no_tone()`.
    pub fn stop(&mut self) -> Result<(), E> {
        self.no_tone()
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
