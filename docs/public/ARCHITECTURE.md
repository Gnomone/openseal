# 🔐 OpenSeal: Atomic Project Sealing (v2.0)

[🇰🇷 한국어 버전 (Korean Version)](./ARCHITECTURE_KR.md)

---

> **"OpenSeal does not modify application code. It replaces the caller."**

OpenSeal is a standard for **Call Boundary Protection**, ensuring the integrity of the entire API server environment.

---

## 1. The Philosophy

### ① Event vs Case
*   **Event**: The actual execution of business logic. OpenSeal encapsulates this as a single 'Event' by monopolizing the execution environment.
*   **Case**: The shell that transports data (Django, Express, etc.). It handles the Event's identity but cannot access internal data.

### ② Redefinition of Return Value: "Atomic Event Assertion"
*   In the OpenSeal environment, the code's `return` value is not trusted as external data.
*   It is a **signal proving the internal state of the capsule**. It is immediately consumed by the runtime and converted into a Seal (`B-hash`) before entering the outside world.

---

## 2. Security Architecture: Caller Monopoly

### ① Execution Context Control
*   OpenSeal does not modify source code; instead, it fully controls the **runtime context** where the code executes.
*   **Execution Isolation**: The parent process (OpenSeal) strictly controls the I/O and memory boundaries of the child process (App).

### ② Dynamic Verification
*   The sealing logic has a **non-deterministic structure** that changes dynamically with each request.
*   Even if an attacker observes the running state, the internal verification logic changes for the next request, making reuse and post-execution forgery impossible.

---

## 3. The Strategy

### 🔑 Zero-Edit
*   Developers code as usual.
*   OpenSeal wraps the API's **Call Boundary**, enforcing a structure where "you cannot create a corresponding seal without executing."

### 🔑 Economic Integrity
*   This model does not claim to make it 'impossible' for a ROOT attacker to tamper with memory in real-time.
*   Instead, it aims to **"make the cost of forgery greater than the cost of honest execution,"** thereby achieving practical integrity.

---

> **OpenSeal: The return value is never trusted as data — it is consumed as a state assertion inside a sealed runtime.**

---

## 4. Intuitive Flow

From the user (developer) perspective, OpenSeal operates simply:

```mermaid
graph TD
    A[📂 Source Code Repo] -->|openseal build| B[📦 Sealed Bundle]
    B -->|Identity Check| C{OpenSeal Runtime}
    
    subgraph Caller Monopoly [Caller Monopoly Zone]
        C -->|Spawn| D[🔒 Child Process (API Server)]
        E[User Request] -->|Context Injection| C
        C --Proxy--> D
        D --Result--> C
        C -->|P(Event)| F[Proof Binding]
    end

    F -->|Response + Seal| G[Client]
```

1.  **Build (`openseal build`)**:
    *   Scans the repository source code to determine the `A-hash` (Pre-operation Identity) and packages it.
    *   The API server itself is unmodified.

2.  **Run (`openseal run`)**:
    *   OpenSeal runs the API server as a child process and isolates it.
    *   External access is blocked; communication is only possible via OpenSeal.

3.  **Sealing**:
    *   **Input**: The API server receives a unique identifier (`Wax`) upon execution.
    *   **Execution**: The code is guaranteed to run exactly as it exists in the repository.
    *   **Output**: The execution result is converted by the runtime into a Seal proving **"the result could not be forged without execution."**
