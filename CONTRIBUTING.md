# Contributing to OpenSeal

We are thrilled that you’re interested in contributing to OpenSeal! Our mission is to build the world's most trusted and developer-friendly execution sealing protocol.

OpenSeal is designed with a clear separation between:
- **Open Verification & Interface Layers**: Community-driven and fully open-source.
- **Protected Execution Runtime**: Maintained by the core team to ensure cryptographic integrity.

## Where to Contribute

We strongly welcome contributions to the following areas:

### 1. Verifiers & SDKs
Help us expand the reach of OpenSeal by building or improving verifiers in different languages (TypeScript, Go, Python, etc.) or porting the verifier to new platforms.

### 2. CLI & UX
The `openseal-cli` is 100% open-source. We value your input on improving the developer experience, adding new commands, or refining the proxy logic.

### 3. Specification (OSIP)
The OSIP-7325 protocol is a living document. Contribution to clarifications, edge-case documentation, and security analysis is highly encouraged.

### 4. Test Vectors & Security Research
We value adversarial testing. If you find edge cases where verification might be bypassed, or have ideas for stronger threat models, please open an issue or submit a pull request with new test vectors.

## Our Philosophy

While the Seal generation engine itself is protected to prevent the creation of "Fake Sealers," **its behavior is fully specified and verifiable.** We believe that trust comes from the ability to verify results, and we highly value community input on the surrounding trust surface.

---

By contributing, you agree that your contributions will be licensed under the project's **Apache License 2.0**.
