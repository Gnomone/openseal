# 🔍 OpenSeal v2.0 Public Verification Spec

[🇰🇷 한국어 버전 (Korean Version)](./SPEC_PUBLIC_KR.md)

---

> **⚠️ Boundary Statement**
> This document is intended to explain **OpenSeal's Verifiability**.
> It intentionally does **NOT** include methods to **generate or reproduce a valid Seal**.
> Generation rules, binding order, and internal state transitions are essentially omitted or abstracted.

---

## 1. Overview
The OpenSeal Verifier determines whether a given Result and Seal **"represent an internally consistent state transition within a registered execution context."**

### Verification Model
```text
Verify(Result, Seal, PublicKeys) -> VALID | INVALID
```

---

## 2. Data Structure

### 2.1 Seal
The proof object returned by the OpenSeal Runtime.

| Field | Description | Verifiable |
|:---:|:---|:---:|
| `a_hash` | **Pre-State ID** | ✅ Public Assertion |
| `b_hash` | **Post-State ID** | ✅ Public Assertion |
| `nonce` | External Identifier for **Uniqueness** | ✅ Public Assertion |
| `signature` | Digital Signature by OpenSeal Runtime over the above data | ✅ Public Assertion |

### 2.2 Result
The actual response data (JSON, String, Binary, etc.) returned by the API server. OpenSeal treats this data not as a "value" but as "evidence of a state assertion."

---

## 3. Verification Process

Callers must perform the following steps to determine validity.

### Step 1: A-hash Verification (Identity Check)
*   **Concept**: "Did this result come from the project I know?"
*   **Method**: Check if the local source code (or known Merkle Root) matches `Seal.a_hash`.

### Step 2: B-hash Verification (Binding Check)
*   **Concept**: "Is the result unmanipulated and from this specific execution context (Nonce)?"
*   **Method**: The Verifier **Asserts** that the Seal contains a proof that **does not contradict the execution result**.
    *   *Note: Internal seal logic varies per request. Its structure cannot be predicted or reused externally.*

### Step 3: Signature Verification (Authenticity Check)
*   **Concept**: "Is this seal signed by a trusted OpenSeal Runtime?"
*   **Method**: Verify `Seal.signature` against the `Seal` content.
*   **Key Lifecycle (Security)**: The signing key (`Pub-Key`) is an **Ephemeral Session Key** generated in RAM at runtime startup. It is **never persisted** to disk. Restarting the runtime creates a new identity, ensuring Forward Secrecy.

---

## 4. Failure Cases

The verification must return **INVALID** if any of the following occur:

1.  **Identity Mismatch**: Submitted `A-hash` differs from the expected identifier.
2.  **Binding Failure**: Seal is logically inconsistent with the execution result.
3.  **Replay Attack**: The `Nonce` has already been used. (Verification fails on reuse)
4.  **Signature Error**: Signature verification fails.

---

## 5. Conclusion
A Verifier implemented according to this spec can mathematically ascertain the **"Integrity of the Result."**
However, this spec alone does not allow one to **Generate** a valid `Seal`, which is central to OpenSeal's security model.
