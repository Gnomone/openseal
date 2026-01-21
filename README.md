# 🔐 OpenSeal v1.0.0-alpha.1

[🇰🇷 한국어 버전 (Korean)](./README_KR.md)

**Trusted Container Runner**: Turn any Docker container into a verifiable API with cryptographic proofs.

> ⚠️ **Alpha Release**: Early preview. Not recommended for production.

---

## What is OpenSeal?

OpenSeal wraps Docker containers with **cryptographic identity**, making every API response:
- ✅ **Verifiable**: Ed25519 signatures prove authenticity
- ✅ **Tamper-Evident**: Any modification breaks the seal
- ✅ **Non-Repudiable**: Mathematical proof of origin

---

## Quick Start

### For API Users
```bash
# Query a sealed API
curl -H "X-OpenSeal-Wax: myChallenge" http://api.example.com/endpoint
```

Response includes cryptographic proof:
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

**[→ Provider Guide](./docs/public/PROVIDER_GUIDE.md)** | **[→ 제공자 가이드](./docs/public/PROVIDER_GUIDE_KR.md)**

### For Developers (Seed Providers)
```bash
# 1. Create your API
# 2. Write Dockerfile
docker build -t my-api:v1 .

# 3. Push to GitHub
docker push ghcr.io/yourorg/my-api:v1

# 4. Publish openseal.json
openseal build --image ghcr.io/yourorg/my-api:v1
```

**[→ Seed Provider Guide](./docs/public/SEED_PROVIDER_GUIDE.md)** | **[→ 시드 제공자 가이드](./docs/public/SEED_PROVIDER_GUIDE_KR.md)**

### For Verifiers

```bash
# Query and verify sealed APIs
curl -H "X-OpenSeal-Wax: challenge" http://api.example.com/endpoint
```

**[→ Verifier Guide](./docs/public/VERIFIER_GUIDE.md)** | **[→ 검증자 가이드](./docs/public/VERIFIER_GUIDE_KR.md)**

---

## Example: Crypto Price Oracle

Verified cryptocurrency prices with cryptographic proof:

```bash
# Provider side
git clone https://github.com/Gnomone/crypto-price-oracle.git
cd crypto-price-oracle
docker build -t crypto-oracle:v1 .
openseal build --image crypto-oracle:v1
openseal run --image crypto-oracle:v1 --port 8080

# User side
curl -X POST http://localhost:8080/api/v1/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: prove-it" \
  -d '{"symbol":"BTC"}'
```

**Full Example**: [crypto-price-oracle](https://github.com/Gnomone/crypto-price-oracle)

---

## How It Works

```
Client → OpenSeal Proxy → Container
         ↓
    1. Compute A-hash (Identity)
    2. Forward to container
    3. Compute B-hash (Result binding)
    4. Sign with Ed25519
         ↓
    Response with Seal
```

**Core Concepts**:
- **Root Hash**: Docker Image Digest (immutable identity)
- **Wax**: Client challenge (prevents replay)
- **A-hash**: `Blake3(Root Hash || Wax)`
- **B-hash**: `b_G(A-hash, Wax, Result)` (secret function)
- **Signature**: `Ed25519.sign(Wax||A||B||ResultHash)`

---

## Documentation

### For Users
- **[User Guide](./docs/public/USER_GUIDE.md)**: How to query and verify sealed APIs
- **[사용자 가이드 (KR)](./docs/public/USER_GUIDE_KR.md)**

### For Providers
- **[Provider Guide](./docs/public/PROVIDER_GUIDE.md)**: How to deploy sealed services
- **[배포자 가이드 (KR)](./docs/public/PROVIDER_GUIDE_KR.md)**

### Architecture & Design
- **[CHANGELOG](./CHANGELOG.md)**: Version history
- **[V1 Development Directive](./docs/internal/v1/V1.0.0-ALPHA_DIRECTIVE_KR.md)**: Core design decisions
- **[Security Model](./docs/internal/TERMINOLOGY_KR.md)**: Cryptographic terminology

---

## Why v1? (Docker-Based)

### v0 Problems
- ❌ Environment fragmentation (PATH, dependencies)
- ❌ Language-specific complexity
- ❌ Reproducibility issues

### v1 Solution
- ✅ Docker = standardized packaging
- ✅ Language-agnostic
- ✅ Image Digest = cryptographic identity
- ✅ Industry-standard isolation

**[Full Rationale](./docs/internal/v1/V1_TRANSITION_STRATEGY_KR.md)**

---

## Roadmap

- **v1.0.0-alpha.1** (Current): Core functionality working
- **v1.0.0-beta.1** (Next): Network whitelist, Registry enforcement, Image scanning
- **v1.0.0** (Stable): Production-ready

---

## Community

- **GitHub**: https://github.com/Gnomone/openseal
- **HighStation**: Integration platform for sealed APIs
- **Discord**: Coming soon

---

## License

MIT License - See [LICENSE](./LICENSE)

---

Built with ❤️ as part of HighStation's Trusted AI Infrastructure
