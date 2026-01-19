# OSIP-7325: 무결성 보증형 API 통신 프로토콜 (OpenSeal Integrity Protocol)

**버전**: 1.0.0  
**상태**: Draft  
**포트**: 7325 (Default)

---

## 1. 개요 (Introduction)

**7325 프로토콜**은 원격지에서 실행되는 코드의 무결성을 수학적으로 보증하기 위한 통신 표준입니다. 이 프로토콜은 검증자가 공급자의 소스코드를 직접 실행하지 않고도, 공급자가 제출한 실행 결과가 "합의된 코드(GitHub Repository)"로부터 생성되었음을 보증하는 것을 목적으로 합니다.

---

## 2. 주체별 역할 (Roles and Responsibilities)

### 2.1 공급자 (Provider - "The Executor")
- **정체성 증명**: 서비스 등록 시 원본 소스코드의 접근 권한(GitHub URL)을 제공합니다.
- **격리 실행**: **OpenSeal 고립 환경(Sanitized Environment)** 내에서 비즈니스 로직을 수행하여 외부 간섭을 차단합니다.
- **봉인(Sealing)**: 실행 결과값과 검증자가 제공한 `Wax`(Challenge)를 포함하여 **서명(Signature)**을 생성합니다. **공급자는 내부 바인딩 구조의 세부 동작을 알 수 없으며, 이를 통해 임의적인 결과 수정이나 서명 위조가 원천 차단됩니다.**

### 2.2 검증자 (Verifier - "The Oracle")
- **신뢰의 기점(Root of Trust)**: 등록된 소스코드를 직접 클론하여 고유 지문(`RootHash`)을 추출합니다.
- **비대칭 검증**: 공급자로부터 받은 `Result`와 `Signature`를 자신이 알고 보관 중인 `RootHash` (A-Hash)를 기준으로 실시간 검증합니다. 이때 비즈니스 로직은 실행하지 않습니다.

### 2.3 사용자 (User - "The Consumer")
- **결과 수신**: 검증자를 경유하여 API를 호출합니다.
- **신뢰 획득**: 비즈니스 결과값 외에도, 검증자가 보증하는 무결성 인증 필드를 함께 수신합니다.

---

## 3. 프로토콜 동작 플로우 (Protocol Flow)

### Phase 1: 등록 및 지문 추출 (Onboarding)
1. 공급자가 검증자에게 **GitHub Repository URL**을 제출합니다.
2. 검증자는 해당 레포지토리를 격리된 환경에서 클론합니다.
3. 검증자는 프로젝트 전체 파일에 대해 머클 루트(Merkle Root)를 계산하여 **`RootHash`**를 생성하고 DB에 저장합니다.

### Phase 2: 요청 및 챌린지 (Request & Challenge)
1. 사용자가 검증자에게 API 요청을 보냅니다.
2. 검증자는 예측 불가능한 난수 **`Wax`**를 생성합니다.
3. 검증자는 공급자에게 `Request + Wax`를 전달합니다.

### Phase 3: 실행 및 서명 (Execution & Signing)
1. 공급자 노드는 OpenSeal 런타임에서 API를 실행합니다.
2. 실행 완료 후, `Result`, `Wax`, `A-hash` 등 상태 식별자들을 포함하여 **`Signature`**를 생성합니다.
3. 공급자는 `Result + Signature`를 검증자에게 반환합니다.

### Phase 4: 검증 및 응답 (Verification & Final Response)
1. 검증자는 받은 `Result`와 자신이 저장한 `RootHash`를 기반으로 직접 `Expected Signature`를 계산합니다.
2. 실제 받은 서명과 계산된 서명이 일치하면, **무결성 보증 마크**를 응답에 포함하여 사용자에게 최종 반환합니다.

---

## 4. 데이터 규격 (Data Specification)

### 4.1 요청 규격 (Request)
**Header 방식 (권장)**: 기존 API 바디를 건드리지 않는 비침투적(Non-intrusive) 방식입니다.
```http
POST /api/endpoint HTTP/1.1
Host: provider.com
Content-Type: application/json
X-OpenSeal-Wax: <Verifier_Generated_Random_Hex>

{
  "key": "value" // (옵션) 원본 API가 요구하는 비즈니스 데이터
}
```

### 4.2 응답 규격 (Response - Provider to Verifier)
OpenSeal 런타임은 원본 응답(`result`)에 무결성 보증(`openseal`)을 래핑하여 반환합니다.
```json
{
  "result": { ... }, // 원본 애플리케이션의 JSON 응답
  "openseal": {
    "signature": "Hex_Signature_String",  // 필수: 무결성 서명
    "pub_key": "Hex_PublicKey",           // 필수: 서명 검증용 일회성 키
    "a_hash": "Hex_Blinded_Identity",     // 필수: 실행 정체성 식별자
    "b_hash": "Hex_Result_Binding"        // 필수: 결과 바인딩 식별자
  }
}
```

### 4.3 응답 규격 (Response - Verifier to User)
(참고) 검증자가 검증을 마친 후 최종 사용자에게 전달하는 포맷 예시입니다.
```json
{
  "result": { ... },
  "security": {
    "certified": true,
    "timestamp": "ISO8601"
  }
}
```

---

## 5. 보안 핵심 원칙 (Security Core Principles)

1. **로직 기밀성(Logic Secrecy)**: 검증 시 소스코드를 노출하지 않을 뿐만 아니라, 바인딩 로직(`g_B`) 자체가 공급자에게 **블랙박스(Black-box)**로 제공됩니다. 이는 공급자가 결과값을 조작하고 그에 맞는 가짜 서명을 생성하는 것을 수학적으로 방지하는 핵심 장치입니다.
2. **고립된 실행 환경(Isolated Runtime)**: 모든 API 실행은 환경 변수 소독(`env_clear`), 네트워크 Monopoly 포트 제어 등이 적용된 **OpenSeal 격리 컨텍스트** 내에서 이루어집니다. 이는 실행 중인 코드에 대한 외부의 동적 간섭을 차단합니다.
3. **독립적 지문 확보**: 검증자는 입력을 믿지 않고 소스코드로부터 직접 해시를 추출하여 신뢰의 기점을 확보합니다.
4. **실시간 바인딩**: 모든 응답은 일회성 `Wax`에 바인딩되어 재사용(Replay Attack)을 차단합니다.
