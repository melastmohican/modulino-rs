//! Concrete example showing how to multiplex two identical `Buttons` modules using a
//! TCA9548A I2C multiplexer (such as the SparkFun Qwiic Mux Breakout:
//! https://www.sparkfun.com/sparkfun-qwiic-mux-breakout-8-channel-tca9548a.html).
//!
//! Because both `Buttons` modules share the same default hardware address (`0x3E`),
//! connecting them directly to the same I2C bus would cause address conflicts.
//! Placing them on separate ports of the multiplexer allows us to select one port,
//! communicate with that device, then select the other port to communicate with the second device.

use core::cell::RefCell;
use modulino::{Buttons, Hub};

/// A simple helper that implements `I2c` for a shared `RefCell` reference.
///
/// This allows multiple drivers to share access to the same underlying physical I2C bus.
struct SharedI2c<'a>(&'a RefCell<DummyI2c>);

impl<'a> embedded_hal::i2c::ErrorType for SharedI2c<'a> {
    type Error = DummyError;
}

impl<'a> embedded_hal::i2c::I2c for SharedI2c<'a> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.0.borrow_mut().read(address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.0.borrow_mut().write(address, write)
    }

    fn write_read(
        &mut self,
        _address: u8,
        _write: &[u8],
        _read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0.borrow_mut().write_read(_address, _write, _read)
    }

    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.0.borrow_mut().transaction(_address, _operations)
    }
}

fn main() {
    // 1. Initialize your microcontroller's clock and I2C peripherals.
    // For this example, we use a dummy driver for local compilation.
    let i2c = dummy_i2c_initialization();

    // 2. Wrap the I2C bus in a RefCell to allow sharing it between the Hub and Modules.
    let shared_i2c = RefCell::new(i2c);

    // 3. Create the Hub driver managing the TCA9548A multiplexer at default address 0x70.
    let mut hub = Hub::new(SharedI2c(&shared_i2c));

    // 4. Create the two Buttons instances.
    // Even though they share the same physical address (`0x3E`), they are isolated
    // behind the Hub's ports 0 and 1. We must select the appropriate port before
    // initializing each button module.

    // Select port 0 on the multiplexer and initialize Buttons A
    hub.select(0).unwrap();
    let mut buttons_a = Buttons::new(SharedI2c(&shared_i2c)).unwrap();

    // Select port 1 on the multiplexer and initialize Buttons B
    hub.select(1).unwrap();
    let mut buttons_b = Buttons::new(SharedI2c(&shared_i2c)).unwrap();

    loop {
        // --- Read Buttons A (on multiplexer Port 0) ---
        hub.select(0).unwrap();
        if let Ok(state) = buttons_a.read() {
            // Toggle corresponding LED based on button A state
            buttons_a.led_a.set(state.a);
            buttons_a.update_leds().unwrap();
        }

        // --- Read Buttons B (on multiplexer Port 1) ---
        hub.select(1).unwrap();
        if let Ok(state) = buttons_b.read() {
            // Toggle corresponding LED based on button A state
            buttons_b.led_a.set(state.a);
            buttons_b.update_leds().unwrap();
        }

        // Add a brief delay before the next polling cycle
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

// Mock/Dummy implementation to satisfy compiling without a real hardware HAL:
fn dummy_i2c_initialization() -> DummyI2c {
    DummyI2c
}

struct DummyI2c;

impl embedded_hal::i2c::ErrorType for DummyI2c {
    type Error = DummyError;
}

#[derive(Debug, Clone, Copy)]
struct DummyError;

impl embedded_hal::i2c::Error for DummyError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

impl embedded_hal::i2c::I2c for DummyI2c {
    fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, _address: u8, _write: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_read(
        &mut self,
        _address: u8,
        _write: &[u8],
        _read: &mut [u8],
    ) -> Result<(), Self::Error> {
        // Return matching pinstraps to satisfy initialization checks
        if _address == 0x3E && !_read.is_empty() {
            _read[0] = 0x7C; // Buttons pinstrap
        }
        Ok(())
    }

    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
