use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Buttons;

#[test]
fn test_buttons_and_leds() {
    let addr = 0x3E;
    let expectations = [
        I2cTransaction::read(addr, vec![0x7C, 0x00, 0x01, 0x00]),
        I2cTransaction::write(addr, vec![0x00, 0x01, 0x00]),
    ];
    let mut buttons = Buttons::new(I2cMock::new(&expectations)).unwrap();
    assert!(buttons.button_b_pressed());
    buttons.set_leds(false, true, false).unwrap();
    buttons.release().done();
}
