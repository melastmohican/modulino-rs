//! Modulino Buttons driver.
//!
//! The Modulino Buttons module has three buttons (A, B, C), each with an associated LED.

use crate::{addresses, I2cDevice, Result};
use embedded_hal::i2c::I2c;

/// Button state representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ButtonState {
    /// Button A pressed state
    pub a: bool,
    /// Button B pressed state
    pub b: bool,
    /// Button C pressed state
    pub c: bool,
}

impl ButtonState {
    /// Check if any button is pressed.
    pub fn any_pressed(&self) -> bool {
        self.a || self.b || self.c
    }

    /// Check if all buttons are pressed.
    pub fn all_pressed(&self) -> bool {
        self.a && self.b && self.c
    }
}

/// LED state for a single button LED.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ButtonLed {
    value: bool,
}

impl ButtonLed {
    /// Create a new LED state (off by default).
    pub const fn new() -> Self {
        Self { value: false }
    }

    /// Check if the LED is on.
    pub fn is_on(&self) -> bool {
        self.value
    }

    /// Turn the LED on.
    pub fn on(&mut self) {
        self.value = true;
    }

    /// Turn the LED off.
    pub fn off(&mut self) {
        self.value = false;
    }

    /// Set the LED state.
    pub fn set(&mut self, on: bool) {
        self.value = on;
    }

    /// Toggle the LED state.
    pub fn toggle(&mut self) {
        self.value = !self.value;
    }
}

/// Driver for the Modulino Buttons module.
///
/// # Example
///
/// ```rust,ignore
/// use modulino::Buttons;
///
/// let mut buttons = Buttons::new(i2c)?;
///
/// // Read button states
/// let state = buttons.read()?;
/// if state.a {
///     println!("Button A pressed!");
/// }
///
/// // Control LEDs
/// buttons.led_a.on();
/// buttons.led_b.off();
/// buttons.led_c.set(state.c);
/// buttons.update_leds()?;
/// ```
pub struct Buttons<I2C> {
    device: I2cDevice<I2C>,
    /// LED A state
    pub led_a: ButtonLed,
    /// LED B state
    pub led_b: ButtonLed,
    /// LED C state
    pub led_c: ButtonLed,
    current_state: ButtonState,
}

impl<I2C, E> Buttons<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new Buttons instance with the default address.
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Self::new_with_address(i2c, addresses::BUTTONS)
    }

    /// Create a new Buttons instance with a custom address.
    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, E> {
        let mut buttons = Self {
            device: I2cDevice::new(i2c, address),
            led_a: ButtonLed::new(),
            led_b: ButtonLed::new(),
            led_c: ButtonLed::new(),
            current_state: ButtonState::default(),
        };

        // Verify device is present
        buttons.read()?;

        Ok(buttons)
    }

    /// Get the I2C address.
    pub fn address(&self) -> u8 {
        self.device.address
    }

    /// Read the current button states.
    ///
    /// Returns a `ButtonState` struct with the pressed state of each button.
    pub fn read(&mut self) -> Result<ButtonState, E> {
        let mut buf = [0u8; 4]; // 1 pinstrap + 3 button states
        self.device.read(&mut buf)?;

        // Skip first byte (pinstrap address)
        self.current_state = ButtonState {
            a: buf[1] != 0,
            b: buf[2] != 0,
            c: buf[3] != 0,
        };

        Ok(self.current_state)
    }

    /// Get the last read button state without performing I2C communication.
    pub fn state(&self) -> ButtonState {
        self.current_state
    }

    /// Check if button A is pressed (uses cached state).
    pub fn button_a_pressed(&self) -> bool {
        self.current_state.a
    }

    /// Check if button B is pressed (uses cached state).
    pub fn button_b_pressed(&self) -> bool {
        self.current_state.b
    }

    /// Check if button C is pressed (uses cached state).
    pub fn button_c_pressed(&self) -> bool {
        self.current_state.c
    }

    /// Update the LED states on the device.
    ///
    /// This writes the current LED states to the hardware.
    pub fn update_leds(&mut self) -> Result<(), E> {
        let data = [
            self.led_a.is_on() as u8,
            self.led_b.is_on() as u8,
            self.led_c.is_on() as u8,
        ];
        self.device.write(&data)?;
        Ok(())
    }

    /// Set all LED states at once and update the hardware.
    pub fn set_leds(&mut self, a: bool, b: bool, c: bool) -> Result<(), E> {
        self.led_a.set(a);
        self.led_b.set(b);
        self.led_c.set(c);
        self.update_leds()
    }

    /// Turn all LEDs off.
    pub fn all_leds_off(&mut self) -> Result<(), E> {
        self.set_leds(false, false, false)
    }

    /// Turn all LEDs on.
    pub fn all_leds_on(&mut self) -> Result<(), E> {
        self.set_leds(true, true, true)
    }

    /// Release the I2C bus.
    pub fn release(self) -> I2C {
        self.device.release()
    }
}
