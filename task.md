# Red Team Audit Task List

- [x] **Reconnaissance**
    - [x] Review `docs/public` vs `crates/openseal-core` consistency.
    - [x] Check if "Sealed" logic is actually exposed in source code.
- [x] **Vulnerability Analysis**
    - [x] **Mutable File Bypass**: Can critical logic be made mutable?
    - [x] **Process Isolation**: Is `openseal run` truly isolating the child?
    - [x] **Policy Violation**: Does the repo code contradict the Disclosure Policy?
- [x] **Reporting**
    - [x] Create `RED_TEAM_REPORT_KR.md` (Internal KR only) with findings and criticality.
