# mccm — micro claude code manager

A lightweight TUI dashboard and macOS menu bar indicator for monitoring
[Claude Code](https://claude.ai/code) sessions in real time.

## Features

- **TUI dashboard** — terminal UI showing all active Claude Code sessions
  with live status updates (active, inactive, needs help)
- **macOS menu bar icon** — pixel art Clawd icon that changes color based
  on aggregate session health (via SwiftBar plugin)
- **Auto-naming** — sessions are automatically named based on the first
  prompt using the Claude CLI
- **File watching** — state updates in real time as sessions start, stop,
  or request help

## Install

### From GitHub Releases

Download the latest binary from [Releases](https://github.com/jjroush/mccm/releases):

```bash
# Apple Silicon
curl -L https://github.com/jjroush/mccm/releases/latest/download/mccm-v0.3.0-aarch64-apple-darwin.tar.gz | tar xz
sudo mv mccm /usr/local/bin/

# Intel Mac
curl -L https://github.com/jjroush/mccm/releases/latest/download/mccm-v0.3.0-x86_64-apple-darwin.tar.gz | tar xz
sudo mv mccm /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/jjroush/mccm.git
cd mccm
cargo install --path .
```

## Quick Start

```bash
# Install hooks into Claude Code
mccm install

# Launch the TUI dashboard
mccm
```

`mccm install` writes a hook script to `~/.claude/mccm/hook.sh` and
registers it in `~/.claude/settings.json` for the `SessionStart`, `Stop`,
`Notification`, and `SessionEnd` events. After that, every Claude Code
session automatically reports its status to `~/.claude/mccm/state.json`,
which the TUI watches in real time.

## macOS Menu Bar (SwiftBar)

A SwiftBar plugin is included that shows a color-coded Clawd icon in
your menu bar at a glance.

| Icon   | Meaning                           |
|--------|-----------------------------------|
| Green  | All sessions actively working     |
| Yellow | At least one session is inactive  |
| Red    | At least one session needs help   |
| Gray   | No live sessions                  |

### Setup

1. Install SwiftBar and jq:
   ```bash
   brew install --cask swiftbar
   brew install jq
   ```

2. Launch SwiftBar and choose a plugin directory (e.g. `~/swiftbar-plugins/`).

3. Symlink the plugin:
   ```bash
   ln -s /path/to/mccm/swiftbar/mccm-status.5s.sh ~/swiftbar-plugins/mccm-status.5s.sh
   ```

   The icons directory is resolved relative to the script, so the
   symlink picks up the included pixel art icons automatically.

4. A gray Clawd should appear in your menu bar. Start a Claude Code
   session and it turns green.

See [`swiftbar/README.md`](swiftbar/README.md) for customization and
troubleshooting.

## Usage

```
mccm              Launch the TUI dashboard
mccm install      Install Claude Code hooks
mccm uninstall    Remove hooks and clean up
```

## Uninstall

```bash
mccm uninstall
```

This removes the hook script and entries from `~/.claude/settings.json`.

## License

MIT
