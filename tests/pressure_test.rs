use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::Pressure;

#[test]
fn test_pressure_init_and_read() {
    let addr = 0x5C;
    let expectations = [
        // init() -> WHO_AM_I (0x0F)
        I2cTransaction::write_read(addr, vec![0x0F], vec![0xB1]),
        // init() -> CTRL_REG1 (0x10) = 0x22 (10Hz, BDU)
        I2cTransaction::write(addr, vec![0x10, 0x22]),
        // pressure() -> OUT_P_XL (0x28)
        I2cTransaction::write_read(addr, vec![0x28], vec![0x00, 0x00, 0x01]), // raw = 0x010000 = 65536
        // temperature() -> OUT_T_L (0x2B)
        I2cTransaction::write_read(addr, vec![0x2B], vec![0xE8, 0x03]), // raw = 0x03E8 = 1000
    ];

    let mut sensor = Pressure::new(I2cMock::new(&expectations));
    sensor.init().unwrap();

    // 65536 / 4096.0 = 16.0 hPa (dummy value)
    assert_eq!(sensor.pressure().unwrap(), 16.0);

    // 1000 / 100.0 = 10.0 °C
    assert_eq!(sensor.temperature().unwrap(), 10.0);

    sensor.release().done();
}
