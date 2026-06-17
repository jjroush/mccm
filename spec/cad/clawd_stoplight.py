"""
Clawd + stop-light + XIAO ESP32-C6 desk figure — SINGLE COMBINED MODEL.

Code-first parametric CAD with build123d (OpenCASCADE kernel). This builds ONE
fused model (base + stop-light tower + Clawd, with internal cavities) and exports
a single STL: spec/cad/out/clawd_stoplight.stl

Design spec: ../3d-model.md   |   Setup: ../3d-modeling-setup.md

Run (inside the venv):
    source spec/cad/.venv/bin/activate
    python spec/cad/clawd_stoplight.py
    SHOW=1 python spec/cad/clawd_stoplight.py   # live VS Code preview (ocp-vscode)

Conventions
-----------
- Units mm. Z is up; the model sits on Z = 0.
- FRONT = -Y (LED windows / Clawd's face point this way, toward the viewer).
  BACK = +Y (the XIAO pocket opens here; USB-C faces out the back).
- The board STEP is a clearance reference only and is NEVER exported.

Print notes (one-piece)
-----------------------
- Orientation: print as-is, base flat on the bed (Z up).
- The tower's open back + the base's open-back XIAO pocket let you insert the
  PCB and ESP32 after printing; the pocket ceiling bridges ~19 mm (fine on FDM).
- Clawd needs tree supports; LED window tops bridge a few mm (add visors later if
  the droop bothers you). The XIAO antenna end is left as an internal air gap.
"""

import os
from pathlib import Path

from build123d import (
    Align,
    Box,
    Compound,
    Cylinder,
    Pos,
    Rot,
    export_stl,
    import_step,
    import_stl,
)

HERE = Path(__file__).resolve().parent
SPEC = HERE.parent
OUT = HERE / "out"
OUT.mkdir(exist_ok=True)

C = (Align.CENTER, Align.CENTER, Align.CENTER)
MIN = Align.MIN
CEN = Align.CENTER

# ---------------------------------------------------------------------------
# PARAMETERS  (measured 2026-06-16; see ../3d-model.md §8. ■ = re-confirm)
# ---------------------------------------------------------------------------
# Tolerances (generic-safe FDM, 0.4 mm nozzle / 0.2 mm layers)
WALL       = 2.0
PCB_CLEAR  = 0.4
PORT_CLEAR = 0.6
EMBED      = 1.0    # how far Clawd sinks into the base top (ensures a fused join)

# Stop-light PCB
SL_PCB_W   = 21.3   # board width (horizontal when standing)
SL_PCB_THK = 2.13   # bare PCB thickness
SL_LED_RIM = 9.19   # LED lens rim diameter -> window = rim + ~1.0
SL_LED_PITCH = 11.56
TOWER_DEPTH  = 16.0  # tower outer front-to-back (~5/8 in)

# XIAO ESP32-C6 (USB-C on one short edge -> faces BACK; antenna on the opposite)
XIAO_L   = 21.0     # along Y (front-back) inside the base
XIAO_W   = 17.8     # along X
XIAO_THK = 4.5      # board + components (from the STEP envelope ~4.46)
ANT_AIR  = 3.0      # air gap at the antenna (inner) end of the pocket

# Figure
TOWER_H        = 54.0
CLAWD_NATIVE_H = 52.0   # standing height after the upright rotation (old Y extent)
CLAWD_TARGET_H = 48.0   # slightly shorter than the 54 mm tower
CLAWD_YAW      = 0.0    # spin Clawd about Z if its face points the wrong way (try 180)
BASE_H         = 14.0
GAP            = 12.0   # space between tower and Clawd
MARGIN         = 5.0    # base border around the parts

# ---------------------------------------------------------------------------
# DERIVED
# ---------------------------------------------------------------------------
CLAWD_SCALE = CLAWD_TARGET_H / CLAWD_NATIVE_H
# After standing upright (rotX +90): width = old X (66), depth = old Z (35).
CLAWD_W = 66.0 * CLAWD_SCALE
CLAWD_D = 35.0 * CLAWD_SCALE

TWR_W = SL_PCB_W + 2 * PCB_CLEAR + 2 * WALL   # tower outer width (X)
TWR_D = TOWER_DEPTH                            # tower outer depth (Y)
CAV_W = SL_PCB_W + 2 * PCB_CLEAR               # tower cavity width

POCKET_W = XIAO_W + 2 * PCB_CLEAR              # X
POCKET_D = XIAO_L + 2 * PCB_CLEAR + ANT_AIR    # Y
POCKET_H = XIAO_THK + 2 * PCB_CLEAR            # Z

# Layout along X: [margin][tower][gap][clawd][margin]
TOWER_CX = MARGIN + TWR_W / 2
CLAWD_CX = MARGIN + TWR_W + GAP + CLAWD_W / 2
BASE_L = MARGIN + TWR_W + GAP + CLAWD_W + MARGIN
BASE_W = max(TWR_D, CLAWD_D, POCKET_D + MARGIN) + 2 * MARGIN
BACK_Y = BASE_W / 2                            # base back face (+Y)


# ---------------------------------------------------------------------------
# BUILD  (build the outer union, then subtract every internal void)
# ---------------------------------------------------------------------------
def build_solid():
    # --- outer solids ---
    base = Box(BASE_L, BASE_W, BASE_H, align=(MIN, CEN, MIN))
    tower = Pos(TOWER_CX, 0, BASE_H) * Box(TWR_W, TWR_D, TOWER_H, align=(CEN, CEN, MIN))
    solid = base + tower

    # --- tower cavity: open at the BACK (+Y) and BOTTOM, 2 mm roof on top ---
    cav = Pos(TOWER_CX, -TWR_D / 2 + WALL, BASE_H - 1) * Box(
        CAV_W, TWR_D, TOWER_H - WALL + 1, align=(CEN, MIN, MIN)
    )
    solid = solid - cav

    # --- three LED windows through the FRONT wall (-Y) ---
    win_r = (SL_LED_RIM + 1.0) / 2
    led_mid = BASE_H + TOWER_H - 16  # center the stack near the top of the tower
    for i in (-1, 0, 1):
        z = led_mid + i * SL_LED_PITCH
        solid = solid - (
            Pos(TOWER_CX, 0, z) * Rot(90, 0, 0) * Cylinder(radius=win_r, height=TWR_D + 2)
        )

    # --- XIAO pocket: opens at the base BACK (+Y); USB-C faces out the opening ---
    pocket_cy = BACK_Y - POCKET_D / 2
    pocket_cz = BASE_H / 2
    solid = solid - (
        Pos(TOWER_CX, pocket_cy, pocket_cz) * Box(POCKET_W, POCKET_D + 1, POCKET_H, align=C)
    )

    # --- wire channel: tower cavity bottom -> XIAO pocket ---
    chan_y0 = TWR_D / 2 - WALL          # ~ tower back interior
    chan_y1 = BACK_Y - POCKET_D + 2     # ~ pocket inner end
    solid = solid - (
        Pos(TOWER_CX, (chan_y0 + chan_y1) / 2, pocket_cz)
        * Box(8, abs(chan_y1 - chan_y0) + 2, BASE_H, align=(CEN, CEN, MIN))
    )
    return solid


def load_clawd():
    """Import the mascot, stand it upright (face -> front), scale, seat on base."""
    clawd = import_stl(str(SPEC / "clawd.stl"))
    clawd = Rot(90, 0, 0) * clawd          # mesh is modelled on its back -> stand it up
    clawd = Rot(0, 0, CLAWD_YAW) * clawd    # set CLAWD_YAW=180 if the face ends up at the back
    try:
        clawd = clawd.scale(CLAWD_SCALE)
    except Exception:  # noqa: BLE001
        from build123d import scale
        clawd = scale(clawd, by=CLAWD_SCALE)
    # Seat precisely: center on Clawd's slot in X/Y, feet on the base top.
    bb = clawd.bounding_box()
    dx = CLAWD_CX - (bb.min.X + bb.max.X) / 2
    dy = -(bb.min.Y + bb.max.Y) / 2
    dz = (BASE_H - EMBED) - bb.min.Z
    return Pos(dx, dy, dz) * clawd


# ---------------------------------------------------------------------------
# ASSEMBLE + EXPORT (one file)
# ---------------------------------------------------------------------------
if __name__ == "__main__":
    print("Building base + tower + cavities...")
    body = build_solid()

    print(f"Placing Clawd (scaled x{CLAWD_SCALE:.3f} -> {CLAWD_TARGET_H:.0f} mm)...")
    clawd = load_clawd()

    # Try a true boolean fuse; fall back to a multi-body compound (one STL either
    # way — overlapping shells are unioned by the slicer at print time).
    try:
        model = body + clawd
        print("  fused Clawd into the body (single solid).")
    except Exception as e:  # noqa: BLE001
        model = Compound(children=[body, clawd])
        print(f"  kept Clawd as a joined body in one file (boolean skipped: {e}).")

    out = OUT / "clawd_stoplight.stl"
    export_stl(model, str(out))
    bb = model.bounding_box().size
    print(f"Exported ONE model -> {out}")
    print(f"  overall size: {bb.X:.1f} x {bb.Y:.1f} x {bb.Z:.1f} mm")
    print("  (board STEP is a placeholder — never printed)")

    if os.environ.get("SHOW"):
        try:
            from ocp_vscode import show

            refs = []
            try:
                refs.append(import_step(str(SPEC / "seeed-studio-xiao-esp32c6-v2.step")))
            except Exception:  # noqa: BLE001
                pass
            show(model, *refs)
            print("Pushed preview to the ocp-vscode viewer.")
        except Exception as e:  # noqa: BLE001
            print(f"Preview skipped ({e}); open the OCP CAD Viewer in VS Code first.")
    else:
        print("Tip: run with SHOW=1 to push a live 3D preview to VS Code (ocp-vscode).")
