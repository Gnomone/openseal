# 🛠️ OpenSeal: 사용법 및 안전 가이드

OpenSeal 서비스의 설정, 실행 및 안전한 관리 방법을 다룹니다.

---

## 1. 5분 퀵스타트

### 1단계: CLI 설치
```bash
# 최신 바이너리를 다운로드하여 설치합니다.
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash
```

### 2단계: 봉인 (Build)
```bash
# 프로젝트 루트에서 실행하세요.
# --output 옵션으로 결과물 폴더를 따로 지정하세요 ('dist_opensealed' 등) - 기존 dist 폴더 보호
openseal build --exec "node app.js" --output dist_opensealed
```

### 3단계: 실행 (봉인 활성화)
```bash
# 원하는 포트 지정 (예: 3000)
# OpenSeal이 내부 포트 충돌을 자동으로 해결합니다.
openseal run --app dist_opensealed --port 3000
```

---

## 2. 언어별 퀵스타트 (Quickstart by Language)

OpenSeal은 검증된 소스 코드를 직접 실행(JIT)하는 것을 권장합니다. 각 환경에 맞는 복사-붙여넣기 명령어입니다.

### 🟢 Node.js (TypeScript)
빌드된 `dist` 대신 **소스 코드 무결성**을 위해 `ts-node` 사용을 권장합니다.
```bash
# 빌드
openseal build --exec "npx ts-node src/index.ts" --output dist_opensealed

# 실행
cd dist_opensealed && npm install && cd ..
openseal run --app dist_opensealed --port 3000
```

### 🟡 Python
```bash
# 빌드
openseal build --exec "python main.py" --output dist_opensealed

# 실행 (필요 시 venv 활성화)
openseal run --app dist_opensealed --port 3000
```

### 🔵 Go
```bash
# 빌드
openseal build --exec "go run main.go" --output dist_opensealed

# 실행
openseal run --app dist_opensealed --port 3000
```

### 🦀 Rust
```bash
# 빌드 (target 폴더는 자동으로 무시됩니다)
openseal build --exec "cargo run --release" --output dist_opensealed

# 실행
openseal run --app dist_opensealed --port 3000
```

### 4단계: 검증 (선택사항 - 테스트용)
```bash
# API 응답의 무결성을 검증합니다.
openseal verify --response result.json --wax "난수값" --root-hash "예상-A-hash"
```

**`result.json` 파일 형식:**
OpenSeal 런타임이 생성한 응답 파일은 다음과 같은 구조를 가집니다:
```json
{
  "result": { /* 실제 API 응답 결과 */ },
  "openseal": {
    "signature": "...",  // 서명
    "pub_key": "...",    // 공개키
    "a_hash": "...",     // 코드 정체성
    "b_hash": "..."      // 결과 바인딩
  }
}
```

**검증 내용:**
- ✅ **서명 검증**: `openseal.signature`가 `pub_key`로 검증 가능한지 확인
- ✅ **Wax 일치**: 응답에 포함된 Wax가 요청 시 보낸 난수와 일치하는지 확인
- ✅ **코드 정체성**: (--root-hash 제공 시) `a_hash`가 예상 코드와 일치하는지 확인

**`verify` 명령어를 사용하는 경우:**
- 배포 전 로컬에서 봉인된 애플리케이션 테스트
- API 응답에 유효한 인감(Seal)이 포함되어 있는지 감사
- 인감 생성 문제 디버깅

**참고**: 프로덕션 환경에서는 공급자가 아닌 클라이언트(소비자)가 오픈소스 검증기를 사용하여 검증합니다.

---

## 2. 안전 가드레일 (Safety Guardrails)

OpenSeal은 홈 디렉토리(`/home`) 등 의도치 않은 위치를 봉인하는 것을 방지합니다.

### 프로젝트 자동 탐지
CLI는 프로젝트 표준 파일(`package.json`, `Cargo.toml`, `.git` 등)이 있는지 확인합니다. 파일이 없으면 진행 여부를 묻습니다:
> `⚠️ WARNING: 표준 프로젝트 파일이 탐지되지 않았습니다. 진행할까요? (y/N)`

### 권장 사항
- **루트 실행**: 반드시 소스 코드의 최상위 디렉토리에서 명령어를 실행하세요.
- **무시 목록 확인**: `.opensealignore`를 사용하여 `node_modules`와 같은 대용량 폴더를 제외하세요.

---

## 3. 제외 규칙
- **.opensealignore**: A-hash 계산에서 완전히 제외 (코드 프라이버시).
- **.openseal_mutable**: 파일의 존재는 봉인하되 내용은 변경 가능 (예: 로그, DB).
