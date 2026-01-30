use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Joystick;

#[test]
fn test_joystick_update_and_read() {
    let addr = 0x2C;

    let expectations = [
        // 1. Joystick::new() calls update()
        // Simulate: Pinstrap=0x58, X=128 (center), Y=128 (center), Button=0
        I2cTransaction::read(addr, vec![0x58, 128, 128, 0]),
        // 2. joystick.update()
        // Simulate: Pinstrap=0x58, X=255 (max right), Y=0 (max down), Button=1 (pressed)
        I2cTransaction::read(addr, vec![0x58, 255, 0, 1]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut joystick = Joystick::new(i2c).unwrap();

    // Check initial state (centered)
    assert_eq!(joystick.x(), 0);
    assert_eq!(joystick.y(), 0);
    assert!(!joystick.button_pressed());

    // Update and check new state
    assert!(joystick.update().unwrap());

    // 255 -> 127 (Max positive)
    // 0 -> -128 (Max negative)
    assert_eq!(joystick.x(), 127);
    assert_eq!(joystick.y(), -128);
    assert!(joystick.button_pressed());

    joystick.release().done();
}

#[test]
fn test_joystick_deadzone() {
    let addr = 0x2C;

    let expectations = [
        // 1. new()
        I2cTransaction::read(addr, vec![0x58, 128, 128, 0]),
        // 2. update() - slightly off center (135), within default deadzone (10)
        // 135 - 128 = 7. Deadzone is 10. Should report 0.
        I2cTransaction::read(addr, vec![0x58, 135, 120, 0]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut joystick = Joystick::new(i2c).unwrap();

    joystick.update().unwrap();

    assert_eq!(joystick.x(), 0); // Should be 0 due to deadzone
    assert_eq!(joystick.y(), 0); // -8 is also within deadzone 10

    joystick.release().done();
}
