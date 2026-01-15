# 📡 OpenSeal v2.0 Release Preparation Plan

## 1. 개요 (Overview)
OpenSeal v2.0의 핵심 기능인 **Caller Monopoly (`openseal run`)** 구현이 완료되었습니다.  
이제 프로젝트를 공식적으로 릴리즈하기 위해 **저장소 정리, 정책 문서화, 버전 태깅**을 수행해야 합니다.

본 문서는 안전하고 체계적인 배포를 위한 단계별 계획입니다.

---

## 2. 작업 상태 점검 (Status Check)

| 항목 | 상태 | 비고 |
|:---:|:---:|:---|
| **Core Logic** | ✅ 완료 | `A-hash`, `B-hash`, `Nonce` 검증, 가변 파일 처리 |
| **Runtime** | ✅ 완료 | `openseal-runtime` 라이브러리화, Proxy 구현 |
| **CLI** | ✅ 완료 | `build`, `run` (Child Process 제어) 구현 |
| **Verification** | ✅ 완료 | `verify_openseal.sh` E2E 테스트 통과 |
| **Documentation** | ✅ 완료 | `README`, `ARCHITECTURE`, `DISCLOSURE_POLICY` 작성 |

---

## 3. 실행 계획 (Action Items)

### Step 1: 모노레포 구조 정리 (Git Cleanup)
현재 `crates/` 하위 디렉토리들에 개별 `.git` 폴더가 존재하여 메인 저장소(Root) 커밋을 방해하고 있습니다. 이를 정리하여 단일 저장소(Monorepo)로 만듭니다.

- [ ] `crates/openseal-cli/.git` 제거
- [ ] `crates/openseal-core/.git` 제거
- [ ] `crates/openseal-runtime/.git` 제거
- [ ] Root `.git` 초기화 및 전체 파일 스테이징 (`git add .`)

### Step 2: 버전 확정 및 태깅 (Tagging)
v2.0의 기능을 동결(Freeze)하고 릴리즈 후보(RC) 태그를 생성합니다.

### Step 3: 보안 검토 및 동결 (Security Review & Freeze)
(Reserved) 공개 전 보안성을 최종 점검하는 단계입니다.
- 내부 명세(`SPEC_INTERNAL.md`)의 무결성 검토
- 공개 문서(`SPEC_PUBLIC.md`)를 통한 위조 가능성 재점검
- **"검증 가능하되 위조 불가능한"** 경계 확인

### Step 4: 문서 분류 및 구조화 (Documentation Partitioning)
**"검증은 민주화하되, 생성은 독점한다"**는 철학에 따라 문서를 재구성합니다.

#### 📂 `docs/public` (검증 전용 / 완전 공개)
위조 레시피가 될 수 있는 구체적 생성 규칙은 제거하고, "검증"에 필요한 인터페이스만 남깁니다.
- `SPEC_PUBLIC.md` (New): `SPEC_KR.md`를 정제하여 검증 인터페이스만 남긴 공개 명세.
- `ARCHITECTURE.md`: 시스템 설계 및 철학
- `OPENSEAL_DISCLOSURE_POLICY.md`: 공개/비공개 범위 지시문
- `README_KR.md`: 프로젝트 소개

#### 🔒 `docs/internal` (생성 규칙 / 비공개)
Fake Prover 제작에 악용될 수 있는 핵심 생성 로직을 포함합니다.
> **⚠️ 본 문서는 OpenSeal 구현 및 보안 유지를 위해 비공개로 관리되며, 본 저장소의 공개 릴리즈에는 포함되지 않습니다.**
- `SPEC_INTERNAL.md`: 기존 `SPEC_KR.md`의 내용을 포함한 상세 생성 규칙 (Hash Recipe 포함).
- `IMPLEMENTATION_PLAN.md`: 내부 구현 계획
- (Future) `CORE_GENERATOR_LOGIC.md`: 동적 함수 생성기 설계도

### Step 5: SPEC 문서 분리 작업 (Refinement)
- `SPEC_KR.md`를 `docs/internal/SPEC_INTERNAL.md`로 이동 (보존).
- `SPEC_PUBLIC.md` 새로 작성:
    - 구체적 해시 결합 순서 제거.
    - "A-hash는 실행 이전 상태 식별자" 등으로 추상화.
    - **"본 명세는 검증 규칙만을 정의하며, 생성 방법은 포함하지 않는다"** 문구 추가.

---

## 4. 최종 확인 (Final Confirmation)
본 계획은 OpenSeal v2.0의 공개 릴리즈 기준안입니다.
문서 구조 및 공개 범위에 대한 이의가 없다면, 본 계획에 따라 릴리즈를 진행합니다.
