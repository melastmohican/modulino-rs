use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::LatchRelay;

#[test]
fn test_latch_relay_control() {
    let addr = 0x02;

    let expectations = [
        // 1. on()
        I2cTransaction::write(addr, vec![0x01, 0x00, 0x00]),
        // 2. off()
        I2cTransaction::write(addr, vec![0x00, 0x00, 0x00]),
        // 3. is_on() -> true
        // Read 4 bytes: [Pinstrap, Status0, Status1, ...]
        // Status0=0, Status1=1 => ON
        I2cTransaction::read(addr, vec![0x04, 0x00, 0x01, 0x00]),
        // 4. is_on() -> false
        // Status0=1, Status1=0 => OFF
        I2cTransaction::read(addr, vec![0x04, 0x01, 0x00, 0x00]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut relay = LatchRelay::new(i2c).unwrap();

    relay.on().unwrap();
    relay.off().unwrap();

    assert_eq!(relay.is_on().unwrap(), Some(true));
    assert_eq!(relay.is_on().unwrap(), Some(false));

    relay.release().done();
}
