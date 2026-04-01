# modulino

[![Crates.io](https://img.shields.io/crates/v/modulino.svg)](https://crates.io/crates/modulino)
[![Documentation](https://docs.rs/modulino/badge.svg)](https://docs.rs/modulino)
[![License](https://img.shields.io/crates/l/modulino.svg)](LICENSE-MIT)

A hardware-agnostic, `no_std` Rust driver for Arduino Modulino breakout boards.

## Overview

This crate provides drivers for the Arduino Modulino family of breakout boards, designed to work with any microcontroller that implements the `embedded-hal` 1.0 I2C traits.

## Supported Modules

| Module | Sensor/IC | Implementation | Default Address |
| :--- | :--- | :--- | :--- |
| `Buttons` | Custom MCU | **Internal** | 0x3E |
| `Buzzer` | Custom MCU | **Internal** | 0x1E |
| `Pixels` | APA102 | **Internal** | 0x36 |
| `Distance` | VL53L4CD | **Internal** | 0x29 |
| `Movement` | LSM6DSOX | **Internal** | 0x6A/0x6B |
| `Knob` | Custom MCU | **Internal** | 0x3A/0x3B |
| `Thermo` | HS3003 | **External** (`hs3003`) | 0x44 |
| `Joystick` | Custom MCU | **Internal** | 0x2C |
| `LatchRelay` | Custom MCU | **Internal** | 0x02 |
| `Vibro` | Custom MCU | **Internal** | 0x38 |
| `LedMatrix` | IS31FL3733 | **Internal (Experimental)** | 0x39 |
| `Pressure` | LPS22HB | **Internal (Experimental)** | 0x5C |
| `Light` | LTR-381RGB | **Internal** | 0x53 |

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
modulino = "0.1"
```

### Example: RGB LEDs

```rust
use modulino::{Pixels, Color};

// Create a Pixels instance with your I2C bus
let mut pixels = Pixels::new(i2c)?;

// Set the first LED to red at 50% brightness
pixels.set_color(0, Color::RED, 50)?;

// Set all LEDs to blue
pixels.set_all_color(Color::BLUE, 25)?;

// Apply the changes
pixels.show()?;
```

### Example: Buttons

```rust
use modulino::Buttons;

let mut buttons = Buttons::new(i2c)?;

loop {
    // Read button states
    let state = buttons.read()?;
    
    // Update LEDs to match button states
    buttons.led_a.set(state.a);
    buttons.led_b.set(state.b);
    buttons.led_c.set(state.c);
    buttons.update_leds()?;
}
```

### Example: Buzzer

```rust
use modulino::{Buzzer, Note};

let mut buzzer = Buzzer::new(i2c)?;

// Play a tone at 440 Hz for 500ms
buzzer.tone(440, 500)?;

// Play a musical note
buzzer.play_note(Note::C5, 1000)?;

// Stop the tone
buzzer.no_tone()?;
```

### Example: Temperature & Humidity

```rust
use modulino::Thermo;

let mut thermo = Thermo::new(i2c);

// Read temperature and humidity (requires a delay provider)
let measurement = thermo.read(&mut delay)?;
println!("Temperature: {:.1}°C", measurement.temperature);
println!("Humidity: {:.1}%", measurement.humidity);
```

### Example: Distance Sensor

```rust
use modulino::Distance;

let mut distance = Distance::new(i2c)?;

// Start continuous ranging
distance.start_ranging()?;

// Read distance in millimeters
let mm = distance.read_distance_blocking()?;
println!("Distance: {} mm", mm);
```

### Example: Rotary Encoder

```rust
use modulino::Knob;

let mut knob = Knob::new(i2c)?;

// Set a range for the encoder value
knob.set_range(-100, 100);

loop {
    if knob.update()? {
        println!("Value: {}, Pressed: {}", knob.value(), knob.pressed());
    }
}
```

### Example: Joystick

```rust
use modulino::Joystick;

let mut joystick = Joystick::new(i2c)?;

// Set custom deadzone
joystick.set_deadzone(15);

loop {
    joystick.update()?;
    
    let (x, y) = joystick.position();
    if joystick.button_pressed() {
        println!("Button pressed at ({}, {})", x, y);
    }
}
```

### Example: Vibration Motor

```rust
use modulino::{Vibro, PowerLevel};

let mut vibro = Vibro::new(i2c)?;

// Vibrate at medium power for 500ms
vibro.on(500, PowerLevel::Medium)?;

// Or use a custom power level (0-100)
vibro.on_with_power(1000, 60)?;
```

### Example: Relay Control

```rust
use modulino::LatchRelay;

let mut relay = LatchRelay::new(i2c)?;

// Turn on the relay
relay.on()?;

// Check state
if relay.is_on()? == Some(true) {
    println!("Relay is ON");
}

// Turn off
relay.off()?;
```

### Example: IMU (Accelerometer/Gyroscope)

```rust
use modulino::Movement;

let mut movement = Movement::new(i2c)?;

// Read acceleration (in g)
let accel = movement.acceleration()?;
println!("Accel: x={:.2}g, y={:.2}g, z={:.2}g", accel.x, accel.y, accel.z);

// Read angular velocity (in dps)
let gyro = movement.angular_velocity()?;
println!("Gyro: x={:.2}°/s, y={:.2}°/s, z={:.2}°/s", gyro.x, gyro.y, gyro.z);
```

### Example: LED Matrix

```rust
use modulino::LedMatrix;

let mut matrix = LedMatrix::new(i2c);
matrix.init()?;

// Set a pixel (x: 0-15, y: 0-5) to full brightness
matrix.set_pixel(0, 0, 255)?;

// Update the display
matrix.show()?;
```

### Example: Pressure & Temperature

```rust
use modulino::Pressure;

let mut pressure = Pressure::new(i2c);

// Read atmospheric pressure in hPa
let hpa = pressure.pressure()?;
println!("Pressure: {:.1} hPa", hpa);

// Read ambient temperature
let temp = pressure.temperature()?;
println!("Temperature: {:.1}°C", temp);
```

### Example: Color & Light Sensor

```rust
use modulino::Light;

let mut light = Light::new(i2c);
light.init()?;

// Read all color channels and calculations
let measurement = light.read()?;
println!("Lux: {:.2}", measurement.lux);
println!("Color: {}", measurement.color_name()); // e.g. "VIVID RED"

// Access raw RGB and IR channels
println!("Red: {}, Green: {}, Blue: {}, IR: {}", 
    measurement.red, measurement.green, measurement.blue, measurement.ir);
```

## Features

- `defmt`: Enable `defmt` formatting for error types (useful for embedded debugging)

```toml
[dependencies]
modulino = { version = "0.1", features = ["defmt"] }
```

## Hardware Requirements

All Modulino devices communicate over I2C at 100kHz. They use the Qwiic/STEMMA QT connector standard for easy daisy-chaining.

## Custom I2C Addresses

Some modules can have their address changed. You can specify a custom address when creating a driver:

```rust
use modulino::{Buttons, addresses};

// Use default address
let buttons = Buttons::new(i2c)?;

// Use custom address
let buttons = Buttons::new_with_address(i2c, 0x40)?;
```

## Minimum Supported Rust Version (MSRV)

This crate requires Rust 1.75 or later.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This library is inspired by the [Arduino Modulino MicroPython library](https://github.com/arduino/arduino-modulino-mpy)
and [Arduino Modulino C++ library](https://github.com/arduino-libraries/Arduino_Modulino)
