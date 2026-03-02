# Feature: Native Swift Menu Bar App

A standalone macOS menu bar app that displays a Clawd icon with
green/yellow/red tint based on aggregate Claude Code session health.

## Why Upgrade from SwiftBar

- No dependency on SwiftBar
- Real-time updates via FSEvents (no polling)
- Proper NSStatusItem with pixel-perfect icon rendering
- Richer dropdown UI (SwiftUI popover)
- Can be distributed as a standalone .app bundle

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  ClawdStatus.app (menu bar only, no dock icon)      │
│                                                      │
│  ┌───────────────┐    ┌──────────────────────────┐  │
│  │ FileWatcher    │───>│ StatusResolver            │  │
│  │ (FSEvents on   │    │ - reads state.json        │  │
│  │  state.json)   │    │ - computes aggregate      │  │
│  └───────────────┘    │ - emits: green/yellow/red │  │
│                        └──────────┬───────────────┘  │
│                                   │                   │
│                        ┌──────────▼───────────────┐  │
│                        │ MenuBarController         │  │
│                        │ - NSStatusItem            │  │
│                        │ - Clawd icon (NSImage)    │  │
│                        │ - tint color binding      │  │
│                        └──────────┬───────────────┘  │
│                                   │                   │
│                        ┌──────────▼───────────────┐  │
│                        │ Dropdown Popover (SwiftUI)│  │
│                        │ - Session list            │  │
│                        │ - Status badges           │  │
│                        │ - "Open mccm" button      │  │
│                        │ - "Open in Terminal" per   │  │
│                        │   session                  │  │
│                        └──────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Key Implementation Details

### 1. Project Setup

- SwiftUI App with `@main` and `MenuBarExtra` (macOS 13+)
- Set `LSUIElement = true` in Info.plist (no dock icon)
- Or use legacy NSStatusItem approach for macOS 12 support

```swift
@main
struct CławdStatusApp: App {
    @StateObject private var monitor = SessionMonitor()

    var body: some Scene {
        MenuBarExtra {
            SessionListView(monitor: monitor)
        } label: {
            Image("clawd")
                .renderingMode(.template)
                .foregroundColor(monitor.aggregateColor)
        }
    }
}
```

### 2. File Watching (Real-Time)

```swift
class SessionMonitor: ObservableObject {
    @Published var sessions: [Session] = []
    @Published var aggregateStatus: AggregateStatus = .none

    private var fileDescriptor: Int32 = -1
    private var source: DispatchSourceFileSystemObject?

    func startWatching() {
        let path = NSHomeDirectory() + "/.claude/mccm/state.json"
        fileDescriptor = open(path, O_EVTONLY)

        source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fileDescriptor,
            eventMask: [.write, .rename],
            queue: .main
        )
        source?.setEventHandler { [weak self] in
            self?.reload()
        }
        source?.resume()
    }

    func reload() {
        // Parse state.json, update sessions, recompute aggregate
    }
}
```

### 3. Aggregate Status Logic

```swift
enum AggregateStatus {
    case green   // all active
    case yellow  // at least one inactive, none needs_help
    case red     // at least one needs_help
    case none    // no live sessions
}
```

### 4. Clawd Icon

- Use a single template image (black silhouette on transparent)
- SwiftUI `.foregroundColor()` or NSImage `.isTemplate = true` with tint
- Template images automatically adapt to light/dark menu bar

### 5. Distribution

- Build with `xcodebuild` or `swift build`
- Distribute as .app bundle in DMG or via Homebrew cask
- Or: include a `swift` single-file script built with `swiftc`
  (~150 lines, no Xcode project needed)

## Estimated Effort

- **Minimal (swiftc single file):** ~150 lines, 2-3 hours
- **Full app (Xcode project):** ~300 lines across 4-5 files, half a day
- **With rich dropdown UI:** add another half day

## Prerequisites

- macOS 13+ for MenuBarExtra API (or NSStatusItem for older macOS)
- Swift 5.9+
- Clawd icon as a template PNG asset
