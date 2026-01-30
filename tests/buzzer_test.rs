use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Buzzer;

#[test]
fn test_buzzer_tone_generation() {
    let addr = 0x1E;
    let expectations = [
        I2cTransaction::write(addr, vec![0x00; 8]),
        I2cTransaction::write(addr, vec![0xB8, 0x01, 0x00, 0x00, 0xF4, 0x01, 0x00, 0x00]),
    ];
    let mut buzzer = Buzzer::new(I2cMock::new(&expectations)).unwrap();
    buzzer.tone(440, 500).unwrap();
    buzzer.release().done();
}
