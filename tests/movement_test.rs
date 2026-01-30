use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Movement;

#[test]
fn test_movement_imu() {
    let addr = 0x6A;
    let expectations = [
        I2cTransaction::write_read(addr, vec![0x0F], vec![0x6C]),
        I2cTransaction::write(addr, vec![0x12, 0x01]),
        I2cTransaction::write(addr, vec![0x10, 0x40]),
        I2cTransaction::write(addr, vec![0x11, 0x40]),
        I2cTransaction::write(addr, vec![0x12, 0x44]),
        I2cTransaction::write_read(addr, vec![0x28], vec![0x00, 0x00, 0x00, 0x00, 0x09, 0x40]),
    ];
    let mut movement = Movement::new(I2cMock::new(&expectations)).unwrap();
    let accel = movement.acceleration().unwrap();
    assert!((accel.z - 1.0).abs() < 0.01);
    movement.release().done();
}
