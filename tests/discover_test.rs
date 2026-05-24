use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::{Buttons, Knob, OptoRelay};

#[test]
fn test_discover_buttons() {
    let addr = 0x3E;
    let expectations = [
        // discover() does a 0-byte write to test if it ACKs
        I2cTransaction::write(addr, vec![]),
    ];

    let mut i2c = I2cMock::new(&expectations);
    let detected_addr = Buttons::discover(&mut i2c).unwrap();
    assert_eq!(detected_addr, addr);
    i2c.done();
}

#[test]
fn test_discover_knob_first_addr() {
    let addr = 0x3A;
    let expectations = [
        // discover() tries first address in addresses::KNOB [0x3A, 0x3B]
        I2cTransaction::write(addr, vec![]),
    ];

    let mut i2c = I2cMock::new(&expectations);
    let detected_addr = Knob::discover(&mut i2c).unwrap();
    assert_eq!(detected_addr, addr);
    i2c.done();
}

#[test]
fn test_discover_knob_second_addr() {
    let expectations = [
        // First address 0x3A fails to ACK
        I2cTransaction::write(0x3A, vec![]).with_error(
            embedded_hal::i2c::ErrorKind::NoAcknowledge(
                embedded_hal::i2c::NoAcknowledgeSource::Address,
            ),
        ),
        // Second address 0x3B ACKs
        I2cTransaction::write(0x3B, vec![]),
    ];

    let mut i2c = I2cMock::new(&expectations);
    let detected_addr = Knob::discover(&mut i2c).unwrap();
    assert_eq!(detected_addr, 0x3B);
    i2c.done();
}

#[test]
fn test_discover_fails() {
    let addr = 0x14; // OptoRelay default
    let expectations = [
        // Probes 0x14, fails
        I2cTransaction::write(addr, vec![]).with_error(
            embedded_hal::i2c::ErrorKind::NoAcknowledge(
                embedded_hal::i2c::NoAcknowledgeSource::Address,
            ),
        ),
        // Final call to generate the typed error
        I2cTransaction::write(addr, vec![]).with_error(
            embedded_hal::i2c::ErrorKind::NoAcknowledge(
                embedded_hal::i2c::NoAcknowledgeSource::Address,
            ),
        ),
    ];

    let mut i2c = I2cMock::new(&expectations);
    assert!(OptoRelay::discover(&mut i2c).is_err());
    i2c.done();
}
