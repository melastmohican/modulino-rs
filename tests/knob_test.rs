use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Knob;

#[test]
fn test_knob_bug_on_set_normal() {
    let addr = 0x3A;
    let expectations = [
        // 1. Initial read inside new_with_address()
        // Simulate: Pinstrap=0x76, encoder=42, button=0
        I2cTransaction::read(addr, vec![0x76, 42, 0, 0]),
        // 2. set_value_internal(100)
        I2cTransaction::write(addr, vec![100, 0, 0, 0]),
        // 3. read_data() to test if value matches 100
        I2cTransaction::read(addr, vec![0x76, 100, 0, 0]),
        // 4. set_value_internal(initial_val = 42)
        I2cTransaction::write(addr, vec![42, 0, 0, 0]),
        // 5. update() in new_with_address()
        I2cTransaction::read(addr, vec![0x76, 42, 0, 0]),
    ];

    let knob = Knob::new(I2cMock::new(&expectations)).unwrap();
    assert_eq!(knob.value(), 42);
    assert!(!knob.pressed());
    knob.release().done();
}

#[test]
fn test_knob_bug_on_set_buggy() {
    let addr = 0x3A;
    let expectations = [
        // 1. Initial read inside new_with_address()
        // Simulate: Pinstrap=0x76, encoder=42, button=0
        I2cTransaction::read(addr, vec![0x76, 42, 0, 0]),
        // 2. set_value_internal(100) (not buggy value yet)
        I2cTransaction::write(addr, vec![100, 0, 0, 0]),
        // 3. read_data() returns e.g. -100 instead of 100 (bug detected)
        I2cTransaction::read(
            addr,
            vec![
                0x76,
                (-100i16).to_le_bytes()[0],
                (-100i16).to_le_bytes()[1],
                0,
            ],
        ),
        // 4. set_value_internal(-initial_val = -42) which gets negated to +42 due to buggy check
        I2cTransaction::write(addr, vec![42, 0, 0, 0]),
        // 5. update() in new_with_address()
        I2cTransaction::read(addr, vec![0x76, 42, 0, 0]),
        // 6. set_value(50) (should be negated to -50)
        I2cTransaction::write(
            addr,
            vec![(-50i16).to_le_bytes()[0], (-50i16).to_le_bytes()[1], 0, 0],
        ),
    ];

    let mut knob = Knob::new(I2cMock::new(&expectations)).unwrap();
    assert_eq!(knob.value(), 42);

    // Set value and verify negation on buggy firmware
    knob.set_value(50).unwrap();

    knob.release().done();
}

#[test]
fn test_knob_debounced_direction() {
    let addr = 0x3A;
    let expectations = [
        // 1. new() sequence (same as normal)
        I2cTransaction::read(addr, vec![0x76, 42, 0, 0]),
        I2cTransaction::write(addr, vec![100, 0, 0, 0]),
        I2cTransaction::read(addr, vec![0x76, 100, 0, 0]),
        I2cTransaction::write(addr, vec![42, 0, 0, 0]),
        I2cTransaction::read(addr, vec![0x76, 42, 0, 0]),
        // 2. first direction read (now = 10ms, < 30ms -> immediate return 0 without I2C write/read)

        // 3. second direction read (now = 35ms, > 30ms -> I2C update)
        // Value increased to 45
        I2cTransaction::read(addr, vec![0x76, 45, 0, 0]),
    ];

    let mut knob = Knob::new(I2cMock::new(&expectations)).unwrap();

    // 10ms is too quick: should return 0
    assert_eq!(knob.direction(10).unwrap(), 0);
    assert_eq!(knob.value(), 42);

    // 35ms is enough time: should perform update, see value 45, return 1 (clockwise)
    assert_eq!(knob.direction(35).unwrap(), 1);
    assert_eq!(knob.value(), 45);

    knob.release().done();
}
