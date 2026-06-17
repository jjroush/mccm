# 3D Model — Clawd + Stop Light + ESP32 Desk Figure

A printed desk object: the **Clawd** mascot standing next to a **3-LED traffic-light
PCB** ("stop light"), with a **Seeed XIAO ESP32-C6** hidden inside, its **USB-C
port exposed out the back**. The ESP32 drives the LEDs to mirror Claude Code
session health (see [`docs/esp32-led.md`](../docs/esp32-led.md)).

> New to 3D modeling? Read [`3d-modeling-setup.md`](3d-modeling-setup.md) first —
> it covers what to install and the learning path. This file is the *design spec*.

---

## 1. Goal

One cohesive desk figure that:

1. Houses the stop-light PCB in a **black traffic-light tower** with the three
   LEDs visible.
2. Stands **Clawd** beside the tower on a **shared base**.
3. Encloses the **XIAO ESP32-C6** inside the base, with the **USB-C connector
   reachable from the back** for power + flashing.
4. **Routes the 4 LED wires** (GND/R/Y/G) from the tower down through the base to
   the ESP32, hidden from view.

---

## 2. Components & verified dimensions

All three physical references live in `spec/`. **Trust the CAD files over any
catalog number; trust your own calipers over both.**

### 2.1 Stop-light LED module (`stoplight-reference.png`)
Common hobbyist "traffic light LED module": a black PCB with three 5 mm LEDs
(red/yellow/green) and a 1×4 header on a thin neck with two mounting holes.

| Property | Value (catalog baseline — **verify with calipers**) |
|---|---|
| PCB body | ~56 × 21 × ~11 mm (L×W×thickness); listings vary 55–60 × 20–21 |
| LEDs | 3 × **5 mm** (lens *rim* ≈ 5.8–6 mm — measure the rim, not the dome) |
| Header | 1×4, **2.54 mm pitch**, labelled GND / R / Y / G (order varies — **read the silkscreen**) |
| Mounting holes | Ø3 mm, 15 mm pitch |
| Electrical | 5 V, common-cathode, built-in resistors |

**Confirmed measured values (2026-06-16, see [§8](#8-measured-values)):** PCB
56 × 21.3 × 2.13 mm; LEDs are the **8 mm-class** variant (lens rim **9.19 mm**,
dome stands **12.9 mm** above the board) at **11.56 mm** pitch; mounting holes
**Ø3.33 mm**; header order (silkscreen) **G · Y · R · GND**; USB-C cable boot
**11.3 × 5.3 mm**. Tower target height **54 mm** (2-1/8 in), depth ~16 mm (5/8 in).

> ⚠️ Two near-identical products exist (5 mm and 8 mm LED versions) and pin order
> is **not** universal — this board is the **8 mm** variant with order
> **G·Y·R·GND**. See the [measured values](#8-measured-values).

### 2.2 Seeed XIAO ESP32-C6 (`seeed-studio-xiao-esp32c6-v2.step`)
The "brain." Import the STEP as a **fixed clearance reference** — **never export it
into a print file.** STEP bounding box of control points measured locally:
≈ 21 × 17.8 mm board (Z spans ~25 mm because the model includes downward header
pins).

| Property | Value | Notes |
|---|---|---|
| PCB outline | **21.0 × 17.8 mm**, ~1.5 mm corner radius | Use 17.8, not the marketing "17.5" |
| PCB thickness | ~1.0 mm | bare board |
| Height w/ components | ~2.5–3.5 mm (no headers) | measure if the slot is tight |
| Pins | 14 (2×7), 2.54 mm pitch; columns 15.25 mm apart | **no screw holes** — pocket/capture the board |
| **USB-C** | centered on one **short (17.8 mm) edge**, protrudes ~1.5 mm | this edge points **out the back** |
| **Antenna keep-out** | chip antenna + U.FL on the short edge **opposite USB-C** | **leave this end OPEN** — plastic/metal here kills Wi-Fi/BLE |
| Buttons | RESET + BOOT, both at the USB-C corner | add cutouts if you want to reflash without opening |

Source: <https://wiki.seeedstudio.com/xiao_esp32c6_getting_started/>

### 2.3 Clawd mascot (`clawd.stl`)
Pre-made low-poly mesh — **124 triangles**, bounding box **66 (X) × 52 (Y) ×
35 (Z) mm**, modelled **lying on its back** (the character silhouette is in the XY
plane). Stand it up with a **+90° rotation about X** → 66 W × 35 D × **52 H**. This
is decoration: position/scale/rotate it, don't redesign it.

---

## 3. Functional requirements

- **R1** Tower fully encloses the stop-light PCB in black, with the 3 LEDs
  visible through round windows or open faces.
- **R2** Tower is **slightly taller than Clawd** (the lights "crown" the figure).
- **R3** XIAO sits inside the base; **USB-C reachable from the back**; **antenna
  end left open**.
- **R4** A hidden channel routes the 4 header wires from tower → base → ESP32,
  with strain relief (wires anchor to printed plastic, **not** the header pins).
- **R5** Everything sits on **one shared base** (single desk object).
- **R6** Enclosures **open** to insert boards and route wires (removable cover or
  back panel — not a sealed box).

---

## 4. Locked design decisions

| Decision | Choice | Rationale |
|---|---|---|
| Assembly | **One combined model** — a single fused STL (base + tower + Clawd) | You wanted one model with everything set up; electronics insert through printed openings. |
| Access | Tower open back + base open-back XIAO pocket; no separate covers | Slide the PCB and ESP32 in after printing; pocket ceiling bridges ~19 mm (fine on FDM). |
| Mascot join | Clawd embedded 1 mm into the base, exported in the same file | OCCT won't boolean the mesh shell, so it's a joined body — the slicer unions the overlap. |
| CAD tool | **Code-first: `build123d`** (Python/OpenCASCADE) | Parametric, version-controlled, imports STEP + STL, exports STL. See setup guide. |
| Mascot in code | Keep as a **mesh placeholder** for placement; **glue on** (no boolean) | Avoids fragile mesh→solid booleans; placement is just an offset. |
| Print path | **No printer yet** → design now, **send STL/3MF to a print service / makerspace** | Use generic-safe FDM tolerances (Section 7). |

### Clawd sizing & orientation
The mesh is modelled **lying on its back** — its real standing height is the old
**52 mm** Y-extent, and it must be rotated **+90° about X** to stand up (face → front,
feet → down). Standing, it's 66 W × 35 D × 52 H before scaling. **Default: scale to
`48 mm` tall** (×0.92 → ~61 W × 32 D), slightly shorter than the 54 mm tower — a
balanced companion. One parameter (`CLAWD_TARGET_H`) tunes height; `CLAWD_YAW = 180`
flips the face if needed.

---

## 5. Parametric variables (the knobs)

Driven from [`cad/clawd_stoplight.py`](cad/clawd_stoplight.py). Measure, then set:

```
# Tolerances (generic-safe FDM — Section 7)
WALL = 2.0 ; PCB_CLEAR = 0.4
PRESS_FIT = 0.25 ; SLIP_FIT = 0.45 ; PORT_CLEAR = 0.6 ; PIN_UNDER = 0.15
# Stop-light PCB (■ caliper these)
SL_PCB_L, SL_PCB_W, SL_PCB_THK       # board outline + thickness
SL_LED_DIA = 5.0 ; SL_LED_RIM = 6.0  # body / lens rim (window = rim + ~1.0)
SL_LED_PITCH                         # center-to-center of the 3 LEDs
# XIAO (mostly known; trust the STEP)
XIAO_L = 21.0 ; XIAO_W = 17.8 ; XIAO_THK = 3.5   # board + components (pocket depth)
USBC_W = 9.5 ; USBC_H = 4.0          # ■ measure the cable BOOT, not the connector
# Figure
TOWER_H = 54.0                       # taller than Clawd
CLAWD_NATIVE_H = 52.0                # standing height (after +90° X rotation)
CLAWD_TARGET_H = 48.0 ; CLAWD_YAW = 0
BASE_H = 14.0 ; GAP = 12.0
```

---

## 6. Enclosure design details

### 6.1 Stop-light tower
- Hollow box, internal cavity = `SL_PCB_W + 0.4` wide × `TOWER_DEPTH − WALL` deep,
  height `TOWER_H`; PCB captured at the front, LEDs protrude through the windows.
- **Walls ≥ 2.0 mm** in black filament (thin black walls glow/bleed between LEDs).
- Capture the PCB with **two internal ledge rails** + a **removable back cover**
  (easier for a beginner than a friction slot).
- **Round LED windows** at `LED rim Ø + ~1.0 mm` (~6 mm for 5 mm LEDs), centered
  on your **measured** LED pitch. Optional "visor" lip above each for the classic
  look. Chamfer window tops so the bridge doesn't droop when printed upright.
- **Wire-exit notch** (~6–7 mm) near the header for a bundled 4-wire lead, routed
  down into the base; add a strain-relief boss.

### 6.2 XIAO enclosure (inside the base)
- Pocket sized to the **grounded STEP envelope** + `PCB_CLEAR` per side.
- **USB-C cutout** on the **back face**, sized for the **cable boot**, not the bare
  connector: opening ≈ `USBC_W + 2·PORT_CLEAR`.
- **Leave the antenna end open** (open slot / air gap; no thick plastic, no metal).
- Optional **BOOT/RESET cutouts** at the USB-C corner if you want to reflash
  without opening the case.
- Removable lid (snap or 2 small screws) for board access.

### 6.3 Base + wire routing
- Footprint spans the tower + Clawd; thick enough to hide the XIAO and a wire
  channel beneath the surface.
- Internal channel: tower wire-notch → ESP32 header pins.
- Alignment-pin sockets for the tower and (optionally) Clawd.

### 6.4 Mascot placement
- `import_stl("clawd.stl")` → **rotate +90° about X to stand it up** (face → front)
  → scale to `CLAWD_TARGET_H` (basis 52 mm) → seat on the base beside the tower.
- Embedded 1 mm into the base and exported in the same file, so the whole figure
  prints as one connected object.

---

## 7. FDM tolerances & print plan

Generic-safe values for sending to a print service (PLA/PETG, 0.4 mm nozzle,
0.2 mm layers):

| Fit | Clearance |
|---|---|
| Press / snap fit | **0.2–0.3 mm** |
| Slip / clearance fit | **0.4–0.5 mm** |
| Around a PCB | ~0.5 mm/side |
| USB-C cutout | **+0.5 mm/side**, sized for the cable **boot** |
| Walls | ≥1.5 mm, **2.0 mm recommended** |
| Holes print ~0.1–0.3 mm undersized | size up or test-fit |
| Alignment pins | **0.1–0.2 mm under** the hole, chamfered tip |

**Print orientation / supports (one-piece)**
- Print the whole model **base-down** (as exported); the tower and Clawd stand up.
- Supports: tree/organic under Clawd's arms and the LED window tops (a few-mm
  bridges); the XIAO pocket ceiling bridges ~19 mm (OK on FDM).
- **Never** export the STEP board into a print file — it's a placeholder.

**Sending to a service (no printer):** send `cad/out/clawd_stoplight.stl` (or
**3MF**, which can carry color/units); state material (PLA/PETG), color (black for
the LEDs to read well), layer height ~0.2 mm. It's a single small part.

---

## 8. Measured values

Calipered 2026-06-16 (these drive [`cad/clawd_stoplight.py`](cad/clawd_stoplight.py)):

| Measurement | Value | Notes |
|---|---|---|
| PCB L × W × thickness | **56 × 21.3 × 2.13 mm** | W at the widest (base); thickness is the bare board |
| LED lens rim Ø | **9.19 mm** | 8 mm-class LED — window cut ≈ 10.2 mm (rim + 1.0) |
| LED dome height (protrusion) | **12.9 mm** | above the board face; sets tower depth (≈ 16 mm) |
| LED pitch (center-to-center) | **11.56 mm** | the 3 lights along the board length |
| Mounting holes | **Ø3.33 mm**, ~1.4 mm from edge | not used for capture (we pocket the board) |
| Header | **straight**, order **G · Y · R · GND** | from silkscreen (note: not the generic GND/R/Y/G) |
| USB-C cable boot | **11.3 × 5.3 mm** (W × H) | back cutout = boot + 2 × 0.6 mm |
| Clawd "up" axis / height | **Z = 35 mm native → 40 mm** target | default scaling (×1.14) |

> ⚠️ One thing to re-confirm visually: I read **P = 12.9 mm** as the LED dome
> height *above the board face*. With the 2.13 mm board that's ~15 mm total
> front-to-back, matching your "5/8 in" depth note — so `TOWER_DEPTH` is set to
> 16 mm. If 12.9 was measured differently, tweak that one variable.

---

## 9. References & files

| File | Role |
|---|---|
| `spec/3d-model.md` | this design spec |
| `spec/3d-modeling-setup.md` | what to install + learning path |
| `spec/cad/clawd_stoplight.py` | parametric `build123d` starting scaffold |
| `spec/clawd.stl` | mascot mesh (124 tri, 66×52×35 mm) — position & scale, don't redesign |
| `spec/seeed-studio-xiao-esp32c6-v2.step` | XIAO clearance reference — **do not print** |
| `spec/stoplight-reference.png` | stop-light photo (converted from `IMG_4244.HEIC`) — use as a CAD canvas |

- Clawd mascot source: <https://www.printables.com/model/1338408-3d-model-for-seeed-studio-xiao-esp32c6/files>
- XIAO ESP32-C6 wiki: <https://wiki.seeedstudio.com/xiao_esp32c6_getting_started/>

---

## 10. TODO / next steps

Print-ready as a display piece today; these are refinements:

- [ ] **LED visors** — small overhanging lips above each window for the classic
      signal look.
- [ ] **Chamfer the LED window tops** (45°) so the bridge doesn't droop when printed
      upright.
- [ ] **Internal PCB ledge rails** in the tower so the board seats at a repeatable
      depth (currently cavity + friction).
- [ ] **Dry-fit check** the USB-C pocket opening and the antenna air-gap against the
      real XIAO before a final print.
- [ ] **Strain-relief boss** in the base wire channel (anchor wires to plastic, not
      the header pins).
- [ ] _(Optional)_ Convert the mascot mesh to a solid and boolean-fuse it for a single
      watertight body instead of a joined overlap.
- [ ] _(Optional)_ **3MF export** with black material assigned, for a print service.
