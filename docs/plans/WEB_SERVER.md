# Implementation Plan: Web Server

## Overview

The web server provides authenticated access to the sync server from the internet. It serves the web application and relays WebSocket traffic to the sync server on the private network.

## Goals

1. Serve the web application (HTML/JS/CSS)
2. Authenticate users before allowing access
3. Relay WebSocket traffic to the sync server
4. Keep the sync server unexposed to the internet

## Design Principles

- **Authentication gateway** - Only authenticated users reach the sync server
- **Simple relay** - No application logic, just forward traffic
- **Minimal state** - Only store authentication credentials

---

## Architecture

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Browser    │◄───────►│  Web Server  │◄───────►│ Sync Server  │
│  (Internet)  │  HTTPS  │  (DMZ or     │  Local  │  (Private)   │
│              │         │   exposed)   │         │              │
└──────────────┘         └──────────────┘         └──────────────┘
```

The web server has two jobs:
1. Authenticate users (password, passkey, etc.)
2. Proxy WebSocket connections to the sync server

---

## Phase 1: Project Setup

### Objective

Set up the web server project with basic HTTP serving.

### Tasks

1. **Choose technology stack**
   - Recommended: Node.js with Express or Fastify
   - Alternatives: Go, Rust, Python
   - Needs: Static file serving, WebSocket proxying, session management

2. **Project initialization**
   - Set up project structure
   - Configure TypeScript (if using Node.js)
   - Set up linting and formatting

3. **Static file serving**
   - Serve the web application files
   - Configure for production (caching, compression)

4. **Configuration**
   - Sync server URL (internal)
   - Port to listen on
   - Session secret
   - Admin credentials

### Deliverables

- Server runs and serves static files
- Configuration system working

### Success Criteria

- Can access web app in browser
- Configuration can be changed without code changes

---

## Phase 2: Authentication

### Objective

Implement user authentication.

### Tasks

1. **Choose authentication method**

   **Option A: Simple Password (Recommended for personal use)**
   - Single shared password configured in environment
   - Session cookie after login
   - Simplest to implement

   **Option B: User Accounts**
   - Multiple users with individual passwords
   - User database (SQLite)
   - More complex but supports multiple users

   **Option C: Passkey (WebAuthn)**
   - Most secure
   - No password to remember
   - Requires more implementation effort

2. **Implement login flow**
   - Login page
   - Password verification
   - Session creation
   - Redirect to app

3. **Session management**
   - Secure session cookies
   - Session expiration
   - Logout functionality

4. **Protected routes**
   - All app routes require authentication
   - Redirect to login if not authenticated

### Deliverables

- Login page works
- Sessions persist across requests
- Unauthenticated users cannot access app

### Success Criteria

- Can log in with password
- Session survives page reload
- Logout works

---

## Phase 3: WebSocket Relay

### Objective

Proxy WebSocket connections to the sync server.

### Tasks

1. **WebSocket endpoint**
   - Accept WebSocket connections at `/sync` or similar
   - Require authentication (valid session)

2. **Connect to sync server**
   - Open WebSocket to internal sync server
   - Handle connection errors

3. **Bidirectional relay**
   - Forward messages from browser to sync server
   - Forward messages from sync server to browser
   - Handle binary data (Automerge uses binary)

4. **Connection management**
   - Handle disconnections gracefully
   - Reconnect to sync server if needed
   - Clean up resources on disconnect

### Deliverables

- WebSocket relay works end-to-end
- Browser can sync through the relay

### Success Criteria

- Automerge sync works through relay
- Connection drop on either end handled gracefully

---

## Phase 4: Root Document Management

### Objective

Handle user's root document ID for the web app.

### Tasks

1. **Root document ID storage**
   - User enters root document ID on first use
   - Store in browser localStorage (client-side)
   - Optionally store on server per user (if multi-user)

2. **First-time setup flow**
   - After login, check if root document ID is set
   - Prompt to enter if not
   - Validate format

3. **Settings page**
   - View current root document ID
   - Change root document ID
   - Clear local data

### Deliverables

- Root document ID management works
- User can set up web access to their documents

### Success Criteria

- Can enter root document ID
- Web app syncs user's documents

---

## Phase 5: Production Readiness

### Objective

Prepare for deployment.

### Tasks

1. **Security hardening**
   - HTTPS (required for WebAuthn, recommended for all)
   - Secure cookie settings
   - Rate limiting on login
   - CORS configuration

2. **Deployment**
   - Dockerfile
   - Docker Compose with web app and server together
   - Environment variable configuration

3. **Reverse proxy setup**
   - nginx or Caddy configuration
   - SSL termination
   - WebSocket proxying

4. **Documentation**
   - Deployment guide
   - Configuration reference
   - Troubleshooting

### Deliverables

- Production-ready deployment
- Documentation for self-hosting

### Success Criteria

- HTTPS working
- Deployment is straightforward
- Can follow docs to deploy

---

## Configuration Reference

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| PORT | No | HTTP port (default: 8080) |
| SYNC_SERVER_URL | Yes | Internal sync server URL (e.g., ws://192.168.1.100:3030) |
| SESSION_SECRET | Yes | Secret for signing session cookies |
| PASSWORD | Yes* | Login password (for simple auth) |
| HTTPS_CERT | No | Path to SSL certificate |
| HTTPS_KEY | No | Path to SSL private key |

### Docker Compose Example

```yaml
version: '3'
services:
  web:
    build: ./web-server
    ports:
      - "443:8080"
    environment:
      - SYNC_SERVER_URL=ws://sync:3030
      - SESSION_SECRET=your-secret-here
      - PASSWORD=your-password-here
    depends_on:
      - sync
    networks:
      - internal
      - external

  sync:
    image: ghcr.io/automerge/automerge-repo-sync-server:main
    volumes:
      - ./data:/data
    environment:
      - DATA_DIR=/data
    networks:
      - internal  # Only accessible internally

networks:
  internal:
    internal: true  # No external access
  external:
```

---

## Database Schema (If Using Multi-User)

### Users Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| username | VARCHAR | Unique username |
| password_hash | VARCHAR | Hashed password |
| root_doc_id | VARCHAR | User's root document ID (optional) |
| created_at | TIMESTAMP | Account creation |

### Sessions Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key / session token |
| user_id | UUID | FK to users (or null for single-user) |
| expires_at | TIMESTAMP | Session expiration |
| created_at | TIMESTAMP | Session creation |

---

## Security Considerations

1. **HTTPS required** - Especially for passwords/sessions
2. **Secure cookies** - HttpOnly, Secure, SameSite
3. **Rate limiting** - Prevent brute force on login
4. **Password hashing** - Use bcrypt or argon2
5. **Session expiration** - Don't keep sessions forever

---

## Estimated Timeline

| Phase | Estimated Duration |
|-------|-------------------|
| Phase 1: Project Setup | 0.5-1 week |
| Phase 2: Authentication | 1 week |
| Phase 3: WebSocket Relay | 1 week |
| Phase 4: Root Document Management | 0.5 week |
| Phase 5: Production Readiness | 1 week |

**Total: 4-5 weeks**

---

## Open Questions

1. **Authentication method** - Simple password vs multi-user vs passkey?
2. **Technology stack** - Node.js, Go, Rust?
3. **Domain/hosting** - Where will users host this?
4. **SSL certificates** - Let's Encrypt integration?
