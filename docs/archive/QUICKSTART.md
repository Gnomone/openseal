# ⚡ OpenSeal 5-Minute Quickstart: Protect Your API

This guide walks you through using OpenSeal to protect your API services and demonstrates how to immediately detect code tampering.

---

## 🚀 Step-by-Step Tutorial

### Step 0: Install OpenSeal CLI
To use OpenSeal commands anywhere, you first need to build and install the CLI to your system path.

```bash
# Verify the path (assuming you are in the project root)
cd openseal

# Build and install the CLI
cargo install --path ./crates/openseal-cli

# Verify installation
openseal --version
```

### Step 1: Prepare Sample Project
Prepare the **Sentence Laundry** API project (also known as Messy Talker) for testing.

```bash
# Clone the sample repository and navigate to it
git clone https://github.com/kjyyoung/crypto-price-oracle
cd crypto-price-oracle

# Activate virtual environment (Recommended for Python)
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

> [!TIP]
> **Why activate the virtual environment (venv)?**  
> OpenSeal automatically excludes the `venv/` folder from integrity checks via `.opensealignore`, ensuring that only the pure source code identity (A-hash) is captured.

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

> [!IMPORTANT]
> **Dynamic Port Allocation**  
> OpenSeal assigns a random "Hidden Internal Port" to the application. The target application **must** be configured to listen on the port specified by the `PORT` environment variable.

### Step 4: Verify Normal Operation
Call the API, save the result, and verify it using `openseal verify`.

```bash
# 1. Call API and save response to file
curl -X POST http://127.0.0.1:7325/wash \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: my-secret-session-123" \
  -d '{"text": "The weather is really nice today."}' > response.json

# 2. Perform Integrity Verification
openseal verify --response response.json --wax "my-secret-session-123"
```

**Result**:
> ✅ **Signature Valid**  
> ✅ **Binding Valid**  
> "✅ SEAL VALID. The result is authentic and untampered."

---

### Step 5: Demonstrate Tampering (Attack)
Now, intentionally modify the source code to see how OpenSeal detects it (Identity Mismatch).

1. Open `translator.py` and modify a comment or logic slightly.
2. Re-build and run (`openseal build ...` -> `openseal run ...`).
   - A new `Root A-Hash` is generated. However, the verifier will check against the **Original A-Hash** noted in Step 2.
3. Call API and Verify. (⚠️ Copy the **Original A-Hash** to `--root-hash`)

```bash
# 1. Call API (Save result)
curl -X POST http://127.0.0.1:7325/wash ... > tampered.json

# 2. Verify against Original Hash
openseal verify --response tampered.json --wax "my-secret-session-123" --root-hash <ORIGINAL_A_HASH>
```

**Result**:
> ❌ **Identity Valid**: ❌  
> "Identity Mismatch. The code executed is different from what was expected."

The verifier correctly detects that the running code differs from the original. This is the core value of OpenSeal.

---

## 🛡️ Using Exclusion Rules
- **Total Exclusion**: Add `venv/` to `.opensealignore`.
- **Content-only Exclusion (Mutable)**: Add `*.log` to `.openseal_mutable`.

---

## 💡 Next Steps
- [Protocol Specification (OSIP-7325.md)](./OSIP-7325.md)
- [Security Model (SECURITY_MODEL.md)](./SECURITY_MODEL.md)
