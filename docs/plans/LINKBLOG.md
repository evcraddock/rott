# Implementation Plan: Linkblog

## Overview

The linkblog is a public-facing website that displays links from ROTT tagged as "public". It provides a way to share curated links with the world, with each link having its own page and a generated slug for shareable URLs.

This is the first web-facing component of ROTT and lays the groundwork for the authenticated web application later.

## Goals

1. Replace existing linkblog with ROTT-powered version
2. Public read-only access (no authentication required)
3. Individual link pages with shareable URLs (slugs)
4. RSS feed for subscribers
5. Real-time updates via sync

## Design Principles

- **Public by default: nothing** — Only links explicitly tagged are shown
- **Simple and fast** — Static-like performance, minimal JavaScript
- **Sync-powered** — Updates automatically when you save in ROTT
- **Foundation for more** — Authentication and private app come later

## Prerequisites

- Sync server deployed
- ROTT CLI working with sync

---

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐
│  ROTT CLI   │◄───────►│ Sync Server  │◄───────►│   Linkblog   │
│  (private)  │  sync   │              │  sync   │   (public)   │
└─────────────┘         └──────────────┘         └──────────────┘
                                                        │
                                                        ▼
                                                 Public website
                                                 yoursite.com/links
```

The linkblog:
1. Connects to sync server
2. Maintains local copy of Automerge document
3. Projects to SQLite for fast queries
4. Serves public HTTP routes

---

## Phase 0: Data Model Changes (rott-core)

### Objective

Add slug support to the Link model for URL-friendly paths.

### Tasks

1. **Add slug field to Link model**
   - Optional string field for URL-friendly path

2. **Slug generation utility**
   - Convert title to URL-friendly slug
   - Handle unicode, special characters
   - Ensure uniqueness (append number if needed)

3. **Update Automerge document handling**
   - Read/write slug field
   - Migration for existing documents (slug = None)

4. **Update SQLite projection**
   - Add slug column
   - Index for fast lookup

5. **CLI integration**
   - `rott link create` auto-generates slug
   - `rott link edit` can modify slug
   - `rott link show` displays slug

### Deliverables

- Links can have slugs
- Slugs are synced across devices
- CLI can set/show slugs

### Success Criteria

- Create link, slug is auto-generated
- Slug syncs to other devices
- Can look up link by slug

---

## Phase 1: Project Setup

### Objective

Set up the linkblog web server project.

### Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust | Reuse rott-core, single binary |
| Web Framework | Axum | Modern, async, good ecosystem |
| Templates | Askama or Tera | Compile-time or runtime templates |
| Styling | Tailwind CSS | Matches existing site, utility-first |

### Tasks

1. **Create new crate**
   - New workspace member: `rott-linkblog`

2. **Dependencies**
   - `rott-core` (workspace dependency)
   - `axum` for web server
   - `tokio` for async runtime
   - `askama` or `tera` for templates
   - `tower-http` for middleware (compression, etc.)

3. **Configuration**
   - Sync server URL
   - Listen address/port
   - Public tag name (default: "public")
   - Site title, description, base URL

4. **Basic server**
   - Health check endpoint
   - Static file serving (CSS, etc.)
   - Graceful shutdown

### Deliverables

- Server runs and responds to requests
- Configuration system working

### Success Criteria

- `cargo run` starts server
- Health check returns 200
- Can configure via file/env

---

## Phase 2: Sync Integration

### Objective

Connect to sync server and maintain local document copy.

### Tasks

1. **Reuse sync client from rott-core**
   - Connect to sync server on startup
   - Handle reconnection

2. **Local document storage**
   - Store Automerge document locally
   - Persist across restarts

3. **SQLite projection**
   - Reuse projection logic from rott-core
   - Filter for public links only
   - Index by slug, created_at

4. **Change detection**
   - Subscribe to document changes
   - Re-project when document updates
   - Invalidate any caches

### Deliverables

- Linkblog syncs document from server
- Local SQLite has public links
- Updates appear automatically

### Success Criteria

- Start linkblog, document syncs
- Add public link via CLI, appears on site
- Remove public tag, disappears from site

---

## Phase 3: Core Routes

### Objective

Implement the main public routes.

### Routes

| Route | Description |
|-------|-------------|
| `GET /` or `GET /links` | Paginated list of links |
| `GET /:slug` | Individual link page |
| `GET /tag/:tag` | Links filtered by tag |
| `GET /feed.xml` | RSS feed |

### Tasks

1. **Link list page (`/`)**
   - Paginated list (10-20 per page)
   - Sorted by created_at descending
   - Shows: title, date, excerpt, author, tags
   - Pagination controls

2. **Link detail page (`/:slug`)**
   - Full link details
   - Title links to source URL
   - Description/notes displayed
   - Author and date
   - Tags (link to tag pages)
   - Meta tags for social sharing (Open Graph, Twitter)

3. **Tag page (`/tag/:tag`)**
   - Same as list but filtered
   - Shows tag name
   - Count of links

4. **RSS feed (`/feed.xml`)**
   - Standard RSS 2.0 or Atom
   - Recent N links
   - Full content or excerpt

5. **404 handling**
   - Custom 404 page
   - Suggest similar slugs (optional)

### Deliverables

- All routes working
- Pages render correctly
- RSS feed validates

### Success Criteria

- Can browse links on site
- Individual link pages work
- RSS feed works in reader

---

## Phase 4: Templates and Styling

### Objective

Create the HTML templates and styling.

### Tasks

1. **Base layout**
   - Header (site title, navigation)
   - Main content area
   - Footer

2. **Component templates**
   - Link card (for list view)
   - Link detail
   - Pagination
   - Tag list

3. **Styling**
   - Tailwind CSS setup
   - Dark/light mode (respect system preference)
   - Responsive design
   - Match existing site aesthetic (or new design)

4. **Static assets**
   - CSS (compiled Tailwind)
   - Favicon
   - Optional: minimal JS for theme toggle

### Deliverables

- Polished, responsive design
- Works on mobile and desktop
- Dark mode support

### Success Criteria

- Looks good on phone and desktop
- Matches desired aesthetic
- Fast page loads

---

## Phase 5: SEO and Performance

### Objective

Optimize for search engines and performance.

### Tasks

1. **SEO basics**
   - Semantic HTML
   - Meta descriptions
   - Open Graph tags
   - Twitter card tags
   - Canonical URLs

2. **Structured data**
   - JSON-LD for articles/links
   - Breadcrumbs

3. **Performance**
   - Gzip/Brotli compression
   - Cache headers for static assets
   - Minimal CSS (purge unused Tailwind)
   - No blocking JavaScript

4. **Sitemap**
   - `GET /sitemap.xml`
   - All public link URLs

5. **robots.txt**
   - Allow all public routes

### Deliverables

- Site ranks well in search
- Fast load times
- Proper social previews

### Success Criteria

- Lighthouse score > 90
- Social sharing shows preview
- Google can index pages

---

## Phase 6: Deployment

### Objective

Deploy the linkblog to production.

### Tasks

1. **Docker image**
   - Multi-stage build (compile + minimal runtime)
   - Single binary + static assets
   - Health check

2. **Docker Compose integration**
   - Add to existing compose file
   - Connect to sync server network
   - Expose on appropriate port

3. **Reverse proxy configuration**
   - nginx/Caddy config
   - SSL termination
   - Cache static assets

4. **Domain setup**
   - DNS configuration
   - SSL certificate (Let's Encrypt)
   - Redirect www if needed

5. **Documentation**
   - Deployment guide
   - Configuration reference
   - Troubleshooting

### Deliverables

- Production deployment working
- Documentation complete

### Success Criteria

- Site accessible at public URL
- SSL working
- Syncs from ROTT automatically

---

## Configuration Reference

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ROOT_DOC_ID` | Yes | — | Automerge root document ID to sync |
| `SYNC_URL` | Yes | — | Sync server WebSocket URL |
| `LISTEN_ADDR` | No | `0.0.0.0:3000` | Address to listen on |
| `DATA_DIR` | No | `./data` | Local storage directory |
| `PUBLIC_TAG` | No | `public` | Tag that marks links as public |
| `SITE_TITLE` | No | `Links` | Site title |
| `SITE_DESCRIPTION` | No | — | Site meta description |
| `BASE_URL` | Yes | — | Public URL (for RSS, sitemap) |

Get your root document ID from the CLI with `rott device show`.

---

## Future Considerations

These are explicitly out of scope for now but inform the design:

1. **Authentication** — Will be added later for private web app
2. **Write operations** — Linkblog is read-only; editing via CLI
3. **Comments** — Could be added for community features
4. **Submissions** — Others submitting links (moderated)
5. **Voting/ranking** — HN-style community features

The architecture should not preclude these, but they are not part of this plan.

---

## Estimated Timeline

| Phase | Estimated Duration |
|-------|-------------------|
| Phase 0: Data Model (slug) | 0.5-1 week |
| Phase 1: Project Setup | 0.5-1 week |
| Phase 2: Sync Integration | 1 week |
| Phase 3: Core Routes | 1-2 weeks |
| Phase 4: Templates/Styling | 1-2 weeks |
| Phase 5: SEO/Performance | 0.5-1 week |
| Phase 6: Deployment | 0.5-1 week |

**Total: 5-9 weeks**

---

## Dependencies

| Dependency | Required For | Status |
|------------|--------------|--------|
| Sync server | Document sync | Must be deployed |
| Slug support in rott-core | URL-friendly paths | Phase 0 |

---

## Relationship to Other Plans

- **SYNC_SERVER.md** — Linkblog connects to this for document sync
- **WEB_SERVER.md** — Linkblog may later merge with this (add auth layer)
- **WEB_APP.md** — Private app routes added on top of linkblog

### Migration Path

Linkblog (this plan) → Add authentication (from WEB_SERVER.md) → Add private routes (from WEB_APP.md) → Unified web application

---

## Open Questions

1. **Slug generation** — Auto from title, or require manual entry?
2. **Template engine** — Askama (compile-time) vs Tera (runtime)?
3. **Styling** — Match current site exactly, or fresh design?
4. **Domain** — Same domain as blog, or separate?
5. **Notes display** — Show all notes, first note only, or excerpt?
