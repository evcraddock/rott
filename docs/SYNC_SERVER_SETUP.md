# Sync Server Setup Guide

This guide explains how to set up a sync server for ROTT, enabling you to sync your links across multiple devices.

## Overview

ROTT uses [automerge-repo-sync-server](https://github.com/automerge/automerge-repo) as a document relay. The sync server:

- Stores and forwards Automerge documents between your devices
- Runs on your private network (home server, NAS, Raspberry Pi, etc.)
- Requires no authentication—network access is the security boundary
- Has no knowledge of ROTT specifically; it's a generic Automerge relay

## Quick Start (Docker)

The fastest way to get started:

```bash
docker run -d \
  --name rott-sync \
  --restart unless-stopped \
  -p 3030:3030 \
  -v rott-sync-data:/data \
  -e DATA_DIR=/data \
  ghcr.io/automerge/automerge-repo-sync-server:main
```

Then configure ROTT to use it:

```bash
rott config set sync_url ws://YOUR_SERVER_IP:3030
rott config set sync_enabled true
```

---

## Deployment Options

### Option 1: Docker Compose (Recommended)

Create a `docker-compose.yml`:

```yaml
services:
  sync-server:
    image: ghcr.io/automerge/automerge-repo-sync-server:main
    ports:
      - "3030:3030"
    volumes:
      - ./data:/data
    environment:
      - PORT=3030
      - DATA_DIR=/data
    restart: unless-stopped
```

Run it:

```bash
docker compose up -d
```

### Option 2: Node.js Direct

If you prefer not to use Docker:

```bash
# Install and run
npx @automerge/automerge-repo-sync-server
```

For a permanent installation:

```bash
npm install -g @automerge/automerge-repo-sync-server
automerge-repo-sync-server
```

### Option 3: Systemd Service

For running as a system service on Linux:

1. Install Node.js on your server
2. Create `/etc/systemd/system/rott-sync.service`:

```ini
[Unit]
Description=ROTT Sync Server
After=network.target

[Service]
Type=simple
User=nobody
Environment=PORT=3030
Environment=DATA_DIR=/var/lib/rott-sync
ExecStart=/usr/bin/npx @automerge/automerge-repo-sync-server
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

3. Enable and start:

```bash
sudo mkdir -p /var/lib/rott-sync
sudo systemctl daemon-reload
sudo systemctl enable rott-sync
sudo systemctl start rott-sync
```

---

## Configuring ROTT

Once your sync server is running, configure ROTT to connect:

```bash
# Set the sync server URL (use your server's IP address)
rott config set sync_url ws://192.168.1.100:3030

# Enable sync
rott config set sync_enabled true

# Verify configuration
rott config show
```

### Testing the Connection

```bash
# Check sync status
rott sync status

# Trigger a manual sync
rott sync
```

---

## Multi-Device Setup

### First Device (Create New Identity)

```bash
# Initialize ROTT (creates a new root document)
rott init

# Note your root document ID - you'll need it for other devices
rott status
```

**Important:** Save your root document ID somewhere safe. It's your identity and the key to accessing your data on other devices.

### Additional Devices (Join Existing)

On each new device:

```bash
# Join using your root document ID from the first device
rott init --join YOUR_ROOT_DOCUMENT_ID

# Configure sync server (same URL as first device)
rott config set sync_url ws://192.168.1.100:3030
rott config set sync_enabled true

# Sync to pull your data
rott sync
```

---

## Remote Access via VPN

The sync server should only be accessible on your private network. For remote access, use a VPN:

### Tailscale (Recommended)

1. Install [Tailscale](https://tailscale.com/) on your sync server and all client devices
2. Use the Tailscale IP address for your sync URL:

```bash
rott config set sync_url ws://100.x.x.x:3030
```

### WireGuard

1. Set up WireGuard on your server and clients
2. Use the WireGuard tunnel IP for the sync URL

### Why VPN Instead of Exposing to Internet?

- **Simpler security**: No need for TLS certificates or authentication
- **No attack surface**: Sync server isn't exposed to the internet
- **Works everywhere**: VPN provides secure access from any network

---

## Security Considerations

### Network Access = Authorization

The sync server has no built-in authentication. Anyone who can reach it on the network can sync documents. This is by design—it keeps the server simple.

**Recommendations:**

- Run only on a trusted private network
- Use a VPN for remote access
- Don't expose port 3030 to the internet

### Data Privacy

- Data is **not encrypted** at rest on the sync server
- Data is **not encrypted** in transit within your private network
- If you need encryption, consider full-disk encryption on the sync server

### Firewall Configuration

If you have a firewall, allow port 3030 only from trusted networks:

```bash
# UFW example - allow only from local network
sudo ufw allow from 192.168.1.0/24 to any port 3030

# Or allow only from Tailscale
sudo ufw allow from 100.64.0.0/10 to any port 3030
```

---

## Hardware Requirements

### Minimum

- Raspberry Pi Zero 2 W or equivalent
- 512MB RAM
- Network connection

### Recommended

- Raspberry Pi 4 or any modern SBC/server
- 1GB+ RAM
- SSD storage (faster sync)
- Ethernet connection (more reliable than WiFi)

### Resource Usage

The sync server is lightweight:
- ~50-100MB RAM typical usage
- Minimal CPU (mostly idle, spikes during sync)
- Storage depends on your data volume

---

## Backup

### What to Back Up

The sync server stores documents in its data directory. Back up this directory to preserve your synced data.

**Docker:**
```bash
# Find the volume location
docker volume inspect rott-sync-data

# Or backup while running
docker run --rm -v rott-sync-data:/data -v $(pwd):/backup alpine \
  tar czf /backup/rott-sync-backup.tar.gz /data
```

**Direct install:**
```bash
tar czf rott-sync-backup.tar.gz /var/lib/rott-sync
```

### Recovery

Your ROTT data is also stored locally on each device. If the sync server is lost:

1. Set up a new sync server
2. Sync from any device that has your data
3. Other devices will receive the data through the new server

---

## Troubleshooting

### Sync server won't start

**Port already in use:**
```bash
# Check what's using port 3030
sudo lsof -i :3030

# Use a different port
docker run -d -p 3031:3030 ... # then use ws://host:3031
```

**Permission denied on data directory:**
```bash
# Fix permissions
sudo chown -R 1000:1000 ./data
```

### Can't connect from ROTT

**Check server is running:**
```bash
docker ps | grep rott-sync
# or
systemctl status rott-sync
```

**Check network connectivity:**
```bash
# From client machine
nc -zv 192.168.1.100 3030
```

**Verify ROTT configuration:**
```bash
rott config show
# Check sync_url and sync_enabled
```

### Sync not working

**Check sync status:**
```bash
rott sync status
```

**Try manual sync:**
```bash
rott sync
```

**Check server logs:**
```bash
docker logs rott-sync
# or
journalctl -u rott-sync
```

### "Pending sync" state after join

This is normal when joining from a new device. Run:

```bash
rott sync
```

The initial sync may take a moment if you have a lot of data.

---

## Advanced: Kubernetes / k3s

For detailed Kubernetes deployment, see [docs/plans/SYNC_SERVER.md](plans/SYNC_SERVER.md).

Quick overview:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: automerge-sync
spec:
  replicas: 1
  selector:
    matchLabels:
      app: automerge-sync
  template:
    spec:
      containers:
      - name: sync
        image: ghcr.io/automerge/automerge-repo-sync-server:main
        ports:
        - containerPort: 3030
        env:
        - name: DATA_DIR
          value: /data
        volumeMounts:
        - name: data
          mountPath: /data
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: automerge-sync-pvc
```

**Note:** The official image may only support amd64. For ARM clusters (Raspberry Pi), you may need to build your own image.

---

## Next Steps

- [Understanding Your Root Document ID](IDENTITY.md) - Learn why your root document ID matters
- [Troubleshooting Guide](TROUBLESHOOTING.md) - Solutions to common problems
