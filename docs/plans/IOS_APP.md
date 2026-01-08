# Implementation Plan: iOS Application

## Overview

The iOS application provides native mobile access to ROTT. It implements the same functionality as other clients using Swift and native iOS frameworks, with automerge-swift for data synchronization.

## Goals

1. Native iOS experience following Apple Human Interface Guidelines
2. Local-first with on-device storage
3. Sync with sync server when on private network or VPN
4. Support iPhone and iPad

## Design Principles

- **Native feel** - Use standard iOS patterns and components
- **Local-first** - Full functionality offline
- **Battery efficient** - Smart sync, background handling

## Prerequisites

- Sync server deployed (on private network)
- Xcode and iOS development environment
- Understanding of automerge-swift

## Technology Choices

| Component | Choice |
|-----------|--------|
| Language | Swift |
| UI Framework | SwiftUI (primary), UIKit (where needed) |
| Automerge | automerge-swift |
| Storage | Local filesystem or SQLite |
| Networking | URLSession + WebSocket |

---

## Phase 1: Project Setup and Data Model

### Objective

Set up the Xcode project and implement the core data model.

### Tasks

1. **Project initialization**
   - Create Xcode project (iOS App)
   - Configure bundle identifier, signing
   - Set minimum iOS version (iOS 15+)
   - Set up folder structure

2. **Dependency management**
   - Add automerge-swift via Swift Package Manager
   - Add any other dependencies

3. **Data model definition**
   - Link struct
   - Note struct
   - Tag handling
   - Match schema from Rust implementation

4. **Automerge integration**
   - Initialize Automerge document
   - Define document schema
   - CRUD operations on Automerge document

5. **Unit tests setup**
   - Test target configuration
   - Basic model tests

### Deliverables

- Project builds and runs
- Data model defined
- Automerge document operations work

### Success Criteria

- App launches on simulator/device
- Can create/read/update/delete items in memory
- Unit tests pass

---

## Phase 2: Local Storage

### Objective

Implement persistent storage for Automerge documents.

### Tasks

1. **Storage implementation**
   - Save Automerge document to local filesystem
   - Load document on app launch
   - Handle document not found (first run)

2. **Root document management**
   - Store root document ID in UserDefaults
   - Load root document by ID
   - First-run detection

3. **App lifecycle**
   - Save on app background
   - Load on app foreground
   - Handle low memory warnings

### Deliverables

- Data persists across app restarts
- Root document ID stored

### Success Criteria

- Create item, kill app, reopen, item exists
- Root document ID persists

---

## Phase 3: Core UI - Navigation and Lists

### Objective

Build the main navigation and list views.

### Tasks

1. **App structure**
   - Tab bar or sidebar navigation
   - Main list view
   - Settings view

2. **Tag/topic view**
   - List of all tags
   - Tag selection
   - Filter by tag

3. **Item list view**
   - List of links and notes
   - Search functionality
   - Sort options
   - Pull to refresh

4. **Empty states**
   - No items yet
   - No results for search
   - Onboarding hints

5. **Navigation**
   - List → Detail
   - Swipe gestures
   - Back navigation

### Deliverables

- Navigate between tags and items
- Search and filter work
- Standard iOS navigation

### Success Criteria

- Navigation feels native
- Search is fast
- List handles many items smoothly

---

## Phase 4: Core UI - Detail and Editing

### Objective

Build detail views and editing functionality.

### Tasks

1. **Link detail view**
   - Display all link metadata
   - Open in Safari button
   - Edit button
   - Share button

2. **Note detail view**
   - Render markdown content
   - Edit button
   - Share button

3. **Link editing**
   - Edit form for link metadata
   - Tag selection/editing
   - Save/cancel actions

4. **Note editing**
   - Text editor for markdown
   - Auto-save (debounced)
   - Tag editing

5. **Create new items**
   - Add button in navigation
   - New link flow (URL entry)
   - New note flow

6. **Delete items**
   - Swipe to delete
   - Confirmation alert
   - Undo option (optional)

### Deliverables

- View full item details
- Edit existing items
- Create new items
- Delete items

### Success Criteria

- All CRUD operations work
- Forms validate input
- Editing feels responsive

---

## Phase 5: Link-Specific Features

### Objective

Implement features specific to links.

### Tasks

1. **URL metadata fetching**
   - Fetch title, description from URL
   - Use URLSession
   - Parse HTML meta tags
   - Handle errors gracefully

2. **Open in Safari**
   - In-app Safari view (SFSafariViewController)
   - Open in Safari option

3. **Share extension** (optional)
   - Share sheet target
   - Quick add from other apps
   - Pre-fill URL and title

4. **Quick add from clipboard**
   - Detect URL in clipboard
   - Prompt to add
   - Privacy-conscious (ask first)

### Deliverables

- Metadata fetching works
- Links open in Safari
- Share extension (if implemented)

### Success Criteria

- URL paste fetches metadata
- Safari view works correctly

---

## Phase 6: Note-Specific Features

### Objective

Implement features specific to notes.

### Tasks

1. **Markdown rendering**
   - Use AttributedString or markdown library
   - Handle links in markdown
   - Basic formatting

2. **Markdown editing**
   - Text view for editing
   - Keyboard handling

3. **Auto-save**
   - Save as user types (debounced)
   - Visual save indicator

### Deliverables

- Markdown renders correctly
- Editing is smooth
- Auto-save works

### Success Criteria

- Markdown displays properly
- No data loss during editing
- Keyboard doesn't obscure editor

---

## Phase 7: Sync Implementation

### Objective

Implement synchronization with sync server.

### Tasks

1. **Sync configuration**
   - Sync server URL in settings
   - Manual entry or discovery
   - Enable/disable sync

2. **Network detection**
   - Detect when on sync-capable network
   - WiFi vs cellular consideration
   - VPN detection (if possible)

3. **WebSocket connection**
   - Connect to sync server
   - Use URLSessionWebSocketTask
   - Handle connection lifecycle

4. **Automerge sync**
   - Exchange sync messages
   - Apply remote changes
   - Send local changes
   - Persist sync state

5. **Sync status UI**
   - Connection indicator
   - Last synced time
   - Sync errors

6. **Background sync** (optional)
   - Background app refresh
   - Sync when app wakes

### Deliverables

- Sync works when on network
- Offline changes sync later
- Clear sync status

### Success Criteria

- Changes sync between devices
- Offline edits sync correctly
- User knows sync status

---

## Phase 8: Device Setup Flow

### Objective

Implement the flow for setting up the iOS app.

### Tasks

1. **First-time setup**
   - Detect first run
   - Choice: New or Join existing

2. **New user flow**
   - Generate root document ID
   - Display for user to save
   - Create initial document

3. **Join existing flow**
   - Enter root document ID
   - Scan QR code option
   - Sync root document from server

4. **Settings integration**
   - Show root document ID
   - Share as QR code
   - Copy to clipboard

### Deliverables

- First-run setup works
- Can join existing with root doc ID
- Easy to share root doc ID

### Success Criteria

- Can set up as first device
- Can set up as additional device
- Devices sync correctly

---

## Phase 9: Settings and Polish

### Objective

Complete settings and polish the app.

### Tasks

1. **Settings screen**
   - Sync server configuration
   - Root document ID display
   - Clear local data
   - About/version info

2. **Appearance**
   - Dark mode support (automatic)
   - Dynamic type support
   - App icon

3. **Accessibility**
   - VoiceOver support
   - Dynamic type
   - Accessibility labels

4. **Performance**
   - Profile and optimize
   - Launch time
   - Memory usage

5. **Error handling**
   - User-friendly error messages
   - Retry mechanisms
   - Offline indicators

6. **App Store preparation**
   - Screenshots
   - App description
   - Privacy policy

### Deliverables

- Complete settings
- Polish across the app
- App Store ready

### Success Criteria

- Settings all functional
- Accessibility audit passes
- Ready for TestFlight

---

## App Architecture

### Folder Structure

```
ROTT/
├── App/
│   ├── ROTTApp.swift
│   └── ContentView.swift
├── Models/
│   ├── Link.swift
│   ├── Note.swift
│   └── Document.swift
├── Views/
│   ├── Tags/
│   │   └── TagListView.swift
│   ├── Items/
│   │   ├── ItemListView.swift
│   │   ├── LinkDetailView.swift
│   │   ├── NoteDetailView.swift
│   │   └── ItemRow.swift
│   ├── Forms/
│   │   ├── LinkFormView.swift
│   │   └── NoteFormView.swift
│   └── Settings/
│       └── SettingsView.swift
├── ViewModels/
│   ├── DocumentStore.swift
│   └── SyncManager.swift
├── Services/
│   ├── StorageService.swift
│   ├── SyncService.swift
│   └── MetadataService.swift
└── Resources/
    └── Assets.xcassets
```

### Architecture Pattern

**MVVM with SwiftUI**

- **Model** - Data structures, Automerge document
- **View** - SwiftUI views
- **ViewModel** - ObservableObject classes

---

## iOS-Specific Considerations

1. **Background app refresh** - Register and handle
2. **State restoration** - Preserve state across app kills
3. **Keyboard handling** - Proper avoidance
4. **iPad support** - Sidebar navigation, larger layouts
5. **VPN integration** - Test with Tailscale/WireGuard apps

---

## Estimated Timeline

| Phase | Estimated Duration |
|-------|-------------------|
| Phase 1: Setup and Data Model | 1 week |
| Phase 2: Local Storage | 1 week |
| Phase 3: Core UI - Navigation | 2 weeks |
| Phase 4: Core UI - Detail/Editing | 2 weeks |
| Phase 5: Link Features | 1 week |
| Phase 6: Note Features | 1 week |
| Phase 7: Sync | 2 weeks |
| Phase 8: Device Setup | 1 week |
| Phase 9: Settings and Polish | 2 weeks |

**Total: 13-14 weeks**

---

## Dependencies

| Dependency | Required For | Status |
|------------|--------------|--------|
| Sync Server | Sync feature | Develop local-first first |
| Apple Developer Account | TestFlight/App Store | Required for distribution |
| automerge-swift | Core data handling | Available |

---

## Open Questions

1. **SwiftUI vs UIKit** - Full SwiftUI or hybrid?
2. **Minimum iOS version** - iOS 15? 16? 17?
3. **iPad-first or iPhone-first** - Where to focus?
4. **Share extension** - Essential or nice-to-have?
5. **QR code scanning** - For root document ID setup?
