# ⚡ OpenSeal 5-Minute Quickstart: Protect Your API

This guide walks you through using OpenSeal to protect your API services and demonstrates how to immediately detect code tampering.

---

## 🚀 Step-by-Step Tutorial

### Step 1: Prepare Sample Project
Prepare the **Sentence Laundry** API project (also known as Messy Talker) for testing. This project provides a "washing" service that translates text through multiple languages.

```bash
# Navigate to the project directory
cd /root/highpass/sentence-laundry

# Install dependencies
pip install -r requirements.txt
```

### Step 2: Seal the Project
Use the `openseal build` command to seal the entire source code with a Merkle Tree and prepare the executable.

```bash
# Build with OpenSeal (Extract source integrity fingerprint & Package)
openseal build --source . --output ./dist --exec "python3 main.py"
```

**Example Output:**
> ✅ **Root A-Hash**: `19bf5835...` (This is the unique identity of your project)  
> 📥 Copied files to build directory.

### Step 3: Run with OpenSeal Runtime
Execute your service within the OpenSeal protective layer. The runtime runs the API as a child process and intercepts all I/O to bind signatures.

```bash
# Run OpenSeal runtime on port 7325
openseal run --app ./dist --port 7325
```

### Step 4: Verify Normal Operation
Call the API and check the **Wax** signature included in the response.

```bash
curl -X POST http://127.0.0.1:7325/wash \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: my-secret-session-123" \
  -d '{"text": "The weather is really nice today."}'
```

**Verification**: Note the signature (`openseal_signature`) returned in the header or JSON. This is your **Golden Truth**. ✅

### Step 5: Demonstrate Tampering (Attack)
Now, intentionally modify the source code to see how OpenSeal detects it.

1. Open `translator.py` and slightly modify the translation logic or text processing (e.g., append a specific word).
2. Re-build and run:
   ```bash
   openseal build --source . --output ./dist --exec "python3 main.py"
   openseal run --app ./dist --port 7325
   ```
3. Execute the **exact same** `curl` command from Step 4.

**Verification**: If even a single byte of source code has been tampered with, the `signature` value in the response **will be completely different.** ❌  
This cryptographically proves that the identity of the code being executed has changed.

---

## 🛡️ Using Exclusion Rules

Files like `venv/` or `__pycache__/` in Python projects should be excluded from integrity checks. OpenSeal respects `.gitignore` by default and allows additional rules via `.opensealignore`.

- **Total Exclusion**: Add `venv/` to `.opensealignore` (ignores the file's existence).
- **Content-only Exclusion (Mutable)**: Add `*.log` to `.openseal_mutable` (verifies existence but ignores content).

---

## 🛡️ Core Policy: Golden Truth
OpenSeal applies the same principle in production environments.

- **Concealment**: Server logic and hash structures are never exposed externally.
- **Proof**: Only the **Golden Truth Signature** for specific contexts (`Wax` + `Input`) is shared with verifiers.
- **Verification**: Verifiers only need to compare the incoming response signature with the shared Golden Truth.

---

## 💡 Next Steps
- [Detailed Security Mechanisms (CORE_MECHANISM_KR.md)](./docs/internal/CORE_MECHANISM_KR.md)
- [Production Deployment Guide (Coming Soon)](./docs/production.md)
