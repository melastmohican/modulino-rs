use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::{DisplayMode, LedMatrix};

#[test]
fn test_led_matrix_coprocessor_monochromatic() {
    let addr = 0x39;

    let expectations = [
        // 1. init() calls set_mode(MonochromaticVertical)
        // 1a. Queries current mode of the device:
        // Simulate: Pinstrap=0x39, device is in MON mode ("MON")
        I2cTransaction::read(addr, vec![0x39, b'M', b'O', b'N']),
        // 1b. Writes "MON" padded to 12 bytes because device is currently in MON
        I2cTransaction::write(addr, vec![b'M', b'O', b'N', 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        // 2. set_pixel(0, 0, 255) -> sets bit 0 of column 0
        // 3. show() writes the 12-byte buffer
        I2cTransaction::write(addr, vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut matrix = LedMatrix::new(i2c);

    matrix.init().unwrap();
    assert_eq!(matrix.mode(), DisplayMode::MonochromaticVertical);

    matrix.set_pixel(0, 0, 255).unwrap();
    matrix.show().unwrap();

    matrix.release().done();
}

#[test]
fn test_led_matrix_coprocessor_mode_switch() {
    let addr = 0x39;

    let expectations = [
        // 1. init() -> queries device (returns MON) -> writes MON (12B)
        I2cTransaction::read(addr, vec![0x39, b'M', b'O', b'N']),
        I2cTransaction::write(addr, vec![b'M', b'O', b'N', 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        // 2. set_mode(Grayscale)
        // 2a. Queries current mode of device: returns MON
        I2cTransaction::read(addr, vec![0x39, b'M', b'O', b'N']),
        // 2b. Device is MON: write new mode "GS4" padded to 12 bytes
        I2cTransaction::write(addr, vec![b'G', b'S', b'4', 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        // 3. set_pixel(0, 0, 255) in grayscale (brightness 255 / 17 = 15)
        // 4. show() writes 48-byte grayscale payload (column 0, row 0 is lower nibble of byte 0)
        I2cTransaction::write(addr, {
            let mut expected = vec![0u8; 48];
            expected[0] = 15; // lower 4 bits of byte 0
            expected
        }),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut matrix = LedMatrix::new(i2c);

    matrix.init().unwrap();
    matrix.set_mode(DisplayMode::Grayscale).unwrap();
    assert_eq!(matrix.mode(), DisplayMode::Grayscale);

    matrix.set_pixel(0, 0, 255).unwrap();
    matrix.show().unwrap();

    matrix.release().done();
}

#[test]
fn test_led_matrix_coprocessor_horizontal() {
    let addr = 0x39;

    let expectations = [
        // 1. init() -> queries device (returns MON) -> writes MON (12B)
        I2cTransaction::read(addr, vec![0x39, b'M', b'O', b'N']),
        I2cTransaction::write(addr, vec![b'M', b'O', b'N', 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        // 2. set_mode(MonochromaticHorizontal)
        // 2a. Queries current mode of device: returns MON
        I2cTransaction::read(addr, vec![0x39, b'M', b'O', b'N']),
        // 2b. Device is MON: write new mode "MON" padded to 12 bytes
        I2cTransaction::write(addr, vec![b'M', b'O', b'N', 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        // 3. set_pixel(0, 0, 255) in row-major horizontal:
        // bit index = 0 * 12 + 0 = 0.
        // Byte 0 bit offset 7.
        // So byte 0 becomes 0x80.
        // On show(), it is converted to column-major (vertical).
        // Row 0 of Column 0 corresponds to bit 0 of Column 0 byte.
        // So Column 0 byte (byte 0) should be 1.
        I2cTransaction::write(addr, vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut matrix = LedMatrix::new(i2c);

    matrix.init().unwrap();
    matrix
        .set_mode(DisplayMode::MonochromaticHorizontal)
        .unwrap();

    matrix.set_pixel(0, 0, 255).unwrap();
    matrix.show().unwrap();

    matrix.release().done();
}
