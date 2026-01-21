# 🛠️ OpenSeal: 사용법 가이드

OpenSeal 서비스의 설정, 실행 및 안전한 관리 방법을 다룹니다.

---

## 1. 5분 퀵스타트

### 1단계: CLI 설치
```bash
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash
```

### 2단계: 봉인 (Build)
```bash
# 프로젝트 루트에서 실행
openseal build --exec "npm run dev" --output dist_opensealed
```

### 3단계: 실행
```bash
# 포그라운드
openseal run --app dist_opensealed --port 3000

# 또는 백그라운드 (프로덕션)
openseal run --app dist_opensealed --port 3000 --daemon
```

> [!TIP]
> `--daemon` 플래그를 사용하면 SSH 연결이 끊겨도 서비스가 계속 실행됩니다.

---

## 2. 언어별 퀵스타트

OpenSeal은 소스 코드를 직접 실행(JIT)하는 것을 권장합니다.

### 🟢 Node.js / TypeScript
```bash
openseal build --exec "npm run dev" --output dist_opensealed
openseal run --app dist_opensealed --port 3000
```
> 💡 **JIT 권장**: `tsx` 또는 `ts-node`로 소스 직접 실행

### 🐍 Python
```bash
openseal build --exec "python main.py" --output dist_opensealed
openseal run --app dist_opensealed --port 8000
```
> 💡 **가상환경**: `venv`, `.venv` 자동 감지

### 🔵 Go
```bash
go build -o app
openseal build --exec "./app" --output dist_opensealed
openseal run --app dist_opensealed --port 8080
```

### 🦀 Rust
```bash
cargo build --release
openseal build --exec "./target/release/myapp" --output dist_opensealed
openseal run --app dist_opensealed --port 8000
```

---

## 3. 주요 옵션

| 옵션 | 설명 | 예시 |
|------|------|------|
| `--exec` | 봉인된 환경에서 실행할 명령어 | `npm run dev`, `python app.py` |
| `--output` | 봉인된 파일이 저장될 폴더 | `dist_opensealed` |
| `--daemon` | 백그라운드 실행 (프로덕션) | - |

---

## 4. 표준 Identity 엔드포인트

모든 OpenSeal 서비스는 자동으로 `/.openseal/identity` 엔드포인트를 노출합니다.

```bash
curl http://localhost:3000/.openseal/identity
```

**응답**:
```json
{
  "service": "OpenSeal Runtime Identity",
  "version": "0.2.6",
  "identity": {
    "a_hash": "14f38520...",
    "file_count": 1630
  },
  "status": "sealed"
}
```

이를 통해 **HighStation** 등 외부 도구가 앱 코드 수정 없이 실시간 무결성을 검증할 수 있습니다.

---

## 5. Runtime 무결성 검증 (v0.2.6+)

OpenSeal Runtime은 시작 시 봉인된 번들의 무결성을 자동으로 검증합니다.

**동작 방식**:
1. `dist_opensealed/` 스캔하여 Live Hash 계산
2. `openseal.json`의 Expected Hash와 비교
3. **변조 감지 시 → Runtime 중단**

**정상 케이스**:
```bash
$ openseal run --app dist_opensealed --port 3000
   ✅ Live A-hash: 14f38520...
   ✅ Integrity Verified!
   🚀 OpenSeal Running
```

**변조 케이스**:
```bash
$ openseal run --app dist_opensealed --port 3000
   🚨 INTEGRITY VIOLATION DETECTED
   Expected: 14f38520...
   Actual:   XXXXXXXX...
   Error: Runtime aborted
```

---

## 6. openseal verify (검증 도구)

API 응답의 무결성을 검증합니다.

```bash
openseal verify --response result.json --wax "난수값" --root-hash "14f38520..."
```

**result.json 형식**:
```json
{
  "result": { "symbol": "BTC", "price": "98500" },
  "openseal": {
    "signature": "...",
    "pub_key": "...",
    "a_hash": "...",
    "b_hash": "..."
  }
}
```

**검증 내용**:
- ✅ **서명 검증**: Ed25519 서명 유효성
- ✅ **Binding 검증**: B-hash 일치 여부
- ✅ **Identity 검증**: A-hash 일치 여부 (--root-hash 제공 시)

---

## 7. 안전 가드레일

OpenSeal은 의도치 않은 위치 봉인을 방지합니다.

**프로젝트 자동 탐지**:
- `package.json`, `Cargo.toml`, `.git` 등 확인
- 파일이 없으면 경고 후 확인 요청

**권장 사항**:
- ✅ 프로젝트 루트에서 실행
- ✅ `.opensealignore`로 불필요한 파일 제외

---

## 8. 제외 규칙

**`.opensealignore`**:
- A-hash 계산에서 완전히 제외
- 예: `node_modules/`, `venv/`, `.git/`

**`.openseal_mutable`**:
- 파일 존재는 봉인, 내용 변경 허용
- 예: `*.db`, `logs/`, `cache/`

---

## 📚 추가 문서

- [프로토콜 사양 (PROTOCOL)](./PROTOCOL_KR.md)
- [언어 독립성 (AGNOSTICISM)](./AGNOSTICISM_KR.md)
- [보안 정책 (POLICY)](./POLICY_KR.md)
