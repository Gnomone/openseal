# 🧶 OpenSeal: Protocol & Security Model (OSIP-7325)

This document defines the cryptographic specification and core components of the OpenSeal v2.0 protocol.

---

## 1. Protocol Specification

OpenSeal achieves "Result-Code Binding" by combining the project's identity (A-hash), a request-specific challenge (Wax), and the execution result (B-hash).

### Core Components
- **A-hash (Identity)**: A Merkle Tree root hash of the entire project source code.
- **Wax (Challenge)**: A one-time nonce provided by the verifier to ensure the freshness of the result.
- **B-hash (Binding)**: A keyed hash of the execution result, where the key is derived from (A-hash, Wax).
- **Seal**: A cryptographic signature over the (A, B, Wax) bundle.

---

## 2. Verification Process

**Anyone can be a verifier.** OpenSeal's verification is platform-independent, allowing not only providers but also **API consumers (clients)** to independently confirm the integrity of results.

A verifier confirms the integrity of the result through the following steps:

1. **A-hash Audit**: Download the code from a trusted source and compute the root hash directly.
2. **Wax Validation**: Ensure the Wax in the response matches the challenge sent.
3. **Signature Check**: Use the public key to verify that the signature over (A, B, Wax) is valid.

> 💡 **Practical Verification**: For step-by-step CLI instructions, see the [Usage Guide (USAGE)](./USAGE.md).

> 🌐 **Decentralized Trust**: Verification works without any central platform. With just the open-source verifier, anyone, anywhere, anytime can validate results.

---

### Implementation Note (Non-Reproducibility)

This specification defines ONLY the interfaces and guarantee conditions required for verification. The Seal generation process is intentionally non-deterministic and protected. It is not possible to reproduce a valid Seal generator using only the information in this document or the public repository.
