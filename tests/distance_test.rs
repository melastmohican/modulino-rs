use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Distance;

#[test]
fn test_distance_logic() {
    let addr = 0x29;
    let expectations = [
        // We skip init() in this test because it requires mocking hundreds of writes.
        // We assume the device is already initialized and test the read sequence.

        // read_distance()
        // 1. Read STATUS (0x0089)
        I2cTransaction::write_read(addr, vec![0x00, 0x89], vec![0x04]),
        // 2. Read DISTANCE (0x0096)
        I2cTransaction::write_read(addr, vec![0x00, 0x96], vec![0x01, 0xF4]), // 500mm
        // 3. clear_interrupt() -> Write SYSTEM_INTERRUPT_CLEAR (0x0086) = 0x01
        I2cTransaction::write(addr, vec![0x00, 0x86, 0x01]),
    ];

    // Distance::new now returns Self, not Result, so no unwrap()
    let mut distance = Distance::new(I2cMock::new(&expectations));

    assert_eq!(distance.read_distance().unwrap(), Some(500));

    distance.release().done();
}
