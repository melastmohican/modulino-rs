use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::LedMatrix;

#[test]
fn test_led_matrix_init_and_set_pixel() {
    let addr = 0x39;

    // Create zeroed buffers for clear() expectations
    let mut off_all = vec![0u8; 25];
    off_all[0] = 0x00; // Starting register offset

    let mut pwm_all = vec![0u8; 193];
    pwm_all[0] = 0x00; // Starting register offset

    let expectations = [
        // init() -> unlock
        I2cTransaction::write(addr, vec![0xFE, 0xC5]),
        // init() -> Reset (Page 3, Read 0x11)
        I2cTransaction::write(addr, vec![0xFD, 0x03]),
        I2cTransaction::write_read(addr, vec![0x11], vec![0x00]),
        // init() -> Awake (Reg 0x00 = 0x01)
        I2cTransaction::write(addr, vec![0x00, 0x01]),
        // init() -> GCC (Reg 0x01 = 0x40)
        I2cTransaction::write(addr, vec![0x01, 0x40]),
        // clear() -> Page 0 (LED Control)
        I2cTransaction::write(addr, vec![0xFD, 0x00]),
        I2cTransaction::write(addr, off_all),
        // clear() -> Page 1 (PWM)
        I2cTransaction::write(addr, vec![0xFD, 0x01]),
        I2cTransaction::write(addr, pwm_all),
        // set_pixel(0, 0, 255)
        // 1. Enable (Page 0, Reg 0 = |= 1)
        I2cTransaction::write(addr, vec![0xFD, 0x00]),
        I2cTransaction::write_read(addr, vec![0x00], vec![0x00]),
        I2cTransaction::write(addr, vec![0x00, 0x01]),
        // 2. PWM (Page 1, Reg 0 = 255)
        I2cTransaction::write(addr, vec![0xFD, 0x01]),
        I2cTransaction::write(addr, vec![0x00, 0xFF]),
    ];

    let mut matrix = LedMatrix::new(I2cMock::new(&expectations));
    matrix.init().unwrap();
    matrix.set_pixel(0, 0, 255).unwrap();

    matrix.release().done();
}
