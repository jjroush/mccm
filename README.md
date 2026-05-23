# mccm — micro claude code manager

A lightweight TUI dashboard and macOS menu bar indicator for monitoring
[Claude Code](https://claude.ai/code) sessions in real time.

## Features

- **TUI dashboard** — terminal UI showing all active Claude Code sessions
  with live status updates (active, inactive, needs help)
- **macOS menu bar icon** — native menu bar daemon showing a pixel art
  Clawd icon that changes color based on aggregate session health;
  auto-starts at login via launchd
- **Auto-naming** — sessions are automatically named based on the first
  prompt using the Claude CLI
- **File watching** — state updates in real time as sessions start, stop,
  or request help

## Install

### Homebrew

```bash
brew install jjroush/tap/mccm
```

### From GitHub Releases

Download the latest binary from [Releases](https://github.com/jjroush/mccm/releases):

```bash
# Apple Silicon
curl -L https://github.com/jjroush/mccm/releases/latest/download/mccm-v0.5.1-aarch64-apple-darwin.tar.gz | tar xz
sudo mv mccm /usr/local/bin/

# Intel Mac
curl -L https://github.com/jjroush/mccm/releases/latest/download/mccm-v0.5.1-x86_64-apple-darwin.tar.gz | tar xz
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

## macOS Menu Bar

`mccm install` sets up a native macOS menu bar daemon as a launchd
LaunchAgent. It starts automatically at login and stays in your menu
bar with a color-coded Clawd icon.

| Icon   | Meaning                           |
|--------|-----------------------------------|
| Green  | All sessions actively working     |
| Yellow | At least one session is inactive  |
| Red    | At least one session needs help   |
| Gray   | No live sessions                  |

Clicking the icon shows session counts, a list of live sessions, and
an "Open mccm" entry that launches the TUI in a new Terminal window.

Logs are at `~/.claude/mccm/menubar.{log,err}`. The LaunchAgent plist
lives at `~/Library/LaunchAgents/io.roush.mccm.menubar.plist`.

## Usage

```
mccm              Launch the TUI dashboard
mccm install      Install hooks and start the menu bar daemon
mccm uninstall    Remove hooks, unload the menu bar daemon, clean up
mccm menubar      Run the menu bar daemon in the foreground (debugging)
```

## Uninstall

```bash
mccm uninstall
```

This unloads the LaunchAgent, removes its plist, removes the hook
script, and clears entries from `~/.claude/settings.json`.

## License

MIT
