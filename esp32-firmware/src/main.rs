//! mccm status-LED firmware for the MuseLab nanoESP32-C6.
//!
//! Listens on the chip's native USB-Serial-JTAG port (the USB-C connector
//! labeled "ESP32C6") for single-byte status commands from `mccm led`:
//!
//!   'R' = needs help  -> red LED    (GPIO18)
//!   'Y' = inactive    -> yellow LED (GPIO19; legacy 'B' also accepted)
//!   'G' = active      -> green LED  (GPIO20)
//!   'N' = no sessions -> all off
//!
//! Any other byte (including the trailing '\n') is ignored. All three
//! LEDs lit means "no host daemon": that's the state after boot and the
//! state we fall back to when the heartbeat (every 2 s) goes missing for
//! IDLE_TIMEOUT_MS, so a dead daemon can't leave a stale status showing.

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
    usb_serial_jtag::UsbSerialJtag,
};

esp_bootloader_esp_idf::esp_app_desc!();

const IDLE_TIMEOUT_MS: u32 = 10_000;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // GPIO18/19/20: adjacent on the bottom header, non-strapping, and clear
    // of the native USB pins (12/13) and the CH343 UART pins (16/17).
    let mut red = Output::new(peripherals.GPIO18, Level::Low, OutputConfig::default());
    let mut yellow = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let mut green = Output::new(peripherals.GPIO20, Level::Low, OutputConfig::default());

    // Wiring self-test: flash all three LEDs together, three times,
    // 2 s per flash...
    for _ in 0..3 {
        for led in [&mut red, &mut yellow, &mut green] {
            led.set_high();
        }
        delay.delay_millis(2_000);
        for led in [&mut red, &mut yellow, &mut green] {
            led.set_low();
        }
        delay.delay_millis(500);
    }
    // ...then hold all three on until the host daemon speaks.
    red.set_high();
    yellow.set_high();
    green.set_high();

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    // Deliberately no writes back to the host: a blocking TX write stalls
    // forever if nothing on the USB side is reading, and `mccm led` only
    // sends. RX is safe — read_byte() never blocks.
    let mut idle_ms: u32 = 0;
    loop {
        match usb_serial.read_byte() {
            Ok(byte) => {
                let lit = match byte {
                    b'R' => Some((true, false, false)),
                    b'Y' | b'B' => Some((false, true, false)),
                    b'G' => Some((false, false, true)),
                    b'N' => Some((false, false, false)),
                    _ => None,
                };
                if let Some((r, y, g)) = lit {
                    set(&mut red, r);
                    set(&mut yellow, y);
                    set(&mut green, g);
                    idle_ms = 0;
                }
            }
            // Error type is Infallible, so this is only ever WouldBlock.
            Err(_) => {
                delay.delay_millis(1);
                idle_ms = idle_ms.saturating_add(1);
                if idle_ms == IDLE_TIMEOUT_MS {
                    red.set_high();
                    yellow.set_high();
                    green.set_high();
                }
            }
        }
    }
}

fn set(led: &mut Output<'_>, on: bool) {
    if on {
        led.set_high();
    } else {
        led.set_low();
    }
}
