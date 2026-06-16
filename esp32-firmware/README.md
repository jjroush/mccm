# mccm-led-firmware

Bare-metal Rust firmware (esp-hal, `no_std`) for the MuseLab
**nanoESP32-C6** that drives three status LEDs from `mccm led`.
Wiring, pin choices, and the serial protocol are documented in
[`../docs/esp32-led.md`](../docs/esp32-led.md).

## One-time setup

The ESP32-C6 is RISC-V, so this builds on plain stable Rust — no espup /
Xtensa toolchain involved. `rust-toolchain.toml` pulls in the
`riscv32imac-unknown-none-elf` target automatically on first build.

You need espflash; 4.x is the version paired with esp-hal 1.x
(3.3.0 is the floor and also works):

```bash
cargo install espflash --locked
```

## Flash

Plug the USB-C cable into the port labeled **ESP32C6** (next to the RST
button) — that's the chip's native USB-Serial-JTAG, used for both
flashing and the status protocol. The other port (CH343) is a plain
UART bridge this firmware doesn't listen on.

```bash
cargo run --release   # builds, flashes, and opens a serial monitor
```

(`cargo run` invokes `espflash flash --monitor` via the runner in
`.cargo/config.toml`. Ctrl-C exits the monitor; the firmware keeps
running.)

On boot the firmware runs a wiring self-test — all three LEDs flash
together three times, 2 s per flash — then all three stay lit until
`mccm led` connects. All-on means "no host daemon" — the board also
returns there if no byte arrives for 10 s, rather than showing a stale
status.

## Recovery

If a future bad flash ever makes the native USB port unresponsive:
hold **BOOT**, tap **RST**, release BOOT. The chip re-enters the ROM
download mode and `espflash` can flash it again.

## Embedded notes (for app devs)

- `#![no_std]`/`#![no_main]`: there's no OS — no heap, no `println!`,
  no exit. `main` returns `!` because there is nowhere to return to.
- `esp_hal::init()` hands out a `Peripherals` struct of singletons;
  ownership of `peripherals.GPIO18` moving into `Output::new` is how
  Rust guarantees at compile time that nothing else can drive that pin.
- The watchdog timers (which reboot the chip if firmware wedges) are
  disabled by `esp_hal::init()`'s default config, so a simple poll loop
  is fine without "feeding" anything.
- `esp_app_desc!()` embeds a metadata block the ESP-IDF second-stage
  bootloader looks for; without it, newer espflash refuses the image.
- The main loop polls with a non-blocking `read_byte()` instead of a
  blocking read so the idle-timeout logic can run while no data is
  arriving. The 1 ms sleep keeps the poll from spinning the CPU flat
  out for no reason.
