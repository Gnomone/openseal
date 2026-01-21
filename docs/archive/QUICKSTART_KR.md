# ⚡ OpenSeal 5분 퀵스타트: 내 API 보호하기

이 가이드는 OpenSeal을 사용하여 자신의 API 서비스를 보호하고, 코드 변조 시 이를 어떻게 즉시 감지할 수 있는지 실습하는 과정을 담고 있습니다.

---

## 🚀 단계별 튜토리얼

### 0단계: OpenSeal CLI 설치
OpenSeal 명령어를 어디서나 사용하려면 먼저 CLI를 빌드하여 시스템 경로에 등록해야 합니다.

```bash
# OpenSeal 레포지토리 클론 (또는 이미 있다면 해당 디렉토리로 이동)
git clone https://github.com/Gnomone/openseal
cd openseal

# CLI 빌드 및 설치 (프로젝트 루트에서 실행)
cargo install --path ./crates/openseal-cli

# 설치 확인
openseal --version
```

> [!NOTE]
> `cargo install`이 완료되면 `openseal` 명령어를 모든 디렉토리에서 바로 사용할 수 있습니다. Rust의 `bin` 경로(`~/.cargo/bin`)가 PATH에 등록되어 있어야 합니다.

### 1단계: 샘플 프로젝트 준비
테스트를 위해 **문장세탁기 (Sentence Laundry)** API 프로젝트를 준비합니다.

```bash
# 샘플 레포지토리 클론 및 이동
git clone https://github.com/kjyyoung/crypto-price-oracle
cd crypto-price-oracle

# 가상 환경 활성화 (Python 프로젝트 권장 사항)
# 환경에 따라 python 또는 python3를 사용하세요.
python3 -m venv venv
source venv/bin/activate

# 의존성 설치
pip install -r requirements.txt
```

> [!TIP]
> **왜 가상 환경(venv)을 활성화하나요?**  
> 시스템 환경과 분리하여 의존성 충돌을 방지하기 위함입니다. OpenSeal은 `.opensealignore` 규칙을 통해 가상 환경 폴더(`venv/`)를 무결성 검사에서 자동으로 제외하므로, 순수 소스코드의 정체성(A-hash)만 정확히 추출할 수 있습니다.

### 2단계: 프로젝트 봉인 (Sealing)
`openseal build` 명령어를 사용하여 전체 소스코드를 머클 트리로 봉인하고 실행 파일을 준비합니다.

```bash
# OpenSeal로 빌드 (소스코드 무결성 지문 추출 및 패키징)
# --exec: 서비스 실행을 위한 엔트리포인트 명령어를 지정합니다. (여기선 main.py 실행)
openseal build --source . --output ./dist --exec "python3 main.py"
```

**출력 결과 예시:**
> ✅ **Root A-Hash**: `19bf5835...` (이 값이 프로젝트의 유일한 정체성입니다. **이 값을 꼭 메모해두세요!**)  
> 📥 Copied files to build directory.

### 3단계: OpenSeal 런타임으로 서버 실행
이제 서비스를 OpenSeal 보호막 안에서 실행합니다. 런타임은 자식 프로세스로 API를 실행하며 모든 입출력을 가로채 서명을 결합합니다.

```bash
# 7325 포트로 OpenSeal 런타임 실행
openseal run --app ./dist --port 7325
```

> [!IMPORTANT]
> **포트 자동 할당 및 환경 변수**  
> OpenSeal은 보안을 위해 내부 애플리케이션에 랜덤 포트(Hidden Internal Port)를 할당합니다. 따라서 보호 대상 애플리케이션은 반드시 `PORT` 환경 변수를 읽어 해당 포트에서 대기하도록 구현되어야 합니다. (예: Python의 `int(os.environ.get("PORT", 8001))` )

### 4단계: 정상 상태 확인 (Verification)
API를 호출하고 그 결과를 파일로 저장한 뒤, `openseal verify`로 검증합니다.

```bash
# 1. API 호출 결과를 파일(response.json)로 저장
curl -X POST http://127.0.0.1:7325/wash \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: my-secret-session-123" \
  -d '{"text": "The weather is really nice today."}' > response.json

# 2. 무결성 검증 수행
openseal verify --response response.json --wax "my-secret-session-123"
```
**결과 확인**:
> ✅ **Signature Valid**  
> ✅ **Binding Valid**  
> "SEAL VALID. The result is authentic and untampered."

---

### 5단계: 변조 실증 (Tampering Attack)
이제 소스코드를 고의로 수정하여 OpenSeal 검증기가 이를 어떻게 감지하는지(Identity Mismatch) 확인합니다.

1. `translator.py` 또는 `main.py`에 주석을 한 줄 추가하거나 코드를 살짝 수정합니다.
2. 다시 빌드하고 실행합니다 (`openseal build ...` -> `openseal run ...`).
   - 이때 새로운 `Root A-Hash`가 생성됩니다. 하지만 검증자는 **2단계에서 메모해둔 원본 해시**를 기준으로 검사할 것입니다.
3. API 호출 후 검증을 시도합니다. (⚠️ `--root-hash` 옵션에 원본 해시 입력)

```bash
# 1. API 호출 (결과 저장)
curl -X POST http://127.0.0.1:7325/wash ... > tampered.json

# 2. 원본 해시를 기준으로 검증 시도
openseal verify --response tampered.json --wax "my-secret-session-123" --root-hash <원본_A_HASH_값>
```

**결과 확인**:
> ❌ **Identity Valid**: ❌  
> "Identity Mismatch. The code executed is different from what was expected."

검증기는 현재 실행된 코드가 원본과 다름을 정확히 감지하고 **INVALID**를 선언합니다. 이것이 OpenSeal의 핵심 가치입니다.

---

## 🛡️ 제외 규칙 활용하기

파이썬 프로젝트의 `venv/`나 `__pycache__/`는 무결성 검사에서 제외되어야 합니다. OpenSeal은 자동으로 `.gitignore`를 존중하며, 추가로 `.opensealignore`를 통해 제외 규칙을 설정할 수 있습니다.

- **전체 제외**: `.opensealignore`에 `venv/` 추가 (파일 존재 자체를 무시)
- **내용만 제외(가변 파일)**: `.openseal_mutable`에 `*.log` 추가 (파일 존재는 확인하되 내용은 무시)

---

## 🛡️ 핵심 정책: Golden Truth
OpenSeal은 실제 프로덕션에서도 동일한 원칙을 적용합니다.

- **은닉**: 서버 로직, 해시 구조 등을 외부에 노출하지 않습니다.
- **증명**: 오직 특정 상황(`Wax` + `Input`)에 대한 **정답 서명(Golden Truth)**만 검증자에게 공유합니다.
- **검증**: 검증자는 응답으로 온 서명이 공유받은 골든 트루스와 일치하는지만 대조하면 됩니다.

---

## 💡 다음 단계
- [프로젝트 아키텍처 개요 (ARCHITECTURE_KR.md)](./ARCHITECTURE_KR.md)
- [공개 기술 명세 (SPEC_PUBLIC_KR.md)](./SPEC_PUBLIC_KR.md)
