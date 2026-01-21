# [계획서] HighStation 전문 워크플로우 및 인증 표준화

현업에서의 표준(Industry Standard)은 단순히 기능을 만드는 것뿐만 아니라, **"실제와 동일한 개발 환경(Dev-Prod Parity)"**을 유지하며 브랜치를 관리하는 것입니다. 현재 로컬 테스트가 어려운 문제를 해결하고 전문적인 브랜치 전략을 수립합니다.

## 1. 인증 시스템 현행화 (Professional Auth Standard)

현재 로컬에서 인증이 "대충" 넘어가는 문제를 해결하기 위한 전략입니다.

### [백엔드] "Structured Local Auth" 도입
*   **문제**: 단순 바이패스는 DB의 RLS(Row Level Security)나 사용자 프로필 기반 로직을 검증하지 못합니다.
*   **해결**: `.env.local`에 정의된 `DEV_USER_ID`와 `DEV_USER_EMAIL`을 사용하는 **고정 개발자 정체성**을 구현합니다.
    *   인증이 성공한 것으로 간주하되, 실제 DB의 `profiles` 테이블과 연동하여 로직을 수행합니다.
    *   마이그레이션 파일(`000_init.sql`)에 해당 개발자 계정을 미리 인서트해두어, 로컬 실행 즉시 "내 서비스 등록" 등이 가능하게 합니다.

### [프론트엔드] "Mocked Auth Sync"
*   **문제**: 프론트엔드에서 `testConnection` 등 API 호출 시 토큰이 없어 실패하는 경우가 발생합니다.
*   **해결**: Vite 개발 모드(`dev`)에서 `useAuth` 훅이 백엔드가 기대하는 개발자 정체성(Header 등)을 자동으로 포함하도록 수정합니다.

---

## 2. 브랜치 전략 (Feature Branching)

작업 단위를 명확히 나누어 메인 브랜치의 안정성을 확보합니다.

### 🌿 `feat/service-registration-flow` 신규 생성
*   기존에 작업한 `CreateService.tsx` 및 `services.ts`의 변경사항을 이 브랜치로 격리합니다.
*   **작업 내용**:
    *   OpenSeal 연동 무결성 감사(Audit) 로직 완성
    *   위저드 UI/UX 개선 (슬라이딩 애니메이션 등)
    *   로컬 인증 표준화 로직 적용

---

## 3. 진행 제안 (Next Steps)

1.  **Step 1**: 백엔드 `authMiddleware.ts`를 수정하여 로컬 전용 "Standard Identity" 로직을 심습니다.
2.  **Step 2**: 프론트엔드 API 클라이언트가 로컬 개발 시 해당 정체성을 사용하도록 동기화합니다.
3.  **Step 3**: `main` 브랜치에서 분기하여 `feat/service-registration-flow`를 정식으로 운영합니다.

---

**판단 결과**: 현업 수준의 개발 품질을 위해서는 **"정교한 로컬 인증 모의(Simulation)"**가 선행되어야 합니다. 그래야 브랜치를 나눈 보람(실제 테스트 가능)이 생깁니다.

이 방향으로 먼저 인증 환경을 고도화한 뒤 브랜치 작업을 진행할까요?
