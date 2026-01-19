[🇺🇸 English Version](./README.md)

# 🔐 OpenSeal: 10초 만에 '검증 가능한 API' 만들기

내 서비스 안에서 내 API 서비스를 단 한 줄도 수정하지 않고, 코드가 변조되지 않았음을 고객에게 수학적으로 증명하세요.

### 1. 설치
```bash
cargo install --git https://github.com/kjyyoung/openseal.git --bin openseal
```

### 2. 봉인 (Build)
> [!IMPORTANT]
> 모든 OpenSeal 명령어는 반드시 **프로젝트 루트** 디렉토리에서 실행해야 합니다.

```bash
# 기존에 사용하던 서버 실행 명령어를 등록합니다.
openseal build --exec "node app.js"
```

### 3. 실행 (Run)
```bash
# 원래 사용하던 포트(예: 3000)를 그대로 입력하세요.
# OpenSeal이 내부 포트 충돌을 자동으로 해결하고 서비스를 보호합니다.
openseal run --app ./dist --port 3000
```

**✅ 끝!** 당신의 API 서비스는 이제 모든 실행 결과에 대해 위조 불가능한 암호학적 인감(Seal)을 찍어 보냅니다.

---

### 🔐 보호된 런타임 (Protected Runtime)

OpenSeal의 인감 생성 엔진은 '보호된 런타임' 형태로 배포됩니다. 이는 보안을 위한 의도적인 설계입니다.
- 모든 **검증 로직(Verification)은 OSIP-7325를 통해 투명하게 공개**됩니다.
- 제3자는 누구나 독립적으로 인감의 유효성을 검증할 수 있습니다.
- 다만, **인감 생성 과정은 의도적으로 보호된 경계 내에 제한**되어, 위조나 리플레이 공격, 혹은 적대적 환경에서의 메모리 패킹 공격을 원천 차단합니다.

이는 Secure Enclave (TEE), HSM 기반 서명 서비스, 에지 실행 런타임 등에서 사용하는 업계 표준 보안 설계를 따르는 것입니다.

---

### 🛡️ 위협 모델 및 보장 (Threat Model)

| 보안 목표 | OpenSeal의 보장 |
| :--- | :--- |
| **결과 무결성** | 결과값이 변조되지 않고 봉인된 코드에서 도출되었음을 수학적으로 증명합니다. |
| **정체성 바인딩** | 실행된 코드가 승인된 상태(A-hash)와 일치함을 보장합니다. |
| **재사용 방지** | Wax(난수)를 통해 과거의 인감을 현재 요청에 재사용하는 것을 방지합니다. |
| **개인정보 보호** | 데이터 수집을 하지 않으며, 생성 코어는 외부와 통신하지 않습니다. |

---

## 📖 더 알아보기
* [프로토콜 규격 (PROTOCOL)](./docs/public/PROTOCOL_KR.md)
* [보안 정책 및 전략 (POLICY)](./docs/public/POLICY_KR.md)
* [사용 가이드 (USAGE)](./docs/public/USAGE_KR.md)
