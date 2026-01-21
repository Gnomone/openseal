# OpenSeal Provider Guide

**For**: Service operators who run sealed APIs (not developers)

[🇰🇷 한국어 버전](./PROVIDER_GUIDE_KR.md)

---

## Overview

As a **Normal Provider**, you:
- ✅ Pull pre-built Docker images from GitHub/Registry
- ✅ Run OpenSeal to seal the service
- ✅ Operate the sealed API server

You do **NOT** need to:
- ❌ Write code
- ❌ Build Docker images
- ❌ Understand the API internals

---

## Prerequisites

1. **Docker** installed
2. **OpenSeal CLI** installed

### Install OpenSeal

```bash
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash
```

---

## Quick Start: Run Crypto Price Oracle

### Step 1: Pull Docker Image

```bash
# From GitHub Container Registry (after Actions build)
docker pull ghcr.io/gnomone/crypto-price-oracle:latest

# Or build from source
git clone https://github.com/Gnomone/crypto-price-oracle.git
cd crypto-price-oracle
docker build -t crypto-price-oracle:v1 .
```

### Step 2: Seal the Image

```bash
openseal build --image ghcr.io/gnomone/crypto-price-oracle:latest
```

This creates `openseal.json`:
```json
{
  "version": "1.0.0",
  "identity": {
    "root_hash": "sha256:...",
    "seal_version": "2.0"
  }
}
```

### Step 3: Run the Sealed Service

```bash
openseal run --image ghcr.io/gnomone/crypto-price-oracle:latest --port 8080
```

Output:
```
🐳 Starting container...
✅ Container ready
🔐 Starting OpenSeal Proxy on port 8080...
📡 Proxy Server Ready!
   Public: http://0.0.0.0:8080
```

### Step 4: Test

```bash
curl -X POST http://localhost:8080/api/v1/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: test123" \
  -d '{"symbol":"BTC"}'
```

✅ **Done!** Your sealed API is running.

---

## Production Deployment

### Option 1: Systemd Service

Create `/etc/systemd/system/crypto-oracle.service`:

```ini
[Unit]
Description=Crypto Price Oracle (OpenSeal)
After=docker.service
Requires=docker.service

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/crypto-oracle
ExecStart=/usr/local/bin/openseal run --image ghcr.io/gnomone/crypto-price-oracle:latest --port 8080
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable crypto-oracle
sudo systemctl start crypto-oracle
sudo systemctl status crypto-oracle
```

### Option 2: Docker Compose

```yaml
version: '3.8'
services:
  oracle:
    image: ghcr.io/gnomone/crypto-price-oracle:latest
    ports:
      - "8080:8080"
    restart: unless-stopped
```

---

## Monitoring

### View Logs

```bash
# Systemd
sudo journalctl -u crypto-oracle -f

# Docker
docker logs -f <container-id>
```

### Health Check

```bash
curl http://localhost:8080/health
```

---

## Updating the Service

```bash
# 1. Pull new image
docker pull ghcr.io/gnomone/crypto-price-oracle:latest

# 2. Stop existing service
sudo systemctl stop crypto-oracle

# 3. Rebuild seal
openseal build --image ghcr.io/gnomone/crypto-price-oracle:latest

# 4. Restart
sudo systemctl start crypto-oracle
```

---

## Troubleshooting

### Port Already in Use

```bash
sudo lsof -i :8080
sudo kill -9 <PID>
```

### Container Won't Start

```bash
docker ps -a
docker logs <container-id>
```

### Image Not Found

```bash
# Check if image exists
docker images | grep crypto-price-oracle

# Re-pull
docker pull ghcr.io/gnomone/crypto-price-oracle:latest
```

---

## Next Steps

- **[Seed Provider Guide](./SEED_PROVIDER_GUIDE.md)**: If you want to create your own sealed service
- **[Verifier Guide](./VERIFIER_GUIDE.md)**: How users verify your seals

---

**Questions?** Open an issue on [GitHub](https://github.com/Gnomone/openseal/issues)
