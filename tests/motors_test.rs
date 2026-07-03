use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use modulino::{DecayMode, Motors};

#[test]
fn test_motors_initialization_and_telemetry() {
    let addr = 0x24;

    let expectations = [
        // 1. Motors::new_with_address calls update() on construction
        // Simulate telemetry read: Pinstrap=0x48, raw_a=1000, raw_b=2000, flags=0x0F (busy, stepper, half-step, HFS)
        I2cTransaction::read(addr, vec![0x48, 0xE8, 0x03, 0xD0, 0x07, 0x0F]),
    ];

    let i2c = I2cMock::new(&expectations);
    let motors = Motors::new_with_address(i2c, addr).unwrap();

    assert_eq!(motors.sensed_raw_a(), 1000);
    assert_eq!(motors.sensed_raw_b(), 2000);
    assert!(motors.busy());
    assert!(motors.stepper_mode_enabled());
    assert!(motors.half_step_enabled());
    assert!(motors.half_full_scale_enabled());
    assert_eq!(motors.decay_mode(), DecayMode::Slow);

    // Scaling verification:
    // raw = 1000, hfs = true (kisen = 3750)
    // mA = (1000 * 3300 * 3750) / (4095 * 4700) = 12375000000 / 19246500 = 642.9738...
    let current_a = motors.sensed_current_a();
    assert!(current_a > 642.0 && current_a < 644.0);

    motors.release().done();
}

#[test]
fn test_motors_commands() {
    let addr = 0x24;

    let expectations = [
        // 1. constructor read
        I2cTransaction::read(addr, vec![0x48, 0, 0, 0, 0, 0]),
        // 2. set_stepper_mode_enabled(true) -> CMD_MODE ('M'), 1
        I2cTransaction::write(addr, vec![b'M', 1, 0, 0, 0, 0, 0, 0]),
        // 3. set_dc_speed_raw(1000, -1000) -> CMD_SPEED_DC ('S')
        I2cTransaction::write(addr, vec![b'S', 0xE8, 0x03, 0x18, 0xFC, 0, 0, 0]),
        // 4. move_stepper(20000, 50, 10) -> CMD_STEPPER ('G')
        // steps: 20000 = 0x00004E20
        // speed_period: 50 = 0x0032
        // release_delay: 10
        I2cTransaction::write(addr, vec![b'G', 0x20, 0x4E, 0x00, 0x00, 0x32, 0x00, 0x0A]),
        // 5. set_decay(DecayMode::Fast) -> CMD_DECAY ('T'), 3
        I2cTransaction::write(addr, vec![b'T', 3, 0, 0, 0, 0, 0, 0]),
    ];

    let i2c = I2cMock::new(&expectations);
    let mut motors = Motors::new_with_address(i2c, addr).unwrap();

    motors.set_stepper_mode_enabled(true).unwrap();
    motors.set_dc_speed_raw(1000, -1000).unwrap();
    motors.move_stepper(20000, 50, 10).unwrap();
    motors.set_decay(DecayMode::Fast).unwrap();

    motors.release().done();
}
