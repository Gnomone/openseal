# OpenSeal v2.0 Whitepaper Implementation Plan

## Goal
Create a definitive Whitepaper that explains the "Why" and "Trust Model" of OpenSeal, complementing the "How" of the Spec.
This document will serve as the "Limit Declaration" and "Security Argument" to preemptively address "Black Box" criticisms.

## New Documents
### 1. `docs/public/WHITEPAPER.md` (English)
- **Structure**:
    1. **Abstract**: Execution Honesty Enforcement.
    2. **Threat Model**: Malicious Operator, Network Attacker.
    3. **Problem**: Why results are unverifiable.
    4. **Core Insight**: Result = State Transition Evidence.
    5. **OpenSeal Model**: Atomic Execution Boundary, One-Way Seal.
    6. **Security Argument**: Cost of Forgery > Cost of Honest Execution.
    7. **What OpenSeal Does NOT Guarantee**: Semantic correctness, bias, data source truth.
    8. **Philosophy**: Public Verification, Monopolized Generation.

### 2. `docs/public/WHITEPAPER_KR.md` (Korean)
- Translation of the above.

## Updates
- **`README.md` & `README_KR.md`**: Add prominent links to the Whitepaper.

## Verification
- Review content against the "8-point structure" provided by the user.
- Ensure clear distinction from `SPEC_PUBLIC.md` (No implementation details).
- Verify "Limit Declaration" clearly states what is NOT covered.
