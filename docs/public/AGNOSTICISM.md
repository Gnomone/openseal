# 🧶 OpenSeal: Language Agnosticism & Source Integrity

This document explains how the OpenSeal protocol provides universal integrity across any programming language and why we prioritize verifying **Source Code (`src`)** over build artifacts (`dist`).

---

## 1. Core Philosophy: "Source is the Golden Truth"

While many security solutions verify the hash of final executables or binaries, OpenSeal focuses on the **Original Source Code**.

### Why Source Code?
1.  **Auditability**: What developers and users can read and verify is the source code, not the binary.
2.  **Solves Non-determinism**: Even with the same source, binary hashes can vary based on compiler versions, build timestamps, or OS environments. Verification based on source prevents unnecessary hash mismatches.
3.  **Supply Chain Security**: Build artifacts (`dist/`, `build/`) are vulnerable to tampering during the deployment process. OpenSeal creates a "direct path" from verified source to execution.

---

## 2. Language Agnostic Design

OpenSeal does not enforce specific programming language syntax or runtimes. Instead, it automatically detects standard project structures and provides optimal integrity protection guides for each language's conventions.

### Support Strategy
- **Auto-Detection**: Identifies project types via `package.json`, `Cargo.toml`, `requirements.txt`, `go.mod`, etc.
- **Standard Exclusion Rules**: Automatically excludes build by-products (e.g., `node_modules`, `venv`, `target`) from integrity checks to extract the "Pure Logic."

---

## 3. Secure Execution Models

Two core patterns for deploying APIs while maintaining OpenSeal's security integrity.

### Pattern A: JIT (Just-In-Time) Execution (Recommended)
The runtime interprets and executes the source code directly without converting it to JavaScript or binaries.
- **Example**: Using `ts-node` for TypeScript, or `python main.py` for Python.
- **Pro**: Guarantees that the code you see (A-hash) is 100% identical to the code being executed.

### Pattern B: Build within Sealed Environment
If compilation is required for performance reasons, the build is performed directly within the OpenSeal-sealed source code area followed by execution.
- **Example**: `npm run build && npm start`
- **Pro**: Faster execution while the build process itself remains under OpenSeal's control.

---

## 4. Conclusion

OpenSeal shifts the focus of trust from **"Who built this API?"** to **"What code produced this result?"** across any language. We strive for a world where every developer can prove the value of the code they write.
