# OpenSeal Disclosure Policy (Defensive)

**Last Updated**: 2026-01-16
**Classification**: Public

---

## 1. Overview
OpenSeal adopts a **Defensive Disclosure** strategy. We transparently disclose the existence of the technology and its verification methods to build trust, but we **strictly seal** the core generation logic and internal structures that could aid attackers.

---

## 2. Open Area (Public)
The following information is publicly accessible to ensure transparency and verifiability.

### 2.1 Execution Identity Model
- We disclose the concept that **"Code Execution State (Identity) is defined mathematically via Merkle Tree Hashing."**
- We define the state derived from static code as **A-Hash** and the state bound to the execution result as **B-Hash**.

### 2.2 Verification Specification (Receiver-Side)
- The protocols for verifying the integrity of results are fully open.
- **Specification**: `Input: (Result, A, B, Signature) -> Output: VALID | INVALID`
- We provide CLI tools (`openseal verify`) and libraries to allow anyone to verify integrity.

### 2.3 Attack Vectors and Defenses
- We disclose the threat models we defend against (e.g., Replay Attacks, Code Tampering) and the architectural principles used for defense (e.g., Wax, Caller Monopoly).

---

## 3. Sealed Area (Private)
The following information remains **Sealed** to prevent reverse engineering and forgery.

### 3.1 Internal Implementation of Execution Capsule
- The exact mechanisms of how the Runtime isolates the process and handles internal states are not disclosed.
- Memory layout, entropy injection methods, and specific system call patterns are hidden.

### 3.2 Internal Binding Structure
- The specific hashing logic and internal processing structures related to A-Hash and B-Hash are **Sealed**.
- The algorithm for generating signatures based on non-public entropy injected at request time is not disclosed.
- **Why?** Should this logic be exposed, a sufficiently motivated attacker might attempt to generate valid signatures for forged results without executing the actual code.

---

## 4. Policy on Open Source
While OpenSeal advocates for open source, we prioritize the **security of the ecosystem**.
- **Reference Implementation**: Publicly available code (`openseal-core`) guarantees verification logic transparency.
- **Production Implementation**: The actual Runtime used in production may employ different, sealed internal logic (`openseal-secret`) compared to the public reference to maximize security.
- Users can trust the system through **Verification**, not by inspecting the Generation code.

> **Summary**: "The theory necessary for verification is not hidden, but the internal mechanisms responsible for generation are sealed."
