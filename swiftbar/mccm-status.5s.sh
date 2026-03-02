#!/bin/bash
# mccm-status.5s.sh — SwiftBar plugin for mccm session status
#
# Displays a Clawd (Claude mascot) icon in the macOS menu bar,
# tinted by aggregate session health:
#   Green  = all sessions actively working
#   Yellow = at least one session is inactive (paused/stopped)
#   Red    = at least one session needs help
#   Gray   = no live sessions
#
# Install:
#   1. Install SwiftBar: brew install --cask swiftbar
#   2. Symlink or copy this file into your SwiftBar plugin directory
#   3. (Optional) Place Clawd icon PNGs in the icons/ directory next to this script

STATE="$HOME/.claude/mccm/state.json"
ICON_DIR="$HOME/.claude/mccm/swiftbar/icons"

# --- Resolve aggregate status ---
if [ ! -f "$STATE" ]; then
  STATUS="none"
else
  needs_help=$(jq '[.sessions[] | select(.status == "needs_help")] | length' "$STATE" 2>/dev/null)
  active=$(jq '[.sessions[] | select(.status == "active")] | length' "$STATE" 2>/dev/null)
  inactive=$(jq '[.sessions[] | select(.status == "inactive")] | length' "$STATE" 2>/dev/null)

  if [ "${needs_help:-0}" -gt 0 ]; then
    STATUS="red"
  elif [ "${inactive:-0}" -gt 0 ]; then
    STATUS="yellow"
  elif [ "${active:-0}" -gt 0 ]; then
    STATUS="green"
  else
    STATUS="none"
  fi
fi

# --- Render menu bar icon ---
# If a Clawd PNG exists for this status, use it (base64-encoded).
# SwiftBar expects 18x18 or 16x16 @2x PNGs for crisp menu bar icons.
ICON_FILE="${ICON_DIR}/clawd-${STATUS}.png"

if [ -f "$ICON_FILE" ]; then
  ICON_B64=$(base64 < "$ICON_FILE")
  echo "| image=${ICON_B64}"
else
  # Fallback: colored circle emoji
  case "$STATUS" in
    red)    echo "🔴" ;;
    yellow) echo "🟡" ;;
    green)  echo "🟢" ;;
    *)      echo "⚪" ;;
  esac
fi

echo "---"

# --- Dropdown: session details ---
if [ ! -f "$STATE" ]; then
  echo "No state file found"
  echo "Run: mccm install | color=gray"
  exit 0
fi

# Count by status
echo "Sessions"
echo "--Active: ${active:-0} | color=green"
echo "--Inactive: ${inactive:-0} | color=orange"
echo "--Needs Help: ${needs_help:-0} | color=red"
echo "---"

# List individual sessions
jq -r '.sessions | to_entries[] | select(.value.status != "done") | "\(.value.name // .key)|\(.value.status)|\(.value.project_path // "unknown")"' "$STATE" 2>/dev/null | \
while IFS='|' read -r name status project; do
  project_name=$(basename "$project")
  case "$status" in
    active)     color="green"  ; badge="working" ;;
    inactive)   color="orange" ; badge="paused" ;;
    needs_help) color="red"    ; badge="needs help" ;;
    *)          color="gray"   ; badge="$status" ;;
  esac
  echo "${name} (${badge}) — ${project_name} | color=${color}"
done

echo "---"
echo "Open mccm | bash=mccm | terminal=true"
echo "Refresh | refresh=true"
