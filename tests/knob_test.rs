use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Knob;

#[test]
fn test_knob_functionality() {
    let addr = 0x3A;
    let expectations = [
        I2cTransaction::read(addr, vec![0x76, 0x64, 0x00, 0x00]), // Initial read in new()
        I2cTransaction::read(addr, vec![0x76, 0x69, 0x00, 0x01]), // update()
    ];
    let mut knob = Knob::new(I2cMock::new(&expectations)).unwrap();
    assert_eq!(knob.value(), 100);
    assert!(knob.update().unwrap());
    assert!(knob.pressed());
    knob.release().done();
}
