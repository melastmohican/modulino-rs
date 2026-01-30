use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::{PowerLevel, Vibro};

#[test]
fn test_vibro_control() {
    let addr = 0x38;

    let expectations = [
        // 1. new() calls off()
        I2cTransaction::write(addr, vec![0u8; 12]),
        // 2. on(500ms, Medium)
        // Freq: 1000Hz (Default) = 0x000003E8 -> [E8, 03, 00, 00]
        // Duration: 500ms = 0x000001F4 -> [F4, 01, 00, 00]
        // Power: Medium = 45 = 0x2D -> [2D, 00, 00, 00]
        I2cTransaction::write(
            addr,
            vec![
                0xE8, 0x03, 0x00, 0x00, 0xF4, 0x01, 0x00, 0x00, 0x2D, 0x00, 0x00, 0x00,
            ],
        ),
        // 3. off()
        I2cTransaction::write(addr, vec![0u8; 12]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut vibro = Vibro::new(i2c).unwrap();

    vibro.on(500, PowerLevel::Medium).unwrap();
    vibro.off().unwrap();

    vibro.release().done();
}
