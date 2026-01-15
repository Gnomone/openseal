[🇰🇷 한국어 버전 (Korean Version)](./README_KR.md)

# 🔐 OpenSeal: Atomic Project Sealing Protocol

> **"OpenSeal does not modify application code. It replaces the caller."**

OpenSeal is an open standard and toolkit for ensuring the integrity of entire API service environments. It shifts from unit-level sealing to **Atomic Project Sealing**, where the service's identity is defined by a Merkle Tree of its entire codebase.

---

## 🚀 OpenSeal v2.0: The Evolution

To overcome the network limitations of WASM and prevent orchestration tampering, v2.0 introduces the **Atomic Project Sealing** model.

*   **WASM-Free**: Uses a hardened native runtime for native performance and network access.
*   **Full Integrity**: Seals the entire project (via Merkle Tree) instead of a single file.
*   **Caller Monopoly**: Replaces the execution caller to capture results as state transitions.

> 📖 **[Read the Whitepaper](./docs/public/WHITEPAPER.md)**: Understanding the "Trust of Execution" Model.

### 🛡️ Security Disclosure Note
This project intentionally does **NOT** disclose:
- How valid seals are generated.
- Any reproducible hash construction.
- Any function or recipe to bind inputs to outputs.

**Any attempt to recreate a seal generator based solely on this repository is expected to fail by design.**

---

### 2.2 Specification

* [5-Minute Quickstart Guide](./docs/public/QUICKSTART.md)
* [7325 Protocol Spec (OSIP-7325.md)](./docs/public/OSIP-7325.md)
* [Public Verification Spec](./docs/pending/SPEC_PUBLIC.md)
* [Architecture](./docs/pending/ARCHITECTURE.md)
* [Disclosure Policy (OPENSEAL_DISCLOSURE_POLICY.md)](./docs/public/OPENSEAL_DISCLOSURE_POLICY.md)

---

## 🏗️ Service Architecture (The Flow)

OpenSeal treats the API server as a "Case" (container) and the execution as an "Event" (sealed).

```mermaid
sequenceDiagram
    participant Client as User (📱)
    participant Case as API Server (Python/JS)
    participant Event as OpenSeal Runtime (Event)
    participant HS as HighStation (Server)

    Client->>Case: 1. Service Request (inc. Wax)
    Case->>Event: 2. Execution Request (Raw Input + Wax)
    
    Note over Event: [Atomic Pipeline]
    Event->>Event: ① Merkle Root based A-hash generation
    Event->>Event: ② Business Logic Execution
    Event->>Event: ③ b_G function based B-hash generation
    
    Event-->>Case: 3. Result + Proof Return
    Case->>HS: 4. (Optional) HighStation Verification & Signing
    Case-->>Client: 5. Final Response (Result + Seal)
```

---

---

## 3. Sealing Configurations & Exclusion Rules

OpenSeal determines which files to include in the Merkle Tree based on the following rules.

### 3.1 Total Exclusion
These files are completely excluded from the **File Integrity Check (A-hash)** and will not be included in the build output.
*   **Base Directory**: All paths are relative to the **Project Root** (the source path provided during `openseal build`).
*   **`.gitignore`**: Respects standard Git ignore rules.
*   **`.opensealignore`**: OpenSeal-specific exclusion rules. Uses the same syntax as `.gitignore`. Any pattern defined here is **100% ignored**, from its existence in the hash to its presence in the output.

> [!IMPORTANT]
> **Integrity of Configurations**: `.opensealignore`, `.openseal_mutable`, and the generated `openseal.json` themselves **are included** in the **File Integrity Check (A-hash)**. This prevents attackers from modifying exclusion rules to bypass security.

### 3.2 Content Exclusion (Mutable Files)
Used when you want to seal the **existence (structure)** of a file but allow its **content** to change (e.g., local databases, logs).
*   **`.openseal_mutable`**: Files matching patterns in this file (e.g., `*.db`, `logs/`) will not affect the overall integrity proof even if their content changes.
*   **Security**: For security reasons, setting critical code files (`.js`, `.py`, `.rs`, etc.) as mutable is strictly forbidden and will be rejected by the runtime.

---

## 4. Components

| Crate | Description |
| :--- | :--- |
| **`crates/openseal-core`** | The heart of v2.0. Merkle Tree scanner and atomic hashing logic. |
| **`crates/openseal-runtime`** | Hardened native runtime that replaces the caller and captures state. |
| **`crates/openseal-cli`** | Command-line tool for project initialization and sealing packaging. |

---

## 🛠️ Usage (Vision)

### 1. Sealing a Project (`openseal build`)
Scan the source code to determine Identity, and package it including the execution command (`--exec`).

```bash
# Node.js project example
openseal build --source . --output dist --exec "node app.js"
```

### 2. Running a Sealed Service (`openseal run`)
OpenSeal becomes the **Parent Process**, spawning the application as a child process and isolating it. External access is only possible via the OpenSeal Proxy (7325).

```bash
# Standard Execution
openseal run --app ./dist --port 7325
```

---

## 🔒 Security Model

### What it Protects
*   **Source Code Tampering**: Modifying even 1 byte of code or environment before execution will cause sealing authentication to fail.
*   **Result Manipulation**: Mathematically guarantees that the result was honestly produced by that specific source code. (Result-Code Binding)

### Limitations (The ROOT Problem)
A ROOT-level attacker with real-time measurement can theoretically tamper with memory. However, OpenSeal is designed such that **"the cost of forgery is greater than or equal to the cost of honest execution,"** achieving economic integrity.

---

> **OpenSeal: The return value is never trusted as data — it is consumed as a state transition inside a sealed runtime.**
