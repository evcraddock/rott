# Implementation Plan: Sync Server

## Overview

The sync server is a simple Automerge document relay deployed on the user's private network. It uses the existing **automerge-repo-sync-server** package.

## Goals

1. Relay Automerge sync messages between devices
2. Store documents for offline devices
3. Run on user's private network (home server, NAS, Raspberry Pi)
4. Minimal configuration and maintenance

## Design Principles

- **Use existing software** - Don't build what already exists
- **Simple deployment** - Docker or single binary
- **No authentication** - Network access is the security boundary
- **No application logic** - Generic Automerge relay

---

## Deployment Options

### Option A: Docker (Recommended for single server)

```bash
docker run -d \
  --name rott-sync \
  -p 3030:3030 \
  -v /path/to/data:/data \
  -e DATA_DIR=/data \
  ghcr.io/automerge/automerge-repo-sync-server:main
```

### Option B: Node.js Direct

```bash
npx @automerge/automerge-repo-sync-server
```

### Option C: Systemd Service

For running on a Linux server/Raspberry Pi as a system service.

### Option D: Kubernetes / k3s

For running on a Kubernetes cluster. See detailed section below.

---

## Phase 1: Basic Deployment

### Objective

Get a sync server running on the private network.

### Tasks

1. **Choose deployment target**
   - Home server, NAS, Raspberry Pi, old laptop, etc.
   - Needs to be always-on (or at least when syncing)

2. **Install Docker** (if using Docker option)
   - Or install Node.js for direct option

3. **Run sync server**
   - Use one of the deployment options above
   - Configure data persistence directory

4. **Network configuration**
   - Ensure server is accessible on local network
   - Note the IP address and port (e.g., `192.168.1.100:3030`)

5. **Test connectivity**
   - From another device on network, connect to WebSocket
   - Verify server is reachable

### Deliverables

- Sync server running on private network
- Accessible from other devices on network

### Success Criteria

- Server starts and stays running
- Can connect from another device on LAN

---

## Phase 2: Persistence and Reliability

### Objective

Ensure the sync server is reliable and data persists.

### Tasks

1. **Data persistence**
   - Configure data directory on persistent storage
   - Verify data survives server restart

2. **Auto-restart**
   - Docker: Use `--restart unless-stopped`
   - Systemd: Configure service to restart on failure

3. **Startup on boot**
   - Docker: Container starts on system boot
   - Systemd: Enable service

4. **Basic monitoring**
   - Check if server is running
   - Simple health check script (optional)

### Deliverables

- Sync server survives reboots
- Data persists across restarts

### Success Criteria

- Reboot server machine, sync server comes back
- Documents still available after restart

---

## Phase 3: Remote Access (Optional)

### Objective

Enable sync when away from home network.

### Tasks

1. **Choose VPN solution**
   - Tailscale (easiest)
   - WireGuard
   - Other VPN

2. **Install VPN on server**
   - Server joins VPN network
   - Note VPN IP address

3. **Configure sync server to listen on VPN**
   - Bind to VPN interface or all interfaces
   - Sync server accessible via VPN IP

4. **Install VPN on client devices**
   - All devices that need remote sync
   - Test connectivity through VPN

### Deliverables

- Sync server accessible via VPN
- Can sync from anywhere with VPN connected

### Success Criteria

- Connect to VPN from remote location
- Sync works through VPN

---

## Configuration Reference

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| PORT | 3030 | WebSocket port |
| DATA_DIR | ./data | Document storage directory |

### Docker Compose Example

```yaml
version: '3'
services:
  rott-sync:
    image: ghcr.io/automerge/automerge-repo-sync-server:main
    restart: unless-stopped
    ports:
      - "3030:3030"
    volumes:
      - ./data:/data
    environment:
      - DATA_DIR=/data
```

### Systemd Service Example

```ini
[Unit]
Description=ROTT Sync Server
After=network.target

[Service]
Type=simple
User=rott
WorkingDirectory=/opt/rott-sync
ExecStart=/usr/bin/npx @automerge/automerge-repo-sync-server
Restart=on-failure
Environment=PORT=3030
Environment=DATA_DIR=/opt/rott-sync/data

[Install]
WantedBy=multi-user.target
```

---

## Kubernetes / k3s Deployment

### ARM Image Consideration

The official `ghcr.io/automerge/automerge-repo-sync-server:main` image may only support `amd64`. For ARM-based clusters (Raspberry Pi), you'll need to build your own multi-arch image.

### Building a Multi-Arch Image

**Dockerfile (ARM-compatible):**

```dockerfile
FROM node:20-slim
WORKDIR /app
RUN npm install @automerge/automerge-repo-sync-server
ENV PORT=3030
ENV DATA_DIR=/data
EXPOSE 3030
CMD ["npx", "@automerge/automerge-repo-sync-server"]
```

**Build and push:**

```bash
# Enable buildx for multi-arch
docker buildx create --use

# Build for both amd64 and arm64
docker buildx build --platform linux/amd64,linux/arm64 \
  -t your-registry/automerge-sync:latest \
  --push .
```

### Kubernetes Manifests

**namespace.yaml** (optional):

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: rott
```

**pvc.yaml:**

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: automerge-sync-pvc
  namespace: rott
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 5Gi
  # storageClassName: local-path  # Uncomment for k3s local-path provisioner
```

**deployment.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: automerge-sync
  namespace: rott
spec:
  replicas: 1
  selector:
    matchLabels:
      app: automerge-sync
  template:
    metadata:
      labels:
        app: automerge-sync
    spec:
      containers:
      - name: sync
        image: your-registry/automerge-sync:latest
        ports:
        - containerPort: 3030
        env:
        - name: DATA_DIR
          value: /data
        - name: PORT
          value: "3030"
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: automerge-sync-pvc
```

**service.yaml:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: automerge-sync
  namespace: rott
spec:
  selector:
    app: automerge-sync
  ports:
  - port: 3030
    targetPort: 3030
    protocol: TCP
```

**ingress.yaml** (optional, for external access via Traefik/nginx):

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: automerge-sync
  namespace: rott
  annotations:
    # For Traefik (k3s default)
    traefik.ingress.kubernetes.io/router.entrypoints: websecure
spec:
  rules:
  - host: sync.your-domain.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: automerge-sync
            port:
              number: 3030
```

### Deploying to k3s

```bash
# Apply all manifests
kubectl apply -f namespace.yaml
kubectl apply -f pvc.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml

# Check status
kubectl get pods -n rott
kubectl logs -n rott deployment/automerge-sync

# Get service IP (for internal cluster access)
kubectl get svc -n rott
```

### Accessing the Sync Server

**From within the cluster:**
```
ws://automerge-sync.rott.svc.cluster.local:3030
```

**From the local network (NodePort):**

Add to service.yaml:
```yaml
spec:
  type: NodePort
  ports:
  - port: 3030
    targetPort: 3030
    nodePort: 30030  # Access via any node IP:30030
```

**Via Ingress (if configured):**
```
ws://sync.your-domain.local
```

### Helm Chart (Optional)

For more complex deployments, consider creating a Helm chart:

```
helm/automerge-sync/
├── Chart.yaml
├── values.yaml
└── templates/
    ├── deployment.yaml
    ├── service.yaml
    ├── pvc.yaml
    └── ingress.yaml
```

This is optional and can be added later if managing multiple deployments.

---

## Hardware Requirements

### Minimum

- Raspberry Pi Zero 2 W or equivalent
- 512MB RAM
- 1GB storage (plus space for documents)
- Network connection

### Recommended

- Raspberry Pi 4 or any modern SBC/server
- 1GB+ RAM
- SSD for storage (faster sync)
- Ethernet connection (more reliable than WiFi)

---

## Backup Considerations

### What to Back Up

- The data directory (contains all Automerge documents)

### Backup Strategy

- Regular copy of data directory to external storage
- Or use filesystem-level backup (ZFS snapshots, etc.)
- Cloud backup if desired (data is not encrypted, consider implications)

---

## Troubleshooting

### Server won't start

- Check port 3030 is not in use
- Check data directory permissions
- Check Docker/Node.js is installed

### Can't connect from other devices

- Verify server IP address
- Check firewall allows port 3030
- Ensure devices are on same network (or VPN)

### Sync not working

- Check WebSocket URL in client config
- Verify network connectivity
- Check server logs for errors

---

## Estimated Timeline

| Phase | Estimated Duration |
|-------|-------------------|
| Phase 1: Basic Deployment | 1-2 hours |
| Phase 2: Persistence and Reliability | 1-2 hours |
| Phase 3: Remote Access (Optional) | 2-4 hours |

**Total: Half a day to a day**

This is intentionally simple - we're using existing software, not building a custom server.

---

## Open Questions

1. **Backup automation** - Worth providing a backup script?
2. **Monitoring** - Simple health check endpoint useful?
3. **Documentation** - Provide setup guides for common platforms (Raspberry Pi, Synology NAS, etc.)?
