# OpenSeal Provider Guide

**For**: Service providers who want to deploy sealed APIs

[🇰🇷 한국어 버전](./PROVIDER_GUIDE_KR.md)

---

## Overview

As a provider, you'll wrap your existing Docker container with OpenSeal to add cryptographic proofs to all responses.

**What You Get**:
- ✅ Every API response includes a cryptographic seal
- ✅ Users can verify results came from your container
- ✅ Tamper-evident: Any modification breaks the seal
- ✅ Non-repudiable: Mathematical proof of origin

---

## Prerequisites

1. **Docker** installed
2. **Your API** already containerized
3. **OpenSeal CLI** installed

### Install OpenSeal

```bash
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash

# Or build from source (v1-dev branch)
git clone https://github.com/Gnomone/openseal.git -b v1-dev
cd openseal
cargo build --release
cp target/release/openseal /usr/local/bin/
```

---

## Quick Start

### Step 1: Dockerize Your API (If Not Already)

```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY . .
ENV PORT=3000
EXPOSE 3000
CMD ["npm", "start"]
```

```bash
docker build -t my-api:v1 .
```

### Step 2: Seal the Image

```bash
openseal build --image my-api:v1
```

This creates `openseal.json`:
```json
{
  "version": "1.0.0",
  "identity": {
    "root_hash": "sha256:7dabd9a9dd2d...",
    "seal_version": "2.0"
  },
  "image": {
    "reference": "my-api:v1",
    "digest": "sha256:7dabd9a9dd2d...",
    "created_at": "2026-01-22T05:30:00Z"
  }
}
```

### Step 3: Run with OpenSeal Proxy

```bash
openseal run --image my-api:v1 --port 8080
```

Output:
```
🐳 Starting container...
✅ Container ready
🔐 Starting OpenSeal Proxy on port 8080...
📡 Proxy Server Ready!
   Public: http://0.0.0.0:8080
   🔑 Public Key (Ephemeral): d30c05d163733bae...
```

### Step 4: Test

```bash
curl -H "X-OpenSeal-Wax: test123" http://localhost:8080/your-endpoint
```

Response includes seal:
```json
{
  "openseal": {
    "a_hash": "...",
    "b_hash": "...",
    "signature": "...",
    "pub_key": "..."
  },
  "result": { "your": "data" }
}
```

✅ **Done!** Your API is now sealed.

---

## Deployment Scenarios

### Scenario 1: Local Development

```bash
# Build from source
cd my-api
docker build -t my-api:dev .
openseal build --image my-api:dev
openseal run --image my-api:dev --port 8080
```

### Scenario 2: Production Server

```bash
# Pull from registry
docker pull ghcr.io/yourorg/my-api:v1.0.0
openseal build --image ghcr.io/yourorg/my-api:v1.0.0
openseal run --image ghcr.io/yourorg/my-api:v1.0.0 --port 8080
```

### Scenario 3: Systemd Service

Create `/etc/systemd/system/my-sealed-api.service`:

```ini
[Unit]
Description=My Sealed API (OpenSeal)
After=docker.service
Requires=docker.service

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/my-api
ExecStart=/usr/local/bin/openseal run --image my-api:v1 --port 8080
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable my-sealed-api
sudo systemctl start my-sealed-api
```

---

## Network Configuration

### Default: No External Access

```bash
# v1.0.0-alpha.1: Uses --network=bridge (development mode)
openseal run --image my-api:v1 --port 8080
```

⚠️ **Alpha Limitation**: No network isolation yet. Coming in beta.

### Allow Specific Domains (Coming in Beta)

```bash
# Future syntax (not yet implemented)
openseal run \
  --image my-api:v1 \
  --port 8080 \
  --allow-network api.coinbase.com:443 \
  --allow-network db.example.com:5432
```

---

## Security Best Practices

### 1. Use Registry Digests (Production)

```bash
# ❌ Development only
openseal build --image my-api:latest

# ✅ Production
docker push my-org/my-api:v1.0.0
openseal build --image my-org/my-api@sha256:abc123...
```

### 2. Publish Your Root Hash

Add to your documentation:

```markdown
## OpenSeal Verification

Our API runs in a sealed container:

**Image Digest (Root Hash)**:
```
sha256:7dabd9a9dd2de714b343b381ee81bbcb3c2bb55b85fecefc3127fd9eafa486b2
```

All responses include cryptographic proofs tied to this digest.
```

### 3. Rotate Signing Keys

OpenSeal generates ephemeral keys per session. Restart periodically:

```bash
# Restart daily for key rotation
0 0 * * * systemctl restart my-sealed-api
```

### 4. Monitor Health

```bash
# Health check endpoint (added by OpenSeal)
curl http://localhost:8080/health
```

---

## Publishing to HighStation

### 1. Register Service

Visit: https://www.highstation.net/dashboard

- **Service Name**: My Sealed API
- **Slug**: `my-api`
- **Endpoint URL**: `https://your-server.com:8080`
- **Upload** `openseal.json`

### 2. Verify Registration

```bash
# Test via HighStation
curl https://api.highstation.net/services/my-api
```

### 3. Enable MCP Integration

HighStation automatically creates MCP configuration for your sealed service.

---

## Troubleshooting

### Container Won't Start

```bash
# Check Docker logs
docker ps -a | grep openseal
docker logs <container-id>
```

**Common Issues**:
- Port already in use
- Image not found
- Insufficient permissions

### Digest Mismatch

```bash
# Rebuild openseal.json
openseal build --image my-api:v1

# Verify
cat openseal.json
docker inspect my-api:v1 --format='{{.Id}}'
```

### Health Check Timeout

```bash
# Check container health
docker ps
docker exec <container-id> curl http://localhost:3000/health

# Increase timeout (coming in beta)
openseal run --image my-api:v1 --health-timeout 60
```

---

## Advanced: Custom Dockerfile

### Recommended Structure

```dockerfile
# Multi-stage build for smaller images
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Runtime stage
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY --from=builder /app/dist ./dist
ENV NODE_ENV=production
ENV PORT=3000
EXPOSE 3000
CMD ["npm", "start"]
```

### Deterministic Builds

```dockerfile
# Pin base image with digest
FROM node:20-alpine@sha256:abc123...

# Pin package versions
RUN npm ci --frozen-lockfile
```

---

## Example: Crypto Price Oracle

Full working example: https://github.com/Gnomone/crypto-price-oracle

```bash
# Clone
git clone https://github.com/Gnomone/crypto-price-oracle.git
cd crypto-price-oracle

# Build
docker build -t crypto-oracle:v1 .

# Seal
openseal build --image crypto-oracle:v1

# Run
openseal run --image crypto-oracle:v1 --port 8080

# Test
curl -X POST http://localhost:8080/api/v1/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: test123" \
  -d '{"symbol":"BTC"}'
```

---

## Monitoring & Logging

### View Logs

```bash
# OpenSeal streams container logs
# Just run and watch

openseal run --image my-api:v1 --port 8080
# [APP] Your application logs appear here
# [SEAL] OpenSeal proxy logs
```

### Production Logging

```bash
# Redirect to file
openseal run --image my-api:v1 --port 8080 2>&1 | tee openseal.log

# Or use systemd journal
sudo journalctl -u my-sealed-api -f
```

---

## Performance Considerations

### Cold Start

- **Container Start**: ~3-5 seconds
- **Health Check**: ~2-3 seconds
- **Total**: ~5-8 seconds

**Mitigation**: Use Daemon Mode (already default in v1)

### Runtime Overhead

- **Seal Generation**: ~5ms per request
- **Comparable to TLS handshake**

---

## Migration from v0

v0.2.x is incompatible with v1.0.x. To migrate:

1. **Dockerize your application**
2. Remove v0 `dist/` bundles
3. Follow Quick Start above

**v0 Preservation**: Tagged as `v0.2.63-final` in the repository.

---

## FAQ

**Q: Can I use OpenSeal with existing Docker images?**  
A: Yes! No code changes needed. Just seal and run.

**Q: What if my app doesn't use Docker?**  
A: Create a Dockerfile first. OpenSeal v1 requires Docker.

**Q: Do I need to modify my API code?**  
A: No! OpenSeal wraps your existing API transparently.

**Q: How do users verify my seals?**  
A: They send `X-OpenSeal-Wax` header. OpenSeal handles the rest.

**Q: Can I run multiple sealed services?**  
A: Yes! Use different ports for each.

---

## Next Steps

- **[User Guide](./USER_GUIDE.md)**: How users interact with your sealed API
- **[Example Service](https://github.com/Gnomone/crypto-price-oracle)**: Full reference implementation
- **[HighStation Integration](https://www.highstation.net)**: List your service

---

**Need Help?** Open an issue on [GitHub](https://github.com/Gnomone/openseal/issues)
