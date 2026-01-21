# 🛠️ OpenSeal: Usage Guide

This guide covers how to set up, execute, and safely manage your OpenSeal-protected services.

---

## 1. 5-Minute Quickstart

### Step 1: Install CLI (v0.2.62+)
```bash
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash
hash -r
openseal --version
```

### Step 2: Seal (Build)
```bash
# Run at your project root
openseal build --exec "npm run dev" --output dist_opensealed
```

> [!TIP]
> **v0.2.62 Automation**: The output directory is now automatically added to `.opensealignore`. This ensures Hash reproducibility during rebuilds.

### Step 3: Run
```bash
# Run (Auto-detect dependencies)
openseal run --app dist_opensealed --port 3000

# Explicitly specify dependency folder (Recommended)
openseal run --app dist_opensealed --port 3000 --dependency node_modules

# Background (Auto-install supported)
openseal run --app dist_opensealed --port 3000 --daemon
```

> [!TIP]
> Use `--daemon` flag to keep the service running even after SSH disconnection.

---

## 2. Quickstart by Language

OpenSeal recommends executing verified source code directly (JIT).

### 🟢 Node.js / TypeScript
```bash
openseal build --exec "npm run dev" --output dist_opensealed
openseal run --app dist_opensealed --port 3000
```
> 💡 **JIT Recommended**: Use `tsx` or `ts-node` to execute source directly

### 🐍 Python
```bash
openseal build --exec "python main.py" --output dist_opensealed
openseal run --app dist_opensealed --port 8000
```
> 💡 **Virtual Env**: Automatically detects `venv`, `.venv`

### 🔵 Go
```bash
go build -o app
openseal build --exec "./app" --output dist_opensealed
openseal run --app dist_opensealed --port 8080
```

### 🦀 Rust
```bash
cargo build --release
openseal build --exec "./target/release/myapp" --output dist_opensealed
openseal run --app dist_opensealed --port 8000
```

---

## 3. Key Options

| Option | Description | Example |
|--------|-------------|---------|
| `--exec` | Command to start your service | `npm run dev`, `python app.py` |
| `--output` | Directory for sealed artifacts | `dist_opensealed` |
| `--daemon` | Run in background (Production) | - |

---

## 4. Standard Identity Endpoint

All OpenSeal services automatically expose a `/.openseal/identity` endpoint.

```bash
curl http://localhost:3000/.openseal/identity
```

**Response**:
```json
{
  "service": "OpenSeal Runtime Identity",
  "version": "0.2.6",
  "identity": {
    "a_hash": "14f38520...",
    "file_count": 1630
  },
  "status": "sealed"
}
```

This allows external tools like **HighStation** to verify code integrity in real-time without modifying your app.

---

## 5. Runtime Integrity Verification (v0.2.6+)

OpenSeal Runtime automatically verifies the sealed bundle on startup.

**How it works**:
1. Scans `dist_opensealed/` and calculates Live Hash
2. Compares with Expected Hash in `openseal.json`
3. **If tampered → Runtime refuses to start**

**Normal case**:
```bash
$ openseal run --app dist_opensealed --port 3000
   ✅ Live A-hash: 14f38520...
   ✅ Integrity Verified!
   🚀 OpenSeal Running
```

**Tampered case**:
```bash
$ openseal run --app dist_opensealed --port 3000
   🚨 INTEGRITY VIOLATION DETECTED
   Expected: 14f38520...
   Actual:   XXXXXXXX...
   Error: Runtime aborted
```

---

## 6. openseal verify (Verification Tool)

Verify the integrity of API responses.

```bash
openseal verify --response result.json --wax "nonce" --root-hash "14f38520..."
```

**result.json format**:
```json
{
  "result": { "symbol": "BTC", "price": "98500" },
  "openseal": {
    "signature": "...",
    "pub_key": "...",
    "a_hash": "...",
    "b_hash": "..."
  }
}
```

**Verification checks**:
- ✅ **Signature**: Ed25519 signature validity
- ✅ **Binding**: B-hash match
- ✅ **Identity**: A-hash match (if --root-hash provided)

---

## 7. Safety Guardrails

OpenSeal prevents accidental sealing of unintended locations.

**Auto-detection**:
- Checks for `package.json`, `Cargo.toml`, `.git`, etc.
- Warns if no project files found

**Recommendations**:
- ✅ Run at project root
- ✅ Use `.opensealignore` to exclude unnecessary files

---

## 8. Exclusion Rules

**`.opensealignore`**:
- Completely excluded from A-hash calculation
- Example: `node_modules/`, `venv/`, `.git/`

**`.openseal_mutable`**:
- File presence sealed, content changes allowed
- Example: `*.db`, `logs/`, `cache/`

---

## 📚 Additional Documentation

- [Protocol Specification (PROTOCOL)](./PROTOCOL.md)
- [Language Agnosticism (AGNOSTICISM)](./AGNOSTICISM.md)
- [Security Policy (POLICY)](./POLICY.md)
