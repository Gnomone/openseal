# Internal Documentation Localization Plan

## Goal
Ensure all internal documentation is in Korean, complying with the user's directive ("내부 문서 등은 한국어로만 작성").

## Actions

### 1. Translate `RED_TEAM_REPORT.md`
- **Source**: `docs/internal/RED_TEAM_REPORT.md` (English)
- **Target**: `docs/internal/RED_TEAM_REPORT_KR.md` (Korean)
- **Content**: Translate findings (Policy Conflict, Mutable File Bypass, Localhost Bypass) into Korean.

### 2. Clean up English Documents in `docs/internal/`
- **Delete**: `docs/internal/RED_TEAM_REPORT.md` (English version)
- **Delete**: `docs/internal/SPEC.md` (English redundancy of `SPEC_INTERNAL.md`)

## Verification
- List `docs/internal/` to confirm only Korean documents exist.
- Verify content of `RED_TEAM_REPORT_KR.md`.
