# Clawd Menu Bar Icons

Pixel art Clawd (Claude mascot) icons for the macOS menu bar. The
SwiftBar plugin base64-encodes these at runtime and displays them as
the status indicator.

## Included Icons

| File                | Status       | Meaning                          |
|---------------------|--------------|----------------------------------|
| `clawd-green.png`   | Active       | All sessions actively working    |
| `clawd-yellow.png`  | Inactive     | At least one session is inactive |
| `clawd-red.png`     | Needs help   | At least one session needs help  |
| `clawd-none.png`    | No sessions  | No live sessions                 |

## Icon Specs

- **Size:** 36x36 px (@2x Retina — displays as 18x18 pt)
- **Format:** PNG with transparent background
- **Style:** Pixel art Clawd silhouette — rectangular body, side ears,
  square eyes, four legs. Solid fill in the status color.

## Customization

To replace icons, keep the same filenames and dimensions. SwiftBar
will base64-encode the PNG at runtime — keep files small (<10 KB each).

## Fallback

If icons are missing, the plugin falls back to colored circle emoji
(🟢 🟡 🔴 ⚪).
