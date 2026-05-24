use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Hub;

#[test]
fn test_hub_select_and_clear() {
    let addr = 0x70; // Default TCA9548A multiplexer address

    let expectations = [
        // 1. select(port 0) -> writes 1 << 0 = 0x01
        I2cTransaction::write(addr, vec![0x01]),
        // 2. select(port 5) -> writes 1 << 5 = 0x20
        I2cTransaction::write(addr, vec![0x20]),
        // 3. clear() -> writes 0x00
        I2cTransaction::write(addr, vec![0x00]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut hub = Hub::new(i2c);

    hub.select(0).unwrap();
    hub.select(5).unwrap();
    hub.clear().unwrap();

    hub.release().done();
}
