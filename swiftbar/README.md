# mccm SwiftBar Plugin

A macOS menu bar indicator showing aggregate Claude Code session health
via a Clawd (pixel art Claude mascot) icon.

## Prerequisites

- macOS
- [SwiftBar](https://github.com/swiftbar/SwiftBar) (`brew install --cask swiftbar`)
- `jq` (`brew install jq`)
- mccm installed with hooks active (`mccm install`)

## Setup

1. **Install SwiftBar**
   ```bash
   brew install --cask swiftbar
   ```

2. **Launch SwiftBar** and choose a plugin directory when prompted
   (e.g., `~/swiftbar-plugins/`).

3. **Symlink the plugin** into the SwiftBar plugin directory:
   ```bash
   cd /path/to/mccm/swiftbar
   ln -s "$(pwd)/mccm-status.5s.sh" ~/swiftbar-plugins/mccm-status.5s.sh
   ```

4. **Copy the icons** next to the plugin script:
   ```bash
   cp -r "$(pwd)/icons" ~/swiftbar-plugins/icons
   ```

   The plugin resolves the `icons/` directory relative to its own
   location. If you symlinked the script (step 3), the icons directory
   inside this repo is used automatically — no copy needed.

5. **Verify** — you should see a gray Clawd icon in your menu bar.
   Start a Claude Code session and it will turn green.

## How It Works

The plugin reads `~/.claude/mccm/state.json` (written by mccm hooks)
every 5 seconds and derives an aggregate status:

| Icon   | Meaning                               |
|--------|---------------------------------------|
| Green  | All sessions are actively working     |
| Yellow | At least one session is inactive      |
| Red    | At least one session needs help       |
| Gray   | No live sessions                      |

Clicking the icon shows a dropdown with:
- Session counts by status (active, inactive, needs help)
- Per-session details with project names
- Quick action to open the mccm TUI
- Refresh button

## Customization

- **Refresh interval:** rename the file (e.g., `mccm-status.3s.sh` for 3 seconds)
- **Icons:** replace the PNGs in `icons/` — see `icons/README.md` for specs

## Troubleshooting

- **No icon appears:** make sure SwiftBar is running and the plugin file
  is in your SwiftBar plugin directory. Check SwiftBar preferences to
  confirm the plugin directory path.
- **Icon shows emoji instead of Clawd:** the `icons/` directory can't be
  found. Ensure it sits next to the plugin script (or that the symlink
  resolves correctly).
- **Gray icon even with active sessions:** verify `~/.claude/mccm/state.json`
  exists and contains session data. Run `mccm install` if you haven't
  set up hooks yet.
