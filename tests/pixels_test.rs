use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::{Color, Pixels};

#[test]
fn test_pixels_formatting() {
    let addr = 0x36;
    let mut expected_data: Vec<u8> = Vec::new();
    expected_data.extend_from_slice(&[0xEF, 0x00, 0x00, 0xFF]); // Red
    for _ in 1..8 {
        expected_data.extend_from_slice(&[0xE0, 0x00, 0x00, 0x00]);
    }

    let expectations = [I2cTransaction::write(addr, expected_data)];
    let mut pixels = Pixels::new(I2cMock::new(&expectations)).unwrap();
    pixels.set_color(0, Color::RED, 50).unwrap();
    pixels.show().unwrap();
    pixels.release().done();
}
