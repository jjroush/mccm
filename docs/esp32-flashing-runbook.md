# Runbook — flashing the ESP32 status-LED firmware

How to build and flash new firmware to the ESP32, and how to recover from
the failures we've actually hit. Firmware lives in
[`../esp32-firmware/`](../esp32-firmware/); wiring and design are in
[`esp32-led.md`](esp32-led.md).

## TL;DR

```bash
# 1. Stop the daemon — it holds the serial port and blocks the flasher
pkill -f "mccm led"

# 2. Build + flash + monitor (from the firmware dir)
cd ~/Developer/cc-middle-manager/esp32-firmware
cargo run --release          # Ctrl-C exits the monitor; firmware keeps running

# 3. Restart the daemon to get live status back
~/.cargo/bin/mccm led        # run in a spare terminal, or background it
```

The golden rule: **only one program can hold the serial port.** The
`mccm led` daemon and `espflash` both want it, so the daemon must be
stopped before every flash and restarted after.

## Full procedure

### 1. Hardware check

- Use a **data** USB-C cable, not a charge-only one. If unsure, use the
  cable that last flashed successfully.
- Plug into the port wired to the **native USB-Serial-JTAG**:
  - nanoESP32-C6 → the connector silkscreened **`ESP32C6`** (next to RST).
  - Espressif ESP32-C6-DevKitC-1 → the connector silkscreened **`USB`**
    (the other one, `UART`, goes through a bridge chip and also works).
- Confirm the board enumerated:
  ```bash
  ls /dev/cu.usbmodem*
  ```
  You should see something like `/dev/cu.usbmodem101`. No device → it's a
  cable or port problem, not software (see Troubleshooting).

### 2. Stop the daemon

```bash
pkill -f "mccm led"
```

Verify nothing is holding the port:

```bash
lsof /dev/cu.usbmodem*        # should print nothing
```

If you've installed the daemon as a launchd LaunchAgent, `pkill` isn't
enough — launchd relaunches it within seconds. Unload it for the duration
of the flash instead:

```bash
launchctl bootout gui/$(id -u)/io.roush.mccm.led    # only if the LaunchAgent exists
```

### 3. Build and flash

```bash
cd ~/Developer/cc-middle-manager/esp32-firmware
cargo run --release
```

`cargo run` invokes `espflash flash --monitor --chip esp32c6` via the
runner in `.cargo/config.toml`. Expect:

- a build (instant if nothing changed),
- chip detection + flash (unchanged segments are skipped — fast),
- `Flashing has completed!`,
- the serial monitor opens, showing the ESP-IDF bootloader's `I (…) boot:`
  lines. **The firmware itself prints nothing** — it never writes to USB
  by design. Silence after the boot log is normal, not a hang.

On the board you'll see the boot self-test: all three LEDs flash together
three times (2 s each), then hold all-on until the daemon connects.

Press **Ctrl-C** to exit the monitor. The firmware keeps running.

### 4. Restart the daemon

```bash
~/.cargo/bin/mccm led
```

Run it in a spare terminal tab, or background it. Within a couple of
seconds it prints `Connected to /dev/cu.usbmodem…` and the first
`state -> …`, and the board's all-on collapses to the live status color.

> Note: a bare `mccm` may still resolve to the old Homebrew 0.5.2 build,
> which has no `led` subcommand. Use the full path `~/.cargo/bin/mccm`, or
> `brew unlink mccm` to let the cargo build win.

## Flash without re-building (optional)

If you already have a release binary and just want to reflash:

```bash
espflash flash --chip esp32c6 --port /dev/cu.usbmodem101 \
  target/riscv32imac-unknown-none-elf/release/mccm-led-firmware
```

Pass `--port` explicitly when more than one serial device is present —
otherwise espflash shows an interactive picker (and fails with "not a
terminal" if run non-interactively).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `No serial ports could be detected` | Charge-only cable, or board not enumerating. `ls /dev/cu.usbmodem*` shows nothing; USB bus has no new device. | Swap to a known-good **data** cable. Check the power LED: lit but no serial = charge-only cable; dark = no power (dead cable/port). Try another Mac port. |
| `Device or resource busy` | Another program holds the port — almost always the `mccm led` daemon. | `pkill -f "mccm led"`, confirm with `lsof /dev/cu.usbmodem*`, reflash. If a LaunchAgent, `launchctl bootout` it. |
| `not a terminal` (dialoguer error) | espflash needs to show a port picker but isn't attached to a TTY. | Pass `--port /dev/cu.usbmodem101` explicitly. |
| Board flashes, then "goes away" / resets | Plugged into the **CH343/UART** port, whose auto-reset circuit toggles the chip on enumeration. | Move the cable to the native **USB / ESP32C6** port. |
| Picked `tty.*` and it hangs on open | `tty.*` (dial-in) blocks waiting for carrier-detect. | Always use the `cu.*` (callout) device on macOS. |
| Flash OK but USB port dead afterward (bad firmware) | Firmware crashed early or repurposed the USB pins (GPIO12/13). | Recovery: hold **BOOT**, tap **RST**, release BOOT → ROM bootloader, then reflash. |
| `error: unrecognized subcommand 'led'` | Running old Homebrew `mccm`, not the branch build. | `~/.cargo/bin/mccm led`, or `cargo install --path .` then `brew unlink mccm`. |

## Why the daemon and flasher conflict

A serial port is an exclusive resource — the OS lets exactly one process
open `/dev/cu.usbmodem101` at a time. `mccm led` opens it to stream status
bytes and keeps it open. `espflash` needs it to talk to the bootloader.
So every flash is: **stop daemon → flash → restart daemon.** This runbook
exists because that ordering isn't obvious until it bites you.
