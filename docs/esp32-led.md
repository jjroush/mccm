# ESP32 status LEDs

`mccm led` mirrors the aggregate Claude Code session state to an ESP32 over
USB serial, driving three LEDs on a breadboard:

| LED    | State        | Meaning                          |
|--------|--------------|----------------------------------|
| Green  | `active`     | All sessions actively working    |
| Yellow | `inactive`   | At least one session is inactive |
| Red    | `needs_help` | At least one session needs help  |
| (off)  | none         | No live sessions                 |

The colors match the macOS menu bar icon exactly: green / yellow / red.

Target board: **MuseLab nanoESP32-C6** (ESP32-C6-WROOM-1, dual USB-C).
Firmware lives in [`esp32-firmware/`](../esp32-firmware/).

## Serial protocol

One ASCII byte plus newline per update, 115200 baud:

| Byte | Meaning      | LEDs            |
|------|--------------|-----------------|
| `R`  | needs help   | red on          |
| `Y`  | inactive     | yellow on       |
| `G`  | active       | green on        |
| `N`  | no sessions  | all off         |

The firmware also accepts `B` as a legacy alias for `Y` (the yellow LED
started life as a blue one), so a daemon and firmware from either side
of that rename still interoperate.

The host resends the current state every 2 seconds as a heartbeat, so the
board converges to the right state after a replug without any handshake.
The firmware ignores unknown bytes (including the `\n`), which keeps the
protocol forward-compatible.

## Board-specific design decisions (nanoESP32-C6)

The nanoESP32-C6 has **two USB-C ports**, and the choice matters:

- **`ESP32C6` port (next to the RST button)** — wired straight to the
  C6's built-in *USB-Serial-JTAG* peripheral (GPIO12/13 internally). This
  is the port to use: one cable both flashes the firmware and carries the
  status bytes, and it enumerates with Espressif's vendor ID
  (`303a:1001`, shows up as `/dev/cu.usbmodem*` on macOS), which is the
  first thing `mccm led` looks for when auto-detecting.
- **`CH343` port (next to the BOOT button)** — goes through a CH343P
  USB-UART bridge into UART0 (GPIO16/17). It also works (`mccm led`
  recognizes the WCH vendor ID `1a86` as a fallback), but it's a second
  code path in the firmware and the bridge's auto-reset circuit can reset
  the chip when the port is opened — or even during USB enumeration. We
  don't use it.

GPIO pin choice — **GPIO18 (red), GPIO19 (yellow), GPIO20 (green)**:

- They sit **side by side on the bottom header**, two pins from a GND, so
  the whole circuit fits in one short row of jumpers:
  `5V · GND · 9 · 18 · 19 · 20 · …` (bottom row, USB end on the left).
- They avoid every pin with a side job on the C6: GPIO4/5/8/9/15 are
  strapping pins sampled at reset (8/9 pick the boot mode — pulling these
  the wrong way bricks booting until rewired), GPIO12/13 are the native
  USB data lines (reusing them kills the serial link), GPIO16/17 are
  UART0 to the CH343, and GPIO24–30 are the flash inside the module.
- Embedded nuance: on a microcontroller, *which* pin you pick is rarely
  arbitrary — most pins double as boot-configuration inputs, debug
  interfaces, or bus lines. "Plain GPIO with no reset-time meaning" is
  the thing you're shopping for.

Other board facts worth knowing:

- There's an onboard **WS2812 addressable RGB LED on GPIO8**. It could
  show all three colors with zero wiring, but it needs a timing-critical
  one-wire protocol driver (RMT peripheral) instead of three `set_high()`
  calls, and GPIO8 is a strapping pin. Discrete LEDs are the better first
  project; the WS2812 is a nice follow-up.
- The BOOT button is GPIO9. If a bad flash ever makes the board
  unresponsive over USB, hold BOOT, tap RST, and it re-enters the ROM
  bootloader for recovery flashing.

## Parts (from a standard starter kit)

- 1 × breadboard
- 3 × LED: red, yellow, green
- 3 × 330 Ω resistor (220 Ω also fine; see current note below)
- 4 × male-male jumper wires
- nanoESP32-C6 + USB-C cable (plugged into the **ESP32C6** port)

## Schematic

Each GPIO sources current through a resistor and LED to ground
("active high"):

```
GPIO18 ───[330Ω]───►├─── ─┐         R = red LED
                  red      │
GPIO19 ───[330Ω]───►├─── ──┼─── GND
                  yellow   │
GPIO20 ───[330Ω]───►├─── ─┘
                  green

►├  = LED, long leg (anode) toward the resistor,
      short leg / flat side (cathode) toward GND
```

Electrical reasoning (the embedded nuances):

- C6 GPIOs swing 0 → 3.3 V. Red and yellow LEDs drop ~2.0–2.1 V, so a
  330 Ω resistor passes (3.3 − 2.1) / 330 ≈ **3.6 mA** — nicely visible.
  Green LEDs in kits are usually the same low-voltage chemistry. (Blue
  and white LEDs drop ~3.0 V+, which is why a blue LED on 330 Ω at 3.3 V
  is nearly invisible — the reason this project's blue became yellow.)
- The resistor is not optional: an LED is a diode, not a resistor — wired
  bare it would draw whatever current the pin can deliver and cook the
  LED, the pin, or both. The safe budget is ~10 mA per pin (the C6 can
  push more, but there's no reason to).
- Resistor on either leg of the LED works; the circuit is a series loop.

## Breadboard layout

The three GPIOs and GND are nearly adjacent on the **bottom header row**
(the row on the same side as the `CH343` silkscreen; USB ports to the
left):

```
nanoESP32-C6, bottom header (USB end → antenna end):
  5V   GND  GPIO9 GPIO18 GPIO19 GPIO20  GPIO21 ...
        │          │      │      │
        │          │      │      │              breadboard
        │          │      │      │     ┌─────────────────────────────┐
        │          │      │      └─────│ d1 ──[330Ω]── d5  ►├ GREEN  │
        │          │      └────────────│ c10──[330Ω]── c14 ►├ YELLOW │
        │          └───────────────────│ b18──[330Ω]── b22 ►├ RED    │
        │                              │      all cathodes → ( – )   │
        └──────────────────────────────│ ( – ) blue ground rail      │
                                       └─────────────────────────────┘
```

Step by step:

1. Seat the board across the breadboard's center channel (or next to the
   breadboard with jumpers, if you haven't soldered headers).
2. Jumper the board's **GND** (bottom row, 2nd pin from the USB end) to
   the breadboard's blue **(–) rail**.
3. For each LED: GPIO jumper → resistor → LED **long leg (anode)**;
   LED **short leg (cathode)** → (–) rail.
4. Nothing connects to 5V or 3V3 — the GPIOs themselves power the LEDs.

## Usage

```bash
# 1. Flash the firmware (see esp32-firmware/README.md)
cd esp32-firmware && cargo run --release

# 2. Run the LED daemon (auto-detects the board, reconnects on replug)
mccm led

# Or pin it to a specific device:
mccm led --port /dev/cu.usbmodem101
```

On boot the firmware runs a wiring self-test — all three LEDs flash
together three times, 2 seconds per flash — then holds **all three LEDs
on** until the daemon connects. All-on always means "no host daemon
talking"; it's also where the board lands if the heartbeat goes silent
for 10 seconds, so a dead daemon shows as obviously-disconnected instead
of a stale status.
