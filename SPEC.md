# 📖 OpenSeal Protocol Technical Specification (v2.0)

> **Status**: Formal Specification v2.0 (Draft)  
> **Topic**: Atomic Project Sealing (APS)  
> **Focus**: Project-wide Integrity & Internalized Sealing

---

## 1. Abstract

OpenSeal v2.0 is an advanced protocol for ensuring the integrity of entire API service environments. It shifts from unit-level sealing (v1.0) to **Atomic Project Sealing**, where the a service's identity is defined by a Merkle Tree of its entire codebase. Trust is established by a hardened runtime that internalizes the sealing logic, making execution itself the proof.

## 2. Definitions

*   **Project Identity (A-hash)**: The Merkle Root hash of the entire project directory (excluding specified variable files like `.env`).
*   **Internalized Sealing**: Embedding the Nonce-handling and B-hash generation logic directly into the application's execution flow via binary instrumentation or middleware.
*   **Hardened Runtime**: An execution environment that encapsulates the application logic in an encrypted/obfuscated container to prevent trivial tampering of live memory.

## 3. Data Structures & Hashing

### 3.1 A-hash: Global Identity (A)
The identity of the entire service context.
*   **Formula**: `A = BLAKE3(MerkleRoot C || Canonical(Input X) || Nonce N || Context E)`
*   **Components**:
    *   `C`: Merkle Root of the project's persistent files.
    *   `X`: Received input (e.g., HTTP request body).
    *   `N`: 32-byte session Nonce.
    *   `E`: Execution metadata.

### 3.2 B-hash: Execution Binding (B)
Binds the result `R` to the identity `A` through a session-specific transformation function $b\_G$.
*   **Formula**: `B = b_G(A, R)`
*   **Mechanism**: $b\_G$ is a dynamic function derived from $A$ and $N$. It is not a static constant.
*   **Effect**: Valid $B$ can only be produced if the correct code ($A$) was executed with the correct input ($X$).

### 3.3 Signature
*   **Payload**: `Version || ServiceID || Nonce || A-hash || B-hash`
*   **Algorithm**: Ed25519 (White-box implementation encouraged).

## 4. Internalization Protocol (The Pipeline)

Every OpenSeal v2.0 compliant application must follow the **Atomic Pipeline**:

1.  **Entry**: On receiving a request, immediately capture Nonce $N$.
2.  **Commitment**: Internally compute $A$ using the project's pre-computed Merkle Root.
3.  **Execution**: Run the business logic.
4.  **Sealing**: Before returning, generate $B$ using the internal $A$ and execution result $R$.
5.  **Output**: Return the payload `{ result, seal: { A, B, N, C } }`.

## 5. Security & Verification

1.  **Trust Model**: Economic Security (Execution Cost $\ge$ Forgery Cost).
2.  **Shadow App Protection**: Since $B$ depends on the internal $A$, a modified "Shadow App" with a different $A'$ cannot produce the $B$ required by the validator for identity $A$.
3.  **Verification**: Clients verify by:
    *   Hashing the provided project bundle (if available) to match $C$.
    *   Re-computing $A$ with $X$ and $N$.
    *   Verifying the Signature against $A, B$, and $N$.

---

> This v2.0 specification replaces v1.0 and focuses on project-wide security and native performance.
