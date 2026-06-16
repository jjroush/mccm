//! mccm status-LED firmware for the ESP32-C6.
//!
//! Listens on the chip's native USB-Serial-JTAG port for single-byte
//! status commands from `mccm led`:
//!
//!   'R' = needs help  -> red LED
//!   'Y' = inactive    -> yellow LED (legacy 'B' also accepted)
//!   'G' = active      -> green LED
//!   'N' = no sessions -> all off
//!
//! Any other byte (including the trailing '\n') is ignored. All three
//! LEDs lit means "no host daemon": that's the state after boot and the
//! state we fall back to when the heartbeat (every 2 s) goes missing for
//! IDLE_TIMEOUT_MS, so a dead daemon can't leave a stale status showing.
//!
//! The LED GPIOs are board-selected at compile time (see the `board-*`
//! cargo features):
//!   board-nano / board-devkitc  -> GPIO18 / 19 / 20  (default)
//!   board-xiao                  -> GPIO0 / 1 / 2  (XIAO pads D0 / D1 / D2)

#![no_std]
#![no_main]

// Exactly one board feature must be active.
#[cfg(not(any(
    feature = "board-nano",
    feature = "board-devkitc",
    feature = "board-xiao"
)))]
compile_error!(
    "Select a board: default builds for board-nano; for XIAO use \
     `--no-default-features --features board-xiao`."
);
#[cfg(all(
    feature = "board-xiao",
    any(feature = "board-nano", feature = "board-devkitc")
))]
compile_error!(
    "Enable only one board feature. For XIAO use \
     `--no-default-features --features board-xiao`."
);

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

    // Board-selected LED pins. All choices are non-strapping and clear of
    // the native USB pins (12/13), UART0 (16/17), and any onboard hardware.
    //   nano/devkitc: GPIO18/19/20 — adjacent on the header, two pins from GND.
    //   xiao:         GPIO0/1/2    — pads D0/D1/D2, three adjacent pads.
    #[cfg(any(feature = "board-nano", feature = "board-devkitc"))]
    let (p_red, p_yellow, p_green) =
        (peripherals.GPIO18, peripherals.GPIO19, peripherals.GPIO20);
    #[cfg(feature = "board-xiao")]
    let (p_red, p_yellow, p_green) =
        (peripherals.GPIO0, peripherals.GPIO1, peripherals.GPIO2);

    let mut red = Output::new(p_red, Level::Low, OutputConfig::default());
    let mut yellow = Output::new(p_yellow, Level::Low, OutputConfig::default());
    let mut green = Output::new(p_green, Level::Low, OutputConfig::default());

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
