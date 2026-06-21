# 3D Modeling Setup (macOS, Apple Silicon) — first-timer guide

Everything to install and learn to design the [Clawd + stop-light figure](3d-model.md),
tailored to your choices: **code-first CAD**, **no printer yet** (design now, print
at a service/makerspace). Current as of mid-2026.

Your machine is ready: Homebrew 6.0.2 at `/opt/homebrew`, `python3`, and the
built-in `sips` are all present.

---

## TL;DR — the stack

| Role | Tool | Why |
|---|---|---|
| **CAD (primary)** | **`build123d`** (Python, OpenCASCADE kernel) | Code-first parametric CAD. Imports your STEP board, imports the mascot STL, exports STL — driven from dimensions as variables you can version-control. |
| **Live 3D preview** | **`ocp-vscode`** (VS Code) or **`yacv`** | See your `build123d` code rendered in 3D as you edit — you need eyes on the geometry. |
| **Visual sanity-check (optional)** | **FreeCAD** | Free, no account. Open the STEP/your STL to eyeball fit and the mascot's position — easier "by eye" than code. |
| **Slicer (preview only, no printer)** | **OrcaSlicer** | Preview supports/orientation and confirm printability before sending to a service. |
| **Image / HEIC** | built-in **`sips`** (or ImageMagick) | Convert the reference photo to PNG for a CAD canvas. Already done → `spec/stoplight-reference.png`. |

> **Why code-first here works** — `build123d` runs on a real B-rep kernel
> (OpenCASCADE), so it reads your `.step` board for exact clearances and gives you
> robust shells/fillets/cutouts. The enclosures (boxes, USB-C cutout, LED windows)
> are *ideal* for parametric code. The one awkward part is positioning the organic
> **mascot mesh** "by eye" — for that, glance at it in `ocp-vscode`/FreeCAD; you're
> only setting an offset, not modeling it.
>
> **Why not OpenSCAD** (the other code-first option): it **cannot read STEP**
> natively and has weak shells/fillets — a poor fit for *this* project.

---

## Install

### 1. CAD: `build123d` in a pinned Python venv

> ⚠️ Your default `python3` is **3.14**, which is very new — the OpenCASCADE
> wheels `build123d` depends on (`cadquery-ocp`) typically lag a release behind.
> Create the venv with **Python 3.12** to get prebuilt wheels and avoid a
> from-source build.

```bash
brew install python@3.12

# project-local venv (kept out of git)
cd /Users/roush/Developer/cc-middle-manager
/opt/homebrew/bin/python3.12 -m venv spec/cad/.venv
source spec/cad/.venv/bin/activate

python -m pip install --upgrade pip
pip install build123d ocp-vscode
```

Verify:

```bash
python -c "import build123d, OCP; print('build123d', build123d.__version__, 'OK')"
```

If `pip install build123d` tries to **compile** OCP (slow / fails), you're on the
wrong Python — confirm `python --version` says **3.12.x** inside the venv. See the
official install notes: <https://build123d.readthedocs.io/en/latest/installation.html>

### 2. Live preview: `ocp-vscode`

- Install the **VS Code extension** "OCP CAD Viewer" (Marketplace), or use the
  `ocp-vscode` package you just installed.
- In your script, `from ocp_vscode import show` then `show(part)` to render.
- Docs: <https://github.com/bernhard-42/vscode-ocp-cad-viewer>

### 3. Companions (brew casks — all verified to exist)

```bash
brew install --cask freecad        # free GUI to eyeball STEP/STL fit (optional)
brew install --cask orcaslicer     # slicer: preview printability (no printer needed)
brew install imagemagick           # `magick` CLI for images (sips is already built-in)
```

First launch of an unsigned cask app: **right-click → Open** to clear Gatekeeper.
Download slicers/CAD **only** from official sites — many top search results for
"OrcaSlicer download" are SEO mirror scams. Official:
<https://github.com/SoftFever/OrcaSlicer>.

### 4. (Optional) Mascot mesh cleanup: Blender

Only if you ever need to clean or re-pose Clawd — **not** needed for the enclosures.
```bash
brew install --cask blender
```

---

## GUI alternative (if code-first stops being fun)

You picked code-first, but if it gets in the way, the most beginner-friendly path
is **Autodesk Fusion (Personal Use)** — the one tool that mixes the imported mesh,
the STEP board, and new parametric solids in a clickable UI.

- **Still free for hobbyists in 2026** (non-commercial, < $1,000/yr from output),
  on yearly-renewable terms. No official Homebrew cask — download + create a free
  Autodesk account: <https://www.autodesk.com/products/fusion-360/personal>
- Personal license caveat: **STEP/STL *import* works and STL/3MF *export* works**
  (all you need to print), but **STEP *export* is disabled** and you're capped at
  10 editable docs. If you ever need to hand a machinist a STEP, use **FreeCAD**
  (free, no account, exports STEP).

---

## Learning path (in order)

You don't need to learn all of CAD — just enough for boxes with cutouts.

1. **`build123d` basics** — the [docs intro](https://build123d.readthedocs.io/)
   and "Introductory Examples." Learn `Box`, `Cylinder`, `Pos`/`Location`,
   `Mode.SUBTRACT`, `fillet`, `export_stl`.
2. **Make a hollow box** with a hole in one wall — that's 80% of the stop-light
   tower and the ESP32 enclosure.
3. **Import the STEP board** and use it as a fixed clearance reference; cut a
   USB-C opening around it.
4. **Import + scale the mascot STL** (`import_stl`, scale to target height); place
   it beside the tower. Glance at it in `ocp-vscode`.
5. **Export each part to STL**, open in OrcaSlicer to preview supports/orientation.
6. Concepts (orientation, supports, infill, tolerances) — the [3D-model spec
   §7](3d-model.md#7-fdm-tolerances--print-plan) has the numbers.

Prefer video / a GUI cross-reference? The best beginner Fusion channel is **Product
Design Online (Kevin Kennedy)** — his "Beginner Project: Electronics Enclosure"
playlist maps almost 1:1 onto the stop-light tower, even if you build in code.

---

## Printing without a printer

You're designing now and printing later at a **service or makerspace**:

- **Export each part as STL** (or **3MF** — carries units/color). `build123d`:
  `export_stl(part, "tower.stl")`.
- Tell the service: **material** (PLA or PETG), **color** (black for the tower),
  **layer height** ~0.2 mm. The figure is small — a few parts on one plate.
- Online services: Craftcloud, Treatstock, JLCPCB 3D printing, or a local library
  / makerspace (often cheapest). A makerspace also lets you iterate test-fits.
- Use the generic-safe tolerances in [§7](3d-model.md#7-fdm-tolerances--print-plan)
  since you can't tune to a specific printer yet.

---

## Quick reference — what's installed where

```bash
# CAD (inside venv)
source spec/cad/.venv/bin/activate
python spec/cad/clawd_stoplight.py     # builds + exports STLs

# Reference image (already generated)
open spec/stoplight-reference.png

# Eyeball fit
open -a FreeCAD spec/seeed-studio-xiao-esp32c6-v2.step
open -a OrcaSlicer                      # then import an exported STL
```
