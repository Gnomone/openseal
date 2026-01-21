# [계획서] OpenSeal Universal Dependency Ghosting 구현

개발자가 별도의 수동 작업 없이도 소스 코드를 봉인(Seal)하고 즉시 실행할 수 있도록, 프로젝트 환경에 맞는 의존성을 자동으로 감지하고 연결하는 기능을 구현합니다.

## 1. 개요
*   **목표**: `openseal build` 시 원본의 의존성(Library)을 감지하고, 출력물 폴더(`dist_opensealed`) 내부에 자동으로 심볼릭 링크를 생성하여 `openseal run` 시 즉시 실행 가능하게 함.
*   **지원 범위**: Node.js (`node_modules`), Python (`venv`, `site-packages`), Go/Rust (필요시 아티팩트 매핑)

---

## 2. 세부 설계

### A. 프로젝트 유형 감지 (Project Discovery)
`openseal build` 프로세스 중에 다음과 같은 파일을 감지합니다:
- `package.json` -> **Node.js 모드** (기본: `node_modules`)
- `requirements.txt`, `pyproject.toml`, `venv/` -> **Python 모드** (기본: `venv`, `.venv`)
- `go.mod` -> **Go 모드**

### B. 의존성 고스팅 (Ghosting Logic)
빌드 결과물 폴더에 다음과 같은 심볼릭 링크를 자동 생성합니다:
1.  **Node.js**:
    - 원본의 `node_modules`가 존재하면 `output/node_modules`로 심볼릭 링크 생성.
2.  **Python**:
    - `.venv` 또는 `venv` 폴더가 존재하면 링크 생성.
    - (고도화) 런타임 환경 변수에 `VIRTUAL_ENV` 경로 주입.

### C. 커스텀 경로 지원 (Explicit Mapping)
표준 이름을 사용하지 않는 경우를 위해 수동 설정 옵션을 제공합니다.
- **CLI Flag**: `openseal build --exec "..." --deps my_custom_libs`
- **--exec 설명**: 봉인된 환경에서 실행할 실제 진입 명령어 (예: `npm run dev`)
- **--deps 설명**: 무결성 체크(A-Hash)에서 제외하고 실행 환경에 연결할 의존성 폴더 (예: `venv`)
- **Manifest**: `openseal.json` 내의 `"deps": "path/to/libs"` 필드 참조.
- **작동**: 자동 감지보다 수동 설정값이 우선순위를 가짐.

### D. 보안 가드 (Security Guards)
- **추적 금지**: 자동 생성된 심볼릭 링크는 A-Hash 계산(무결성 체크)에서 반드시 제외되어야 함. (이미 `.opensealignore` 기본값에 포함되어 있음)
- **사용자 알림**: "Found node_modules, automatically ghosting to build directory..." 메시지 출력으로 투명성 확보.

---

## 3. 구현 단계
1.  **crates/openseal-cli**: `build` 커맨드 로직에 `detect_and_link_dependencies` 함수 추가.
2.  **crates/openseal-core**: (필요시) 프로젝트 유형 감지 유틸리티 강화.
3.  **docs/USAGE_KR.md**: `--deps` 옵션 설명 및 언어별(Node, Python, Go) 가이드 추가.
4.  **검증**: `crypto-price-oracle` (Node) 및 테스트용 Python 프로젝트에서 `ln -s` 없이 성공 여부 확인.

---

**결론**: 이 기능이 구현되면 "OpenSeal은 어렵다"는 인식을 깨고, 한 번의 명령으로 안전한 서비스를 즉시 띄울 수 있는 강력한 UX를 제공하게 됩니다.
