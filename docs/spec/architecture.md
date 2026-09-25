# Architecture and platforms

[Specification index](README.md)

## Required architecture

ROTT must be a new application with a fresh codebase. Existing Rust ROTT is neither its starting point nor a required reference; evaluating legacy code reuse is not a prerequisite.

Start with one reusable Rust core library/crate, internally organized around accounts, downloading, saved items, publishing, storage, and sync. These are responsibility areas, not selected module names or signatures. There is no upfront split into multiple domain libraries. Platform bridge and build glue may live separately.

| Boundary | Responsibility |
| --- | --- |
| Rust core | Domain/data operations, saving/tagging/private notes, follows and publishing/reply/Share API operations, feed fetching/parsing/caching, filtered post/content/reply access, Automerge persistence and sync |
| Platform UI | Presentation, navigation, editors, and visibly distinct private/public actions |
| Platform adapters | Secure credentials, browser/OS integration as needed, lifecycle integration, and bindings to the core |
| SlugKit and website | Shared semantic API and canonical website publication; optional backend storage/federation and authoritative social state |

All interfaces must share platform-independent site integration through the core. ROTT must call the SlugKit API directly; it must not invoke the CLI as a subprocess or implement a separate federation layer. Actor signing keys remain on the website.

## Platforms and bindings

| Target | Decision |
| --- | --- |
| macOS | Native Swift/SwiftUI over the compiled Rust core; first focus |
| iPhone | Required native Swift/SwiftUI application over the core; first focus |
| Windows and Linux | Required after Mac/iPhone; toolkit deferred |
| Future TUI | Hypothetical direct Rust consumer, not a committed feature |

No ordering within either platform pair or release date is selected. Each target requires its own build and packaging; shared logic does not require identical interfaces or binaries. C#/WinUI is not required for Windows. Slint and Iced are candidate Windows/Linux toolkits; GTK4/libadwaita is a candidate GNOME-specific alternative. None is selected.

The core must remain independent of any particular bridge. Each consuming application supplies appropriate adapters/bindings, including thin Rust export glue if needed. Apple applications may share Swift bridge code while compiling the core for each target. Rust callers can call it directly; other languages use suitable bridges. No universal multi-language bridge is required.

UniFFI and swift-bridge are candidates without a selected preference. Implementation must address asynchronous operations/cancellation, UI events/change notifications, errors, ownership/lifetimes, platform services, and signatures. No code generator or shared TypeScript client package is selected.

## iPhone design boundary

The core must assume neither a three-pane UI nor a continuously running process. iPhone navigation, layout, lifecycle, background scheduling and device-sync policy are deferred to iPhone application design. Manual website Download remains required independently of optional device sync. Desktop-style continuous background sync must not be assumed on iPhone.

## Coexistence and packaging

The current Rust CLI/TUI must remain isolated and usable. The new application must not write into its data or assume permission to change its schema. Distinct development commands, configuration/data directories, Automerge documents and sync identity are proposed isolation mechanisms; exact names and paths are not selected.

A monorepo is recommended but not finalized. Publishing the core as a standalone package is possible, not required. Forgejo hosting is under consideration; repository migration is not authorized.

If old data migration is later pursued, use a separate helper script/application built on ROTT facilities. A built-in importer is not required for the initial application, and no importer is authorized now.
