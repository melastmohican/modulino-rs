# Modulino Examples

Since this is a `no_std` library, examples need to target specific hardware platforms.

## Running Examples

To run these examples, you'll need to:

1. Choose a target microcontroller platform (e.g., STM32, ESP32, RP2040)
2. Set up the appropriate HAL crate for your platform
3. Configure the I2C peripheral

## Example Hardware Setups

### With Embassy (async) on RP2040

```rust
use embassy_rp::i2c::{I2c, Config};
use modulino::{Pixels, Color};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    
    let i2c = I2c::new_blocking(p.I2C0, p.PIN_1, p.PIN_0, Config::default());
    
    let mut pixels = Pixels::new(i2c).unwrap();
    pixels.set_all_color(Color::RED, 50);
    pixels.show().unwrap();
}
```

### With ESP-HAL on ESP32

```rust
use esp_hal::{i2c::I2C, prelude::*};
use modulino::{Buzzer, Note};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        100.kHz(),
        &clocks,
    );
    
    let mut buzzer = Buzzer::new(i2c).unwrap();
    buzzer.play_note(Note::C5, 500).unwrap();
    
    loop {}
}
```

### With STM32 HAL

```rust
use stm32f4xx_hal::{i2c::I2c, prelude::*};
use modulino::{Buttons, ButtonState};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    
    let gpiob = dp.GPIOB.split();
    let scl = gpiob.pb6.into_alternate_open_drain();
    let sda = gpiob.pb7.into_alternate_open_drain();
    
    let i2c = I2c::new(dp.I2C1, (scl, sda), 100.kHz(), &clocks);
    
    let mut buttons = Buttons::new(i2c).unwrap();
    
    loop {
        let state = buttons.read().unwrap();
        buttons.set_leds(state.a, state.b, state.c).unwrap();
    }
}
```
