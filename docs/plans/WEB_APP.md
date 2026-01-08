# Implementation Plan: Web Application

## Overview

The web application is a TypeScript-based browser application that provides access to ROTT from any web browser. It implements the same functionality as the CLI/TUI but with a web interface.

## Goals

1. Full ROTT functionality in the browser
2. Local-first with IndexedDB storage
3. Sync through the web server relay
4. Responsive design for desktop and mobile browsers

## Design Principles

- **Local-first** - Works offline, syncs when connected
- **Native feel** - Fast, responsive, no page reloads
- **Simple auth** - Login through web server, then use the app

## Prerequisites

- Web server deployed (for authentication and relay)
- Sync server deployed (on private network)

## Technology Choices

| Component | Recommended Options |
|-----------|---------------------|
| Framework | React, Vue, Svelte, or SolidJS |
| Language | TypeScript |
| State Management | Framework-native or Zustand/Jotai |
| Styling | Tailwind CSS or CSS Modules |
| Automerge | automerge-js, @automerge/automerge-repo |
| Storage | IndexedDB (via idb or Dexie) |
| Build Tool | Vite |

---

## Phase 1: Project Setup and Authentication

### Objective

Set up the web application with authentication flow.

### Tasks

1. **Project initialization**
   - Create project with Vite + chosen framework
   - Configure TypeScript
   - Set up linting (ESLint) and formatting (Prettier)

2. **Basic routing**
   - Login page
   - Main application (protected)
   - Settings page

3. **Authentication integration**
   - Login form (sends to web server)
   - Handle session cookie
   - Protected route wrapper
   - Logout functionality

4. **Root document ID setup**
   - First-time prompt for root document ID
   - Store in localStorage
   - Settings page to view/change

### Deliverables

- User can log in through web server
- Root document ID is stored
- Authenticated routes are protected

### Success Criteria

- Login flow works end-to-end
- Session persists across page reloads
- Cannot access app without login

---

## Phase 2: Data Model and Local Storage

### Objective

Implement data model and local persistence using Automerge and IndexedDB.

### Tasks

1. **Automerge setup**
   - Install automerge-js and @automerge/automerge-repo
   - Initialize Automerge repo with IndexedDB storage
   - Connect to WebSocket (through web server relay)

2. **Root document handling**
   - Load root document by ID from localStorage
   - Create Automerge document handle
   - Subscribe to changes

3. **Data model implementation**
   - Link: title, source, author, dates, description, tags
   - Note: title, body, dates, tags
   - CRUD operations on Automerge document

4. **IndexedDB storage**
   - Configure automerge-repo IndexedDB adapter
   - Documents persist locally
   - Works offline

5. **Reactivity**
   - Subscribe to Automerge changes
   - Update UI when data changes
   - Optimistic updates

### Deliverables

- Data persists in IndexedDB
- CRUD operations work
- Changes sync through relay

### Success Criteria

- Create item, refresh page, item persists
- Changes sync to other devices
- Works offline

---

## Phase 3: Core UI Components

### Objective

Build the main user interface components.

### Tasks

1. **Layout**
   - App shell (header, sidebar, main content)
   - Responsive design (mobile-friendly)
   - Navigation

2. **Tag/topic sidebar**
   - List all tags
   - Tag selection
   - Tag counts
   - Search/filter tags

3. **Item list view**
   - List links and notes
   - Filter by selected tag
   - Search within items
   - Sort options (date, title)

4. **Item detail view**
   - View link/note details
   - Open link in new tab
   - Edit button

5. **Create/edit forms**
   - Create new link (URL input with metadata fetch)
   - Create new note (markdown editor)
   - Edit existing items
   - Tag selection/creation

6. **Delete confirmation**
   - Confirmation dialog
   - Undo option (optional)

### Deliverables

- Complete UI for browsing and managing items
- Responsive layout
- All CRUD operations accessible

### Success Criteria

- Can perform all operations via UI
- Works on mobile and desktop
- No visual glitches

---

## Phase 4: Link-Specific Features

### Objective

Implement features specific to links.

### Tasks

1. **URL metadata fetching**
   - Fetch title, description, author from URL
   - Handle via web server proxy (CORS)
   - Handle timeout and errors
   - Preview before saving

2. **Link opening**
   - Open in new tab
   - Visual indicator for external links

3. **Quick add**
   - Keyboard shortcut to add link
   - Paste URL and auto-fetch
   - Minimal friction flow

### Deliverables

- Metadata fetching works
- Quick link addition flow
- Links open correctly

### Success Criteria

- Pasting URL fetches metadata
- Metadata appears within a few seconds
- Failed fetch doesn't block saving

---

## Phase 5: Note-Specific Features

### Objective

Implement features specific to notes.

### Tasks

1. **Markdown editor**
   - Simple textarea or rich editor
   - Preview mode (or live preview)
   - Basic formatting toolbar (optional)

2. **Note rendering**
   - Render markdown to HTML
   - Syntax highlighting for code blocks
   - Safe HTML (sanitize)

3. **Auto-save**
   - Save as user types (debounced)
   - Save indicator
   - No explicit save button needed

4. **Full-screen editing** (optional)
   - Distraction-free mode
   - Keyboard shortcut to toggle

### Deliverables

- Markdown editing and preview
- Auto-save working
- Pleasant writing experience

### Success Criteria

- Markdown renders correctly
- Changes save without explicit action
- Editor is responsive

---

## Phase 6: Sync and Offline

### Objective

Ensure sync works well and offline is seamless.

### Tasks

1. **Sync status UI**
   - Connection indicator
   - Last synced timestamp
   - Sync errors

2. **Offline detection**
   - Detect when offline
   - Visual indicator
   - Queue changes locally

3. **Reconnection**
   - Auto-reconnect when online
   - Sync queued changes
   - Handle sync conflicts (Automerge automatic)

4. **Initial sync**
   - Loading state while first sync
   - Progress indicator for large syncs

### Deliverables

- Clear sync status
- Works offline seamlessly
- Reconnection works

### Success Criteria

- Can edit offline
- Changes sync when reconnected
- User knows sync status

---

## Phase 7: Settings and Polish

### Objective

Add settings and polish the application.

### Tasks

1. **Settings page**
   - View root document ID
   - Change root document ID
   - Clear local data
   - Logout

2. **Keyboard shortcuts**
   - Navigation (j/k, arrows)
   - Actions (n for new, etc.)
   - Help overlay showing shortcuts

3. **Theme support**
   - Light/dark mode
   - System preference detection
   - Manual toggle

4. **Performance optimization**
   - Virtual scrolling for large lists
   - Lazy loading
   - Bundle optimization

5. **Error handling**
   - User-friendly error messages
   - Retry mechanisms
   - Offline error handling

6. **Accessibility**
   - Keyboard navigation
   - Screen reader support
   - Focus management

### Deliverables

- Settings page complete
- Keyboard shortcuts work
- Theme support
- Polished UI

### Success Criteria

- All settings functional
- Keyboard users can navigate fully
- Works in light and dark mode

---

## Application Architecture

### Component Structure

```
src/
├── components/
│   ├── layout/
│   │   ├── AppShell
│   │   ├── Sidebar
│   │   └── Header
│   ├── items/
│   │   ├── ItemList
│   │   ├── ItemCard
│   │   ├── LinkDetail
│   │   └── NoteDetail
│   ├── forms/
│   │   ├── LinkForm
│   │   ├── NoteForm
│   │   └── TagSelector
│   └── common/
│       ├── Button
│       ├── Input
│       └── Modal
├── pages/
│   ├── Login
│   ├── Home
│   └── Settings
├── services/
│   ├── auth.ts
│   ├── repo.ts       # Automerge repo setup
│   └── metadata.ts   # URL metadata fetching
├── stores/
│   ├── auth.ts
│   ├── documents.ts
│   └── ui.ts
└── types/
    ├── link.ts
    └── note.ts
```

### Data Flow

```
User Interaction
       │
       ▼
   UI Component
       │
       ▼
   Store/State
       │
       ▼
 Automerge Repo ──────► IndexedDB (local)
       │
       ▼
 WebSocket ──────► Web Server ──────► Sync Server
```

---

## Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| IndexedDB | ✅ | ✅ | ✅ | ✅ |
| WebSocket | ✅ | ✅ | ✅ | ✅ |
| ES2020+ | ✅ | ✅ | ✅ | ✅ |

Target: Modern browsers from last 2 years.

---

## Estimated Timeline

| Phase | Estimated Duration |
|-------|-------------------|
| Phase 1: Setup and Auth | 1 week |
| Phase 2: Data Model and Storage | 1-2 weeks |
| Phase 3: Core UI | 2-3 weeks |
| Phase 4: Link Features | 1 week |
| Phase 5: Note Features | 1 week |
| Phase 6: Sync and Offline | 1 week |
| Phase 7: Settings and Polish | 1-2 weeks |

**Total: 8-12 weeks**

---

## Dependencies

| Dependency | Required For | Status |
|------------|--------------|--------|
| Web Server | Authentication, relay | Must be deployed |
| Sync Server | Document sync | Must be deployed |

---

## Open Questions

1. **Framework choice** - React, Vue, Svelte, or SolidJS?
2. **Markdown editor** - Simple textarea, CodeMirror, or other?
3. **URL metadata proxy** - Build into web server or separate service?
4. **PWA** - Add service worker for better offline?
