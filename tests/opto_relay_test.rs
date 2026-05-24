use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::OptoRelay;

#[test]
fn test_opto_relay() {
    let addr = 0x14; // Default address 0x28 >> 1 = 0x14

    let expectations = [
        // 1. new() calls update()
        // Simulate: Pinstrap=0x28, status=0
        I2cTransaction::read(addr, vec![0x28, 0, 0, 0]),
        // 2. relay.on()
        I2cTransaction::write(addr, vec![1, 0, 0]),
        // 3. relay.off()
        I2cTransaction::write(addr, vec![0, 0, 0]),
        // 4. relay.update()
        // Simulate status is ON (1)
        I2cTransaction::read(addr, vec![0x28, 1, 0, 0]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut relay = OptoRelay::new(i2c).unwrap();

    assert!(!relay.is_on());

    relay.on().unwrap();
    assert!(relay.is_on());

    relay.off().unwrap();
    assert!(!relay.is_on());

    assert!(relay.update().unwrap());
    assert!(relay.is_on());

    relay.release().done();
}
