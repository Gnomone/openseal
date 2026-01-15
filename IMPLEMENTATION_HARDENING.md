# Hardening Documentation Implementation Plan

## Goal
Transition public documentation to a "Defensive Disclosure" state. Remove all "recipes" for creating Seals while maintaining clear instructions for "Verifying" Seals.

## Changes

### `docs/public/SPEC_PUBLIC.md`
- [MODIFY] Update Boundary Statement with stronger language.
- [MODIFY] Replace "Expected_B_hash" calculation logic with "Consistency Check" language.
- [MODIFY] Remove mathematical hash formulas (e.g., `H(Root || Input || Nonce)`).
- [MODIFY] Redefine A-hash/B-hash simply as "Pre-execution Identity" and "Post-execution Identity".
- [MODIFY] Mask the `b_G` dynamic function logic and Nonce injection points.

### `docs/public/ARCHITECTURE.md`
- [MODIFY] Soften "State Transition" diagrams to prevent reconstruction of the state machine.
- [MODIFY] Ensure flow descriptions explain *what* happens, not *how* to replicate it.

### `README_KR.md` / `README.md`
- [MODIFY] Add `Security Disclosure Note` warning about the non-reproducibility of Seals.

## Verification
### Manual Review
1. **Word Check**: Grep for forbidden words (`compute`, `derive`, `function`, `transform`) in public docs.
2. **Logic Check**: Read `SPEC_PUBLIC.md` as an attacker. Can I build a Python script to generate a valid B-hash from a Result?
   - The answer must be "No".
3. **Consistency Check**: Ensure the "Verifier" role is still actionable conceptually (relying on SDK/Lib blackbox).
