# mccm SwiftBar Plugin

A macOS menu bar indicator showing aggregate Claude Code session health
via a Clawd (Claude mascot) icon.

## Setup

1. **Install SwiftBar**
   ```bash
   brew install --cask swiftbar
   ```

2. **Launch SwiftBar** and choose a plugin directory when prompted
   (e.g., `~/swiftbar-plugins/`).

3. **Symlink the plugin** into that directory:
   ```bash
   ln -s "$(pwd)/mccm-status.5s.sh" ~/swiftbar-plugins/mccm-status.5s.sh
   ```

4. **(Optional) Add Clawd icons** — see `icons/README.md` for specs.

## How It Works

The plugin reads `~/.claude/mccm/state.json` (written by mccm hooks)
every 5 seconds and derives an aggregate status:

| Icon   | Meaning                               |
|--------|---------------------------------------|
| Green  | All sessions are actively working     |
| Yellow | At least one session is inactive      |
| Red    | At least one session needs help       |
| Gray   | No live sessions                      |

Clicking the icon shows a dropdown with per-session details and a
shortcut to launch the mccm TUI.

## Customization

- **Refresh interval:** rename the file (e.g., `mccm-status.3s.sh` for 3 seconds)
- **Icons:** drop Clawd PNGs in `icons/` — see `icons/README.md`
