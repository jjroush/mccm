# Clawd Menu Bar Icons

Place Clawd (Claude mascot) icon PNGs here. The SwiftBar plugin will
automatically pick them up and display them in the macOS menu bar.

## Required Files

| File                | Used When                          |
|---------------------|------------------------------------|
| `clawd-green.png`   | All sessions actively working      |
| `clawd-yellow.png`  | At least one session is inactive   |
| `clawd-red.png`     | At least one session needs help    |
| `clawd-none.png`    | No live sessions                   |

## Icon Specs

- **Size:** 18x18 px (or 36x36 @2x for Retina)
- **Format:** PNG with transparency
- **Style:** Clawd mascot silhouette tinted/outlined in the status color
  - Green Clawd: happy, actively working
  - Yellow Clawd: sleeping or idle
  - Red Clawd: confused or waving for help
  - Gray/None Clawd: dimmed or outline-only

SwiftBar will base64-encode the PNG at runtime. Keep files small (<10 KB each).

## Fallback

If icons are missing, the plugin falls back to colored circle emoji
(🟢 🟡 🔴 ⚪).
