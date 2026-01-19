# 🛠️ OpenSeal: Usage & Safety Guide

This guide covers how to set up, execute, and safely manage your OpenSeal-protected services.

---

## 1. 5-Minute Quickstart

### Step 1: Install CLI
```bash
# One-line installation via GitHub
cargo install --git https://github.com/kjyyoung/openseal.git --bin openseal
```

### Step 2: Seal (Build)
```bash
# Run at your project root
openseal build --exec "node app.js"
```

### Step 3: Run (Sealing Active)
```bash
# Use your original port (e.g., 3000)
# OpenSeal handles internal port redirection automatically.
openseal run --port 3000
```

---

## 2. Safety Guardrails

OpenSeal prevents accidental sealing of unintended locations (like the home directory).

### Project Detection
The CLI checks for standard files (`package.json`, `Cargo.toml`, `.git`, etc.). If missing, it will request interactive confirmation:
> `⚠️ WARNING: No standard project files detected. Proceed anyway? (y/N)`

### Best Practices
- **Run at Root**: Always execute commands at the top-level directory of your source code.
- **Verify Exclusions**: Use `.opensealignore` to exclude large, non-essential folders like `node_modules`.

---

## 3. Rules for Exclusion
- **.opensealignore**: Completely excludes files from A-hash calculation (Code Privacy).
- **.openseal_mutable**: Seals the file's existence, but allows its content to change (e.g., logs, local DB).
