mod app;
mod notification;
mod session;
mod state;
mod ui;

use std::io;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;

#[derive(Parser)]
#[command(name = "mccm", about = "TUI dashboard for Claude Code sessions")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install hooks into ~/.claude/settings.json and set up the hook script
    Install,
    /// Uninstall hooks and clean up
    Uninstall,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => install_hooks(),
        Some(Commands::Uninstall) => uninstall_hooks(),
        None => run_tui(),
    }
}

fn run_tui() -> anyhow::Result<()> {
    // Set up panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new();
    let result = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

const HOOK_SCRIPT: &str = r#"#!/bin/bash
# mccm hook script
# Invoked by Claude Code hooks to track session state.

STATE_DIR="$HOME/.claude/mccm"
STATE_FILE="$STATE_DIR/state.json"

mkdir -p "$STATE_DIR"

# Read hook input from stdin
INPUT=$(cat)
HOOK_EVENT=$(echo "$INPUT" | jq -r '.hook_event_name // empty')
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // empty')
CWD=$(echo "$INPUT" | jq -r '.cwd // empty')
NOTIFICATION_TYPE=$(echo "$INPUT" | jq -r '.notification_type // empty')
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // empty')

# Skip if this is a meta session spawned by mccm for auto-naming
if [ -n "$MCCM_NAMING" ]; then
    exit 0
fi

if [ -z "$SESSION_ID" ]; then
    exit 0
fi

# Determine status based on hook event
case "$HOOK_EVENT" in
    "SessionStart")
        STATUS="active"
        ;;
    "Stop")
        STATUS="inactive"
        ;;
    "Notification")
        STATUS="needs_help"
        ;;
    "SessionEnd")
        STATUS="done"
        ;;
    *)
        exit 0
        ;;
esac

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Initialize state file if it doesn't exist
if [ ! -f "$STATE_FILE" ]; then
    echo '{"sessions":{}}' > "$STATE_FILE"
fi

# Preserve existing name if present
EXISTING_NAME=$(jq -r --arg sid "$SESSION_ID" '.sessions[$sid].name // empty' "$STATE_FILE" 2>/dev/null)

# Atomic update: read, modify, write to temp, rename
TMPFILE=$(mktemp "$STATE_DIR/state.XXXXXX.json")
jq --arg sid "$SESSION_ID" \
   --arg status "$STATUS" \
   --arg ts "$TIMESTAMP" \
   --arg cwd "$CWD" \
   --arg ntype "$NOTIFICATION_TYPE" \
   --arg name "$EXISTING_NAME" \
   '.sessions[$sid] = {
       "status": $status,
       "last_updated": $ts,
       "project_path": $cwd,
       "notification_type": (if $ntype != "" then $ntype else null end),
       "name": (if $name != "" then $name else null end)
   }' "$STATE_FILE" > "$TMPFILE" && mv "$TMPFILE" "$STATE_FILE"

# Auto-name: on first Stop event, generate a name via claude CLI in background
if [ "$HOOK_EVENT" = "Stop" ] && [ -z "$EXISTING_NAME" ] && [ -n "$TRANSCRIPT_PATH" ]; then
    (
        # Unset to allow nested claude CLI usage from hook context
        unset CLAUDECODE

        # Extract the first user prompt from the transcript
        # content may be a plain string or an array of content blocks
        FIRST_PROMPT=$(jq -r '
            select(.message.role == "user")
            | .message.content
            | if type == "array" then
                map(select(.type == "text") | .text) | join(" ")
              else
                tostring
              end' "$TRANSCRIPT_PATH" 2>/dev/null \
            | head -c 500 \
            | head -1)

        if [ -z "$FIRST_PROMPT" ]; then
            exit 0
        fi

        # Generate name via claude CLI (haiku for speed/cost)
        # Export MCCM_NAMING so hooks triggered by this call skip processing
        NAME=$(MCCM_NAMING=1 claude -p --model haiku "Generate a concise 3-5 word title for this coding session. Output ONLY the title, nothing else. No quotes. User's request: $FIRST_PROMPT" 2>/dev/null)

        if [ -n "$NAME" ]; then
            # Write the name back to state.json
            TMPFILE2=$(mktemp "$STATE_DIR/state.XXXXXX.json")
            jq --arg sid "$SESSION_ID" \
               --arg name "$NAME" \
               '.sessions[$sid].name = $name' "$STATE_FILE" > "$TMPFILE2" && mv "$TMPFILE2" "$STATE_FILE"
        fi
    ) &
fi

exit 0
"#;

// SwiftBar plugin and icons embedded in the binary
const SWIFTBAR_PLUGIN: &str = include_str!("../swiftbar/mccm-status.5s.sh");
const ICON_GREEN: &[u8] = include_bytes!("../swiftbar/icons/clawd-green.png");
const ICON_YELLOW: &[u8] = include_bytes!("../swiftbar/icons/clawd-yellow.png");
const ICON_RED: &[u8] = include_bytes!("../swiftbar/icons/clawd-red.png");
const ICON_NONE: &[u8] = include_bytes!("../swiftbar/icons/clawd-none.png");

fn hook_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory must exist")
        .join(".claude")
        .join("mccm")
}

fn hook_script_path() -> PathBuf {
    hook_dir().join("hook.sh")
}

fn swiftbar_dir() -> PathBuf {
    hook_dir().join("swiftbar")
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory must exist")
        .join(".claude")
        .join("settings.json")
}

fn install_hooks() -> anyhow::Result<()> {
    // 1. Create directory and write hook script
    let dir = hook_dir();
    std::fs::create_dir_all(&dir)?;

    let script_path = hook_script_path();
    std::fs::write(&script_path, HOOK_SCRIPT)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("Wrote hook script to {}", script_path.display());

    // 2. Update settings.json
    let settings_file = settings_path();
    let mut settings: serde_json::Value = if settings_file.exists() {
        let content =
            std::fs::read_to_string(&settings_file).context("Reading settings.json")?;
        serde_json::from_str(&content).context("Parsing settings.json")?
    } else {
        serde_json::json!({})
    };

    let hook_cmd = format!("bash {}", script_path.display());

    let hook_entry = serde_json::json!([
        {
            "hooks": [{
                "type": "command",
                "command": hook_cmd
            }]
        }
    ]);

    let hooks = settings
        .as_object_mut()
        .context("settings.json must be an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks.as_object_mut().context("hooks must be an object")?;

    for event in &["SessionStart", "Stop", "Notification", "SessionEnd"] {
        hooks_obj.insert(event.to_string(), hook_entry.clone());
    }

    let settings_str = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_file, settings_str)?;

    println!("Updated {}", settings_file.display());

    // 3. Write SwiftBar plugin and icons
    let sb_dir = swiftbar_dir();
    let sb_icons_dir = sb_dir.join("icons");
    std::fs::create_dir_all(&sb_icons_dir)?;

    let sb_plugin_path = sb_dir.join("mccm-status.5s.sh");
    std::fs::write(&sb_plugin_path, SWIFTBAR_PLUGIN)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&sb_plugin_path, std::fs::Permissions::from_mode(0o755))?;
    }

    std::fs::write(sb_icons_dir.join("clawd-green.png"), ICON_GREEN)?;
    std::fs::write(sb_icons_dir.join("clawd-yellow.png"), ICON_YELLOW)?;
    std::fs::write(sb_icons_dir.join("clawd-red.png"), ICON_RED)?;
    std::fs::write(sb_icons_dir.join("clawd-none.png"), ICON_NONE)?;

    println!("Wrote SwiftBar plugin to {}", sb_dir.display());

    println!("\nInstallation complete! Hooks are now active for new Claude Code sessions.");
    println!("Run `mccm` to launch the dashboard.");
    println!("\nSwiftBar (optional):");
    println!("  ln -s {} <your-swiftbar-plugins-dir>/mccm-status.5s.sh", sb_plugin_path.display());

    Ok(())
}

fn uninstall_hooks() -> anyhow::Result<()> {
    // Remove hooks from settings.json
    let settings_file = settings_path();
    if settings_file.exists() {
        let content = std::fs::read_to_string(&settings_file)?;
        let mut settings: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
            for event in &["SessionStart", "Stop", "Notification", "SessionEnd"] {
                hooks.remove(*event);
            }
            if hooks.is_empty() {
                settings.as_object_mut().unwrap().remove("hooks");
            }
        }

        let settings_str = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&settings_file, settings_str)?;
        println!("Removed hooks from {}", settings_file.display());
    }

    // Remove hook script and state
    let dir = hook_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
        println!("Removed {}", dir.display());
    }

    println!("\nUninstall complete.");
    Ok(())
}
