# ⚡ OpenSeal 5-Minute Quickstart: Protect Your API

This guide walks you through using OpenSeal to protect your API services and demonstrates how to immediately detect code tampering.

---

## 🚀 Step-by-Step Tutorial

### Step 0: Install OpenSeal CLI
To use OpenSeal commands anywhere, you first need to build and install the CLI to your system path.

```bash
# Clone the OpenSeal repository (or navigate to it if already cloned)
git clone https://github.com/kjyyoung/openseal
cd openseal

# Build and install the CLI (Run from the project root)
cargo install --path ./crates/openseal-cli

# Verify installation
openseal --version
```

> [!NOTE]
> After `cargo install` completes, you can use the `openseal` command directly in any directory. Ensure that Rust's `bin` path (`~/.cargo/bin`) is in your PATH.

### Step 1: Prepare Sample Project
Prepare the **Sentence Laundry** API project (also known as Messy Talker) for testing.

```bash
# Clone the sample repository and navigate to it
git clone https://github.com/kjyyoung/sentence-laundry
cd sentence-laundry

# Activate virtual environment (Recommended for Python)
# Use python or python3 depending on your environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

> [!TIP]
> **Why activate the virtual environment (venv)?**  
> It isolates dependencies from the system to prevent conflicts. OpenSeal automatically excludes the `venv/` folder from integrity checks via `.opensealignore`, ensuring that only the pure source code identity (A-hash) is captured.

### Step 2: Seal the Project
Use the `openseal build` command to seal the entire source code with a Merkle Tree and prepare the executable.

```bash
# Build with OpenSeal (Extract source integrity fingerprint & Package)
# --exec: Specifies the entry point command to start the service (executing main.py here).
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
- [Project Architecture Overview (ARCHITECTURE.md)](./ARCHITECTURE.md)
- [Public Technical Specifications (SPEC_PUBLIC.md)](./SPEC_PUBLIC.md)
