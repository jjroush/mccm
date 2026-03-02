# Feature: Rust Menu Bar Integration (mccm --menubar)

Integrate the macOS menu bar status icon directly into the mccm binary
using Rust's Objective-C FFI bindings.

## Why This Approach

- Single binary: `mccm` handles both TUI and menu bar
- Shares state.rs, session.rs, and notification logic
- No Swift toolchain dependency
- Consistent versioning and distribution

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  mccm binary                                             │
│                                                          │
│  mccm (default)       → run_tui()    [existing]         │
│  mccm install         → install_hooks() [existing]      │
│  mccm menubar         → run_menubar()   [NEW]           │
│  mccm menubar --with-tui → both        [stretch goal]   │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │ run_menubar()                                      │  │
│  │                                                    │  │
│  │  1. Create NSApplication (shared)                  │  │
│  │  2. Create NSStatusItem via NSStatusBar            │  │
│  │  3. Set Clawd icon (NSImage from embedded bytes)   │  │
│  │  4. Spawn notify file watcher on background thread │  │
│  │  5. On state change → update icon tint on main     │  │
│  │     thread via dispatch_async                      │  │
│  │  6. NSApplication::run() — blocks on main thread   │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Key Implementation Details

### 1. Crate Dependencies

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = ["NSString", "NSData"] }
objc2-app-kit = { version = "0.2", features = [
    "NSApplication",
    "NSStatusBar",
    "NSStatusItem",
    "NSImage",
    "NSMenu",
    "NSMenuItem",
    "NSRunningApplication",
] }
```

### 2. NSStatusItem Setup (Rust + objc2)

```rust
use objc2_app_kit::{NSApplication, NSStatusBar, NSStatusItem, NSImage};
use objc2_foundation::NSString;

fn run_menubar() -> anyhow::Result<()> {
    let app = unsafe { NSApplication::sharedApplication() };
    // Hide dock icon
    unsafe { app.setActivationPolicy(NSApplicationActivationPolicyAccessory) };

    let status_bar = unsafe { NSStatusBar::systemStatusBar() };
    let status_item = unsafe {
        status_bar.statusItemWithLength(NSVariableStatusItemLength)
    };

    // Load embedded Clawd icon
    let icon_bytes: &[u8] = include_bytes!("../assets/clawd-template.png");
    let ns_data = unsafe { NSData::withBytes(icon_bytes) };
    let image = unsafe { NSImage::initWithData(NSImage::alloc(), &ns_data) };
    unsafe { image.setTemplate(true) }; // adapts to light/dark menu bar
    unsafe { status_item.button().unwrap().setImage(Some(&image)) };

    // Start file watcher on background thread
    std::thread::spawn(move || {
        watch_state_file(/* callback to update icon */);
    });

    // Run the app event loop (blocks)
    unsafe { app.run() };
    Ok(())
}
```

### 3. Icon Tinting Challenge

NSStatusItem with template images doesn't support arbitrary tint colors
natively. Options:

**Option A: Pre-rendered colored icons**
- Embed 4 PNGs (green/yellow/red/gray Clawd)
- Swap the entire image on status change
- Simplest, most reliable

**Option B: Template image + NSImageView with color filter**
- More complex, fragile across macOS versions

**Option C: Draw into NSImage programmatically**
- Use Core Graphics to composite Clawd shape + color fill
- Most flexible, most code

**Recommendation:** Option A (pre-rendered icons) is the pragmatic choice.

### 4. Thread Safety

The main thread must own the NSApplication run loop. File watching
(using the existing `notify` crate) runs on a background thread. To
update the icon from the background thread:

```rust
use objc2_foundation::MainThreadMarker;
use dispatch::Queue;

// From background thread:
Queue::main().exec_async(move || {
    // Safe to touch NSStatusItem here
    update_icon(new_status);
});
```

Or use `std::sync::mpsc` to send status updates to the main thread,
polled via an NSTimer.

### 5. CLI Integration

```rust
// In main.rs, add to clap subcommands:
#[derive(Subcommand)]
enum Commands {
    Install,
    Uninstall,
    RunTui,       // existing
    Menubar,      // NEW
}

match cli.command {
    Some(Commands::Menubar) => run_menubar()?,
    // ...
}
```

### 6. Dropdown Menu

```rust
fn build_menu(sessions: &[Session]) -> NSMenu {
    let menu = unsafe { NSMenu::new() };

    for session in sessions {
        let title = format!("{} ({})", session.name, session.status);
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(),
                &NSString::from_str(&title),
                None,
                &NSString::from_str(""),
            )
        };
        unsafe { menu.addItem(&item) };
    }

    // Separator + "Open TUI" + "Quit"
    unsafe { menu.addItem(&NSMenuItem::separatorItem()) };
    // ... add action items

    menu
}
```

## Challenges & Risks

| Challenge | Severity | Mitigation |
|-----------|----------|------------|
| objc2 crate API instability | Medium | Pin exact versions, wrap in abstraction layer |
| NSRunLoop blocks main thread | High | All state logic on background thread, dispatch to main for UI |
| Template image tinting | Medium | Use pre-rendered colored PNGs instead |
| Conflicts with TUI mode | Low | Separate subcommand, mutually exclusive |
| macOS version compatibility | Medium | Test on macOS 12-15, use availability checks |
| Binary size increase | Low | ~500KB for AppKit bindings, acceptable |
| Code signing for distribution | Medium | Requires Apple Developer ID for notarization |

## Estimated Effort

- **Core menu bar icon:** 2-3 days (learning objc2 FFI)
- **Dropdown menu with sessions:** +1 day
- **Polish & testing:** +1 day
- **Total:** ~1 week

## Prerequisites

- macOS SDK (comes with Xcode Command Line Tools)
- Clawd icon as template PNG (18x18 @1x, 36x36 @2x)
- Familiarity with Objective-C runtime concepts

## Alternative: tray-icon Crate

The `tray-icon` crate (by tauri team) provides a cross-platform
abstraction. Simpler API but less control:

```rust
use tray_icon::{TrayIconBuilder, Icon};

let icon = Icon::from_rgba(rgba_data, width, height)?;
let tray = TrayIconBuilder::new()
    .with_icon(icon)
    .with_tooltip("mccm")
    .with_menu(menu)
    .build()?;
```

**Pros:** Simpler API, cross-platform
**Cons:** Less native feel, limited macOS-specific features (no
template images), depends on winit event loop
