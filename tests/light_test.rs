use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Light;

#[test]
fn test_light_init_and_read() {
    let addr = 0x53;
    let expectations = [
        // 1. init() -> Verify Part ID (0x06)
        I2cTransaction::write_read(addr, vec![0x06], vec![0xC2]), // 0xC0 expected in high nibble
        // 2. init() -> Clear status (0x07)
        I2cTransaction::write_read(addr, vec![0x07], vec![0x00]),
        // 3. init() -> Set gain (0x05) to Gain18x (0x04)
        I2cTransaction::write(addr, vec![0x05, 0x04]),
        // 4. init() -> Set config (0x04) to Res16Bit (0x04) and Rate25ms (0x00) -> 0x40
        I2cTransaction::write(addr, vec![0x04, 0x40]),
        // 5. init() -> Enable (0x00) with RGB mode (0x06)
        I2cTransaction::write(addr, vec![0x00, 0x06]),
        // read() sequence
        // IR (0x0A)
        I2cTransaction::write_read(addr, vec![0x0A], vec![0x01, 0x02, 0x03]),
        // Green (0x0D)
        I2cTransaction::write_read(addr, vec![0x0D], vec![0x04, 0x05, 0x06]),
        // Red (0x10)
        I2cTransaction::write_read(addr, vec![0x10], vec![0x07, 0x08, 0x09]),
        // Blue (0x13)
        I2cTransaction::write_read(addr, vec![0x13], vec![0x0A, 0x0B, 0x0C]),
    ];

    let mut light = Light::new(I2cMock::new(&expectations));
    light.init().unwrap();

    let data = light.read().unwrap();

    // Value = buf[0] | buf[1]<<8 | buf[2]<<16
    assert_eq!(data.ir, 0x030201);
    assert_eq!(data.green, 0x060504);
    assert_eq!(data.red, 0x090807);
    assert_eq!(data.blue, 0x0C0B0A);

    light.release().done();
}
