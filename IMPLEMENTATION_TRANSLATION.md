# Documentation Translation Plan

## Goal
Release documentation in English by default, with Korean translations available via toggle/link.

## Files to Create (English)
1. **`docs/public/SPEC_PUBLIC.md`**:
   - Translated from `SPEC_PUBLIC_KR.md`
   - Must adhere to "Hardening Directive" (No recipes, only verification assertions).
   - Add link to Korean version.

2. **`docs/public/ARCHITECTURE.md`**:
   - Translated from `ARCHITECTURE_KR.md`
   - "State Transition" -> "Atomic Event Assertion"
   - Hardened diagram descriptions.
   - Add link to Korean version.

3. **`docs/public/OPENSEAL_DISCLOSURE_POLICY.md`**:
   - Translated from `OPENSEAL_DISCLOSURE_POLICY_KR.md`
   - Core boundary definitions.
   - Add link to Korean version.

## Updates
- **`README.md`**: Update links to point to English docs in `docs/public/*.md`.
- **`README_KR.md`**: Update links to point to `docs/public/*_KR.md`.
- **Cross-links**: Each document should have a header link to the other language.

## Verification
- Check all links work.
- Check "Hardening Directive" is preserved in English (no `compute`, `derive`).
