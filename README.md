[🇰🇷 한국어 버전 (Korean Version)](./README_KR.md)

# 🔐 OpenSeal: Create a 'Verifiable API' in 10 Seconds

Verify that your service code is untampered and mathematically proven to your customers, without modifying a single line of your actual business logic.

### 1. Install via Binary (Recommended)
```bash
# Download and install the latest binary for your OS
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash
```
> **Note**: Source compilation via `cargo install` is no longer supported for public builds as core security logic is protected. Please use the binary release.

### 2. Seal (Build)
> [!IMPORTANT]
> Always run OpenSeal commands at your **project root** directory.

```bash
# Node.js Example: Register execution command and output to 'dist_opensealed'
openseal build --exec "node app.js" --output dist_opensealed
```
> [!TIP]
> For **other languages** like Python, Go, and Rust, see the [Usage Guide (USAGE)](./docs/public/USAGE.md#2-quickstart-by-language).

### 3. Run
```bash
# Use your original port (OpenSeal manages internal port automatically)
openseal run --app dist_opensealed --port 3000
```

**✅ Done!** Your API service now attaches an unforgeable cryptographic Seal to every execution result.

---

### 🔐 Protected Runtime

OpenSeal’s Seal generation engine is distributed as a protected runtime. This is a deliberate design choice:
- All **verification logic is fully open-source** via OSIP-7325.
- Any third party can independently verify every Seal.
- However, **Seal generation is intentionally constrained** within a protected boundary to prevent forgery, replay, or memory-patching attacks in adversarial environments.

This mirrors industry-standard designs used in Secure Enclaves (TEE), HSM-backed signing, and edge execution runtimes.

---

### 🛡️ Threat Model & Guarantees

| Security Goal | OpenSeal Guarantee |
| :--- | :--- |
| **Result Integrity** | Proves the result originated from the sealed code under the given context. |
| **Identity Binding** | Ensures the execution environment (A-hash) matches the approved state. |
| **Anti-Replay** | Prevents reusing old Seals for new requests via mandatory Wax (Nonce). |
| **Privacy** | Zero data collection. No outbound network calls from the generation core. |

---

## 📖 Learn More
* [Protocol Specification (PROTOCOL)](./docs/public/PROTOCOL.md)
* [Security Policy & Strategy (POLICY)](./docs/public/POLICY.md)
* [Language Agnosticism & Source Integrity (AGNOSTICISM)](./docs/public/AGNOSTICISM.md)
* [Usage Guide (USAGE)](./docs/public/USAGE.md)
