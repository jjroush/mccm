# Agent runbook — set up a Claude Code traffic light (XIAO ESP32-C6, macOS)

You are an AI coding agent (e.g. Claude Code) running on the user's Mac,
helping them turn an **unflashed Seeed XIAO ESP32-C6** + a **3-color
traffic-light LED module** (R/Y/G, common ground, built-in resistors) into a
desk status light driven by `mccm led`. The human equivalent of this doc is
[`xiao-setup-guide.html`](xiao-setup-guide.html).

## Operating rules

- **You can do:** install tooling, clone/build, flash, run/verify the daemon,
  read serial ports, diagnose errors. Do these yourself via the shell.
- **You cannot do:** soldering, plugging/unplugging cables, pressing BOOT/RESET,
  wiring the module. For each such step, give the user a precise instruction,
  then **stop and wait** for them to confirm before continuing.
- **Gate every phase.** Run the verification command and do not advance until it
  passes. Report the actual output, don't assume success.
- **Idempotent.** Every install step is safe to re-run; if a tool is already
  present, say so and move on.
- **Hardware target facts (do not re-derive):** ESP32-C6 → RISC-V target
  `riscv32imac-unknown-none-elf`, builds on stable Rust. The XIAO's single USB-C
  is native USB-Serial-JTAG (`303a:1001`), enumerates as `/dev/cu.usbmodem*`,
  no macOS driver needed. LED firmware feature = `board-xiao` (red=GPIO0=pad D0,
  yellow=GPIO1=D1, green=GPIO2=D2; module GND→XIAO GND). Active-high drive suits
  a common-cathode module.

---

## Phase 0 — Preconditions

Confirm with the user (one message, don't belabor):
- They have the XIAO, the header pins, the traffic-light module, a soldering
  iron, 4 jumper wires, and a **USB-C data cable** (not charge-only).
- They're on macOS.

If they lack a soldering iron or data cable, stop — those are hard blockers.

---

## Phase 1 — Machine setup (agent-executable)

Run each, report result, continue on success.

```bash
# 1.1 Apple CLT (compiler + git). Prints "already installed" if present.
xcode-select -p >/dev/null 2>&1 && echo "CLT present" || xcode-select --install

# 1.2 Homebrew (skip if `brew` exists)
command -v brew >/dev/null && echo "brew present" || \
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
# If just installed, ensure it's on PATH for this session:
[ -x /opt/homebrew/bin/brew ] && eval "$(/opt/homebrew/bin/brew shellenv)"

# 1.3 Rust (skip if `cargo` exists)
command -v cargo >/dev/null && echo "rust present" || \
  (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y)
source "$HOME/.cargo/env" 2>/dev/null || true

# 1.4 espflash
command -v espflash >/dev/null && echo "espflash present" || cargo install espflash --locked

# 1.5 mccm (host binary + led daemon)
command -v mccm >/dev/null && echo "mccm present" || brew install jjroush/tap/mccm

# 1.6 Firmware source
[ -d "$HOME/mccm" ] || git clone https://github.com/jjroush/mccm.git "$HOME/mccm"
```

**Gate 1:**

```bash
cargo --version && espflash --version && mccm --help >/dev/null 2>&1 && echo "mccm OK"
```

All three must succeed. If a `command not found` appears, the shell hasn't
picked up a freshly-installed tool — re-`source "$HOME/.cargo/env"` and ensure
`brew shellenv` ran, then retry. Do not proceed otherwise.

---

## Phase 2 — Solder headers (HUMAN ACTION — instruct & wait)

You cannot do this. Tell the user, verbatim intent:

> Push the header pin strips into the holes along both long edges of the XIAO
> (pins down), and solder each pin to its silver ring. You need at least **D0,
> D1, D2** (top three on one edge) and **GND**, but soldering all pins keeps the
> board sturdy. Each joint should be a small shiny cone — not a dull blob, and
> not bridging two pins. First time? Watch a short "soldering header pins" video;
> the rule is *heat the metal, feed solder to the metal*.

Then **wait** for the user to confirm soldering is done. There is no software
gate for this; the real test is the self-test in Phase 4.

---

## Phase 3 — Flash the firmware (agent-executable, with human cable/button help)

Ask the user to **plug the XIAO into USB-C**, then:

**Gate 3a — board enumerated:**

```bash
ls /dev/cu.usbmodem* 2>/dev/null && echo "port present" || echo "NO PORT"
```

- `NO PORT` → likely charge-only cable. Tell the user to swap to a data cable.
  Re-run until a port appears.

**Flash:**

```bash
# Stop the daemon first in case it's holding the port (safe if not running).
pkill -f "mccm led" 2>/dev/null; sleep 1
cd "$HOME/mccm/esp32-firmware"
cargo run --release --no-default-features --features board-xiao
```

`cargo run` opens an interactive monitor after flashing. If your shell can't
handle interactive processes, flash without the monitor instead:

```bash
cargo build --release --no-default-features --features board-xiao
PORT=$(ls /dev/cu.usbmodem* | head -1)
espflash flash --chip esp32c6 --port "$PORT" \
  target/riscv32imac-unknown-none-elf/release/mccm-led-firmware
```

**Success criteria:** output contains `Flashing has completed!`.

**Failure branch — `espflash::timeout` / "Timeout while running command":**
The XIAO didn't auto-enter download mode. Instruct the user (HUMAN ACTION):

> Hold the **BOOT** button. While holding it, unplug and replug the USB-C cable
> (or tap RESET). Release BOOT.

Then re-run the flash command. A serial monitor would show `waiting for
download` when it's ready; the re-run connects immediately. The
`Saved PC: ... core::mem::replace` line in that banner is not an error.

**Failure branch — `Device or resource busy`:** `mccm led` (or a monitor) holds
the port. `pkill -f "mccm led"`, confirm with `lsof /dev/cu.usbmodem*` (empty),
reflash.

---

## Phase 4 — Wire the module (HUMAN ACTION — instruct & wait)

Ask the user to unplug USB, then wire **by label** (module pin → XIAO pad):

| Module | XIAO pad |
|--------|----------|
| R      | D0       |
| Y      | D1       |
| G      | D2       |
| GND    | GND (right edge, 2nd from bottom) |

No resistors/breadboard — the module has built-in resistors and a common
ground. Wait for confirmation.

**Gate 4 — self-test:** have the user plug USB back in and watch the lights.
On power-up the firmware flashes **all three LEDs together, 3×**. If they see
that, soldering + wiring are good. If not:
- No lights at all → re-check GND wire and that flashing succeeded (Phase 3).
- Some lights missing/flickering → cold solder joint or loose jumper on the
  missing color's pad; reheat/reseat.

---

## Phase 5 — mccm setup (agent-executable)

```bash
# 5.1 Install Claude Code hooks + menu bar daemon (idempotent).
mccm install

# 5.2 Start the LED daemon in the background.
mccm led
```

**Gate 5 — daemon connected:** the daemon prints, within ~2 s:

```
Connected to /dev/cu.usbmodem...
state -> G   (or Y / R / N)
```

If you ran it backgrounded, capture and show that output to confirm. If it
prints `rescanning...` repeatedly, the board isn't enumerating — recheck the
cable/port (Gate 3a).

---

## Phase 6 — End-to-end verification

1. With a **Claude Code session active**, the light is **green** (`state -> G`).
2. When the session goes idle, it turns **yellow** (`state -> Y`).
3. A session needing input turns it **red** (`state -> R`); no live sessions =
   **off** (`state -> N`).

Demonstrate by checking the daemon's printed transitions match the light.

**Done.** Summarize for the user: what color means what, that `mccm led` must
stay running (background it or it stops after the session), and the re-flash
rule — **stop `mccm led` before flashing, restart after** (they share the port).

---

## Quick reference

| Need | Command |
|------|---------|
| Flash (XIAO) | `cargo run --release --no-default-features --features board-xiao` |
| Force bootloader | hold BOOT → replug USB → release BOOT → reflash |
| Find the port | `ls /dev/cu.usbmodem*` |
| Free a busy port | `pkill -f "mccm led"` |
| Run the light | `mccm led` |
| Color map | green=active · yellow=idle · red=needs help · off=none |
