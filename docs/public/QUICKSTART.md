# ⚡ OpenSeal 5-Minute Quickstart: Protect Your API

This guide walks you through using OpenSeal to protect your API services and demonstrates how to immediately detect code tampering.

---

## 🚀 Step-by-Step Tutorial

### Step 1: Prepare a Sample Project (or Your Own)
Prepare a simple API server for testing. If you don't have one, clone the [OpenSeal Samples](https://github.com/org/openseal-samples) to get started.

```bash
# Example: Cloning the sample repository
git clone https://github.com/org/openseal-samples
cd openseal-samples/nodejs-example
npm install
```

> [!NOTE]
> This sample API is [description pending]. It will include basic numerical operations or text processing examples.

### Step 2: Establish the Golden Truth (Sealing)
Extract a signature of the "Golden Truth"—the most honest state of your project. This signature will be your benchmark for verification.

```bash
# Use openseal-cli to extract the correct signature for a specific input
./openseal seal \
  --app ./nodejs-example \
  --wax "my_first_test_123" \
  --input '{"name": "OpenSeal"}'
```

**Example Output:**
> ⚖️ **Golden Truth Signature**: `a1b2c3d4e5f6...`  
> (Save this value in a notepad)

### Step 3: Run the Server with OpenSeal Runtime
Now, run your service inside the OpenSeal protective envelope.

```bash
# Run the OpenSeal runtime on port 7325 (Internal app is automatically detected and executed)
openseal run ./nodejs-example --port 7325
```

### Step 4: Verify Normal Operation
Call the API and confirm that the signature accompanying the response matches the Golden Truth.

```bash
curl -X POST http://127.0.0.1:7325/greet \
  -H "X-OpenSeal-Wax: my_first_test_123" \
  -H "Content-Type: application/json" \
  -d '{"name": "OpenSeal"}'
```

**Confirmation**: If the `openseal.signature` value in the response JSON matches the value you saved in Step 2, it's **Normal**! ✅

### Step 5: Proof of Tampering (Tampering Attack)
Now, intentionally modify the source code to see how OpenSeal detects it.

1. Open `nodejs-example/index.js` and slightly modify the response text (e.g., change `Hello` to `Hi`).
2. Restart the server: `openseal run ./nodejs-example --port 7325`
3. Run the **exact same** `curl` command from Step 4.

**Confirmation**: The `signature` in the response will now be **completely different.** ❌  
This proves that even a single-byte change in the source code breaks the proof bound to the result.

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
