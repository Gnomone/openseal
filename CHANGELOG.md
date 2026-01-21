# Changelog

All notable changes to OpenSeal will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0-alpha.3] - 2026-01-22

### Fixed
- 📦 **Dynamic Port Mapping**: Fixed port conflict issue (`Bind for 0.0.0.0:3000 failed`) by assigning random host port to container and resolving it dynamically.
  - Now safe to run multiple containers or when ports 3000/8080 are occupied.
  - Proxy correctly forwards to the dynamically assigned internal port.

## [1.0.0-alpha.2] - 2026-01-22

### Added
- 🛡️ **Verify Command**: Added `openseal verify` to cryptographically verify API responses.
  - Usage: `openseal verify --response <file> --wax <challenge>`
  - Supports `--root-hash` for identity verification.

## [1.0.0-alpha.1] - 2026-01-22

### 🎉 Major Architecture Change: Docker-Based Identity

v1.0.0 represents a complete redesign of OpenSeal, transitioning from file-based distribution to Docker container wrapping. This resolves fundamental environment fragmentation issues while maintaining all cryptographic security properties.

### Added

#### Core Features
- **Docker Image Digest as Root Hash**: SHA256 digest of container image replaces file-based Merkle Root
- **Daemon Mode Execution**: Containers run in detached mode (`-d`) with health check before proxy starts
- **Health Check System**: TCP connection wait (30s timeout) ensures container is ready
- **openseal.json v1 Format**: New schema with `image.digest`, `image.reference`, `identity.root_hash`
- **Graceful Shutdown**: Ctrl+C handler automatically stops and removes containers

#### CLI Commands
- `openseal build --image <image>`: Extract Image Digest and create openseal.json
- `openseal run --image <image> --port <port>`: Start container + OpenSeal Proxy

#### Security
- Container sandboxing with `--read-only`, `--cap-drop=ALL`, `--security-opt=no-new-privileges`
- Digest verification before execution (prevents image substitution)
- Maintained all v0 cryptographic properties: A-hash, B-hash, Ed25519 signatures

#### Developer Experience
- Development mode: Local Image ID support (no registry push required for testing)
- Automatic log streaming from container
- Clear UX messages with emoji indicators

### Changed

- **Root Hash Calculation**: `Merkle Tree of Files` → `Docker Image Digest (SHA256)`
- **Distribution Model**: `File bundles (dist/)` → `Docker Containers (Registry)`
- **Execution Model**: `Process spawn` → `Docker daemon containers`
- **Network Default**: `isolated (--network=none)` → `bridge (development mode)`
  - Rationale: BTC Oracle and similar use cases need external API calls
  - TODO: Implement whitelist in beta

### Removed

- File-based build system (`openseal build --source`)
- Dependency ghosting (`--deps`, symlink management)
- `.opensealignore` file scanning
- `dist/` output directory
- Multi-file Merkle Tree computation
- PATH environment manipulation

### Fixed

- Environment fragmentation (node_modules, venv not found)
- Dependency resolution across languages
- Reproducibility issues with file timestamps

### Known Limitations (Alpha)

- **No Registry Digest Enforcement**: Accepts local Image Tags (e.g., `crypto-oracle:v1`)
  - Production requires: `user/api@sha256:abc123...`
- **No Network Whitelist**: Uses `--network=bridge` unconditionally
  - Planned: Startup DNS resolve + iptables in beta
- **Basic Error Handling**: Limited validation and user-friendly messages
- **No Image Scanning**: Trivy/Grype integration planned for beta
- **Mixed Logs**: Container logs and proxy logs not separated

### Migration from v0

> ⚠️ **Breaking Changes**: v1 is NOT compatible with v0.2.x

**v0 is preserved as `v0.2.63-final` tag** for reference.

To migrate:
1. Dockerize your application (create `Dockerfile`)
2. Build Docker image: `docker build -t my-api:v1 .`
3. Run `openseal build --image my-api:v1`
4. Run `openseal run --image my-api:v1 --port 8080`

See [V1_TRANSITION_STRATEGY_KR.md](./docs/internal/v1/V1_TRANSITION_STRATEGY_KR.md) for details.

### Performance

- **Cold Start**: ~5-8 seconds (container start + health check)
  - v0: ~1-2 seconds (process spawn)
  - Trade-off accepted for environment isolation
- **Runtime Overhead**: Identical to v0 (same proxy logic)

### Security Considerations

#### Addressed in v1
- ✅ Environment isolation (Docker containers)
- ✅ Dependency management (Docker solves this)
- ✅ Image integrity (Digest verification)

#### TODO in Beta
- ⚠️ Network isolation with whitelist
- ⚠️ Registry digest enforcement
- ⚠️ Deterministic builds (base image pinning)
- ⚠️ Container escape prevention (AppArmor/Seccomp profiles)

See [V1.0.0_DESIGN_CONSIDERATIONS_KR.md](./docs/internal/v1/V1.0.0_DESIGN_CONSIDERATIONS_KR.md) for full security analysis.

### Verified Use Case

**BTC Price Oracle**
- Container: Node.js + Express + Axios
- External API: Coinbase Price API
- OpenSeal Seal: Proves price is from sealed container
- Verified Response:
  ```json
  {
    "openseal": {
      "a_hash": "18ddef79a8138634...",
      "b_hash": "493911b28d91e0ae...",
      "signature": "c327c9ef05b62792..."
    },
    "result": {
      "symbol": "BTC",
      "price": "89553.03"
    }
  }
  ```

### Internal Changes

- `openseal-cli/src/main.rs`: 582 lines → 280 lines (52% reduction)
- Removed crates: None (openseal-core, openseal-secret, openseal-runtime reused)
- New dependencies: None (reused existing tokio, blake3, ed25519-dalek)

---

## [0.2.63-final] - 2026-01-22

### Archived

v0.2.63 is the final release of the file-based architecture. Tagged as `v0.2.63-final` and preserved in the private repository.

**Status**: No further development. Use v1.0.0+ for new projects.

### Summary of v0 (Hackathon Edition)

- File-based Root Hash (Merkle Tree)
- Dependency ghosting (symlink management)
- Multi-language support (Node.js, Python, Go, Rust)
- `.opensealignore` for selective sealing

**Why Archived**: Environment fragmentation proved unsolvable at the file level. Docker provides a better abstraction.

---

## Versioning Strategy

- **v0.x**: File-based (Deprecated)
- **v1.0.0-alpha.x**: Docker-based (Current, unstable)
- **v1.0.0-beta.x**: Feature-complete, testing
- **v1.0.0**: Production-ready stable release

---

## Links

- [GitHub Repository](https://github.com/Gnomone/openseal)
- [v1 Implementation Plan](./docs/internal/v1/V1.0.0-ALPHA_IMPLEMENTATION_PLAN_KR.md)
- [Security Model](./docs/internal/TERMINOLOGY_KR.md)
