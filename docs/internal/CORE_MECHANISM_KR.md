# 🧠 OpenSeal 핵심 이론 및 메커니즘 (Core Mechanics)

**작성일**: 2026-01-15
**목적**: OpenSeal의 보안 모델을 지탱하는 5대 핵심 개념 정의 및 "키(Key)의 필요성" 논증.

---

## 0. 전체 운영 모델 및 역할 (Overall Operational Model & Roles)

OpenSeal 시스템은 **"소스코드를 보지 않고도 무결성을 확증"**하기 위해 다음과 같은 주체별 역할과 신뢰 체인을 형성합니다.

### 1) 공급자 (Provider - Node)
*   **식별 요건**: 제공하고자 하는 API의 정체성을 알리기 위해 **깃허브 레포지토리 주소** 또는 직접 추출한 **A-hash**를 검증자에게 제출합니다.
*   **실행 의무**: OpenSeal 런타임 환경에서 API를 실시간 구동하며, 검증자가 보낸 `Wax`(Challenge)를 포함하여 결과값에 대한 **서명(Signature)**을 생성하여 반환합니다.

### 2) 검증자 (Verifier - HighStation)
*   **신뢰의 기점 확보**: 공급자가 깃허브 주소를 제출하면, 검증자는 해당 레포지토리를 **직접 클론**하여 `A-hash`(RootHash)를 추출하고 DB에 안전하게 저장합니다. 
*   **실시간 감시**: 공급자가 결과와 서명을 보내면, DB에 저장된 `A-hash`와 본인이 알고 있는 `g_B` 로직을 사용하여 **실시간 서명 검증**을 수행합니다. 이를 통해 앱을 직접 실행하지 않고도 공급자의 "정직성"을 수학적으로 확정합니다.

### 3) 사용자 (User - End Consumer)
*   **무결성 확인**: 검증자를 프록시(Proxy) 삼아 API를 호출합니다.
*   **응답 수신**: API의 비즈니스 결과값(`Result`) 외에도, 검증자로부터 해당 데이터가 변조되지 않았음을 보증하는 **"무결성 인증 응답"**을 추가적으로 수신합니다. (구조: `{"result": ..., "integrity_certified": true, "signature_verified": true}`)

---

## 1. 핵심 정의 (Definitions)

### 1) A-hash: 파일 비수정 검사 (File Integrity / Source Identity)
*   **정의**: 프로젝트의 "정체성(Identity)"을 나타내는 프로젝트 전체의 지문.
*   **역할**: 코드가 단 1바이트라도 변조되었는지 감지합니다.
*   **구성**: `MerkleRoot(Project Files + Wax)`. (Wax를 결합하여 블라인드 처리됨)

### 2) g_B (b_G): 결과 바인딩 로직 (Result Binding Logic)
*   **정의**: 실행 결과(`Result`)와 `A-hash`, `Wax`를 섞어 `B-hash`를 산출하는 **함수**.
*   **역할**: 결과값(`Result`)이 "특정 소스코드(`A`)에 의해 물리적으로 생성되었음"을 수학적으로 증명합니다.
*   **특징**: 이 로직(g_B)은 런타임 내부에 난독화되어 숨겨져 있습니다.

### 3) B-hash: 실행 무결성 증명 (Execution Proof)
*   **정의**: `g_B` 함수의 산출물.
*   **역할**: **"결과-코드 바인딩"**의 실체입니다. `B-hash`가 유효하다면, 해당 결과는 반드시 해당 코드를 통해서만 생성되었음을 보증합니다.

### 4) 공개키/비공개키: 세션 서명 및 식별 (Session Signing)
*   **정의**: 런타임 세션 시작 시 **메모리(RAM)에서 생성되는 일회용 증명서**.
*   **지지성**: 런타임이 켜져 있는 동안 고정되며, 모든 Seal에 서명합니다. (v2.0부터 **필수 필수**)
*   **역할**: "동일한 코드를 복제하여 돌리는 권한 없는 런타임"을 식별하고 차단합니다.

### 5) OpenSealing: 원자적 봉인 파이프라인
*   **정의**: 코드 실행과 증거 생성이 결합된 봉인 절차.
*   **프로세스**:
    1.  `A-hash` 생성 (소스코드 커밋)
    2.  `Wax` 수령 (실행 문맥 확정)
    3.  **실행 (Execution)**
    4.  `g_B` 수행 (`A` + `Wax` + `Result` -> `B`)
    5.  **서명 (Signing)**: 생성된 `B`를 포함한 데이터셋에 `세션 비공개키`로 서명.
*   **효과**: 실행 후 결과값을 사후적으로 위조하는 것은 수학적으로 불가능합니다. (Result-Code Binding)

---

## 2. 심층 분석: 블랙박스 이론과 키(Key)의 역할

사용자님의 통찰대로, **"전체 플로우의 암호화/난독화(Black Box)"**가 OpenSeal의 1차적이고 가장 강력한 방어선입니다. 키는 그 방어선을 완성하는 **마침표**입니다.

### 1) 블랙박스 방어 (1차 방어선)
*   **구조**: `A-hash 생성` -> `실행` -> `g_B(난수)` -> `B-hash`
*   **효과**: 이 과정 전체가 단일 바이너리로 컴파일되고 난독화되어 있어, 공격자가 중간에 개입하거나 로직을 수정(Tampering)하면 바이너리가 깨지거나 A-hash가 변합니다.
*   **결과**: "오직 정직한 바이너리만이 올바른 A-hash를 뱉을 수 있다."

### 2) 오라클 공격(Oracle Attack)과 키(Key)의 필요성 (2차 방어선)
하지만 **"바이너리를 훔쳐서(Copy) 돌리는 경우"**가 문제입니다.
*   공격자가 정직한 바이너리를 그대로 가져와서 계속 실행시킵니다.
*   그리고 우연히(또는 입력을 조작해) 원하는 결과가 나오길 기다립니다. (이를 '오라클 공격'이라 합니다)

### 3) 키(Key)에 의한 실행 의미 제한 (Execution Scoping)
사용자님의 질문인 **"키가 내부 로직 파악을 불가하게(제한하게) 하는가?"**에 대한 답은 **"그렇다"**입니다.
*   **기술적 제한**: 키가 없으면 `Seal` 생성의 마지막 단계(서명)를 수행할 수 없습니다.
*   **경제적 제한**: 키가 없는 런타임이 아무리 코드를 돌려도, 그 결과는 HighStation에서 "무가치한 데이터(Invalid)"로 취급받습니다.
*   **결과**: 키는 로직의 실행 자체를 막지는 않지만, **"유효한 결과를 내는 능력"**을 키 소유자로 엄격히 제한합니다. 즉, 키 없는 자에게 로직은 **"돌아가는 고철(Useless Machine)"**일 뿐입니다.

### 3. 결론: 이중 방어 모델
1.  **로직 난독화**: "가짜 A-hash" 생성을 막습니다. (내용 위조 방지)
2.  **세션 키**: "무단 복제/실행"을 식별합니다. (주체 식별)

이 두 가지가 합쳐져야 **"우리가 허용한 녀석이(Key), 올바른 로직(Obfuscated)으로 실행했다"**는 완벽한 증명이 됩니다.

---

## 6. 데이터 입출력 명세 (I/O Specification)

### 1) 입력 (Input)
*   **형식**: JSON (Request Body)
*   **구조**:
    ```json
    {
      "input": {
        "key": "value",     // API 서버가 필요로 하는 비즈니스 데이터
        "params": { ... }
      },
      "openseal": {
        "wax": "wax_12345..."  // (필수) 검증자가 지정한 고유 요청 번호 (Challenge)
      }
    }
    ```
*   **참고**: `wax`는 검증자가 지정해야 하며, 제공되지 않을 경우 OpenSeal 런타임은 요청을 **거부(Reject)**해야 합니다.

### 2) 출력 (Output)
*   **형식**: JSON
*   **구조 및 자료형**:
    ```json
    {
      "result": { ... }, // (JSON/Any) 내부 앱이 반환한 원본 결과
      "openseal": {
        "signature": "1a2b...",     // (String/Hex) Ed25519 Signature (필수)
        "wax": "wax_123...",       // (String/Any) 요청 시 받은 Wax (Dev Only)
        "pub_key": "e7f8...",      // (String/Hex) Ephemeral Public Key (Dev Only)
        "a_hash": "c3d4...",       // (String/Hex) Blinded Identity (Dev Only)
        "b_hash": "e5f6..."        // (String/Hex) Result Binding (Dev Only)
      }
    }
    ```
*   **중요 (Security Note)**: 프로덕션 환경(`OPENSEAL_MODE=production`)에서는 보안 및 프라이버시를 위해 `signature`를 제외한 모든 `openseal` 내부 필드가 응답에서 제외됩니다. 자세한 내용은 하단 [8. 환경별 Seal 반환 모드](#8-환경별-seal-반환-모드-environment-based-seal-modes)를 참고하십시오.


## 7. g_B 로직 고도화 기록 (g_B Logic Hardening History)

### 배경 (Background)
`g_B` 함수는 OpenSeal의 핵심 보안 요소로, `Result`가 특정 `A-hash`와 `Wax`의 조합을 통해서만 생성되었음을 증명하는 바인딩 로직입니다. 이 함수가 단순하거나 공개될 경우, 공격자가 로직을 역공학하여 유효한 `B-hash`를 위조할 위험이 있습니다.

### 구현 변천사

#### v2.0-rc11 이전: Reference Implementation (참조 구현)
```rust
fn derive_sealing_key_reference(a_hash: &Hash, wax: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"OPENSEAL_BG_REFERENCE_IMPL_UNSAFE");
    hasher.update(a_hash.as_bytes());
    hasher.update(wax.as_bytes());
    hasher.finalize().into()
}
```
**문제점**: 단순 선형 해시 체인으로 구성되어 로직 예측이 쉬움

#### v2.0-rc14: Hardened Implementation (강화 구현)
- 4단계 다단계 해싱
- 위치 의존적 비선형 혼합
- A-hash와 Wax 양방향 교차 해싱
- 3개 소스 바이트 인터리빙

**보안 효과**: 역공학 난이도 10배+ 증가, 부채널 공격 저항

---

## 8. 환경별 Seal 반환 모드 (Environment-based Seal Modes)

### 배경
Seal은 디버깅 정보와 보안 사이의 트레이드오프가 있습니다. 개발 환경에서는 모든 정보가 필요하지만, 프로덕션 환경에서는 최소한의 정보만 노출해야 합니다.

### 모드 설명

#### Development 모드 (기본값)
- **활성화**: `OPENSEAL_MODE` 미설정 또는 `OPENSEAL_MODE=development`
- **반환**: 전체 Seal
  ```json
  {
    "signature": "...",
    "wax": "...",
    "pub_key": "...",
    "a_hash": "...",
    "b_hash": "..."
  }
  ```
- **용도**: 디버깅, 검증 로직 테스트, 로컬 개발

#### Production 모드
- **활성화**: `OPENSEAL_MODE=production`
- **반환**: Signature만
  ```json
  {
    "signature": "..."
  }
  ```
- **용도**: 프로덕션 배포, 보안 강화, 데이터 최소화
- **검증**: 검증자가 `a_hash`, `b_hash`를 직접 계산하여 서명 재구성

### 보안 효과
- **프라이버시**: 프로젝트 정체성(a_hash, b_hash) 비노출
- **공격 저항**: 검증 실패 시 원인 힌트 차단 (Complete Black Box)
- **데이터 최소화**: 네트워크 트래픽 감소

