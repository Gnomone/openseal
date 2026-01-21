# OpenSeal 보안 모델 및 위협 방어 (Security Model & Defense)

본 문서는 OpenSeal 프로토콜이 상정하는 위협 모델(Threat Model)과, 이에 대응하기 위한 다층적 방어 메커니즘을 기술합니다.

---

## 1. 핵심 보호 목표 (Core Security Objectives)

OpenSeal은 다음의 질문에 대해 수학적/구조적 확신을 제공하는 것을 목표로 합니다.

> **"지금 내가 받은 이 결과값(Result)이, 내가 합의한 소스코드(Identity)로부터, 내가 요청한 시점(Wax)에 생성된 것이 맞는가?"**

이를 위해 다음 3가지 속성을 보장합니다.
1.  **무결성 (Integrity)**: 결과값이 생성 후 변조되지 않았음.
2.  **정체성 (Authenticity)**: 실행된 코드가 검증자가 알고 있는 원본과 일치함.
3.  **최신성 (Freshness)**: 결과가 재사용(Replay)된 것이 아님.

---

## 2. 위협 모델 (Threat Model)

OpenSeal은 다음과 같은 공격 시나리오를 방어하도록 설계되었습니다.

### 2.1 결과 변조 및 위조 (Result Tampering & Forgery)
*   **공격 시나리오**: 공격자(악의적 공급자)가 코드를 실행하지 않거나, 실행 결과를 임의로 수정하여 반환합니다.
*   **방어 기제**:
    *   **서명(Signature)**: 결과값은 일회성 키로 서명되며, 서명 생성에는 `Result` 자체가 포함됩니다.
    *   **바인딩 식별자(B-Hash)**: 결과값은 내부적으로 `A-Hash` 및 `Wax`와 결합되어 고유한 식별자를 생성합니다. 공격자는 내부 구조(`g_B`)를 모르므로 유효한 B-Hash를 생성할 수 없습니다.

### 2.2 재전송 공격 (Replay Attack)
*   **공격 시나리오**: 공격자가 과거의 유효한(정상적인) 응답을 가로채어, 현재 요청에 대한 응답인 것처럼 제출합니다.
*   **방어 기제**:
    *   **Wax (Challenge)**: 모든 요청에는 검증자가 생성한 일회성 난수(`Wax`)가 포함됩니다.
    *   **강제 결합**: 서명 생성 시 `Wax`가 필수 요소로 포함되므로, `Wax`가 다른 과거의 응답은 현재 검증을 통과할 수 없습니다.

### 2.3 섀도우 애플리케이션 공격 (Shadow Application Attack)
*   **공격 시나리오**: 공격자가 `Main Code` 대신, 검증자를 속이기 위해 조작된 `Fake Code`를 실행합니다.
*   **방어 기제**:
    *   **A-Hash 검증**: OpenSeal 런타임은 실행 직전 프로젝트 파일 전체를 머클 트리(Merkle Tree)로 해싱하여 `Root Hash`를 생성합니다.
    *   **Identity Mismatch**: 검증자는 자신이 알고 있는 원본의 `Root Hash`와 런타임이 보고한 `A-Hash`를 대조하여, 코드가 단 한 바이트라도 다르면 즉시 거부합니다.

### 2.4 런타임 환경 오염 (Environment Poisoning)
*   **공격 시나리오**: `LD_PRELOAD`, `PYTHONPATH` 등을 조작하여 실행 환경을 납치하거나 라이브러리를 바꿔치기합니다.
*   **방어 기제**:
    *   **환경 소독 (Environment Sanitization)**: OpenSeal 런타임은 자식 프로세스 실행 전 `env_clear()`를 통해 모든 환경 변수를 초기화하고, 화이트리스트 변수만 주입합니다.

---

## 3. 구조적 방어 (Architectural Defense)

### 3.1 Caller Monopoly (호출자 독점)
*   OpenSeal 런타임은 "데몬"이 아니라 **"일회용 프로세스"**로 설계되었습니다.
*   한 번의 요청-응답 사이클이 끝나면 런타임과 내부 앱은 종료됩니다.
*   이는 메모리 상주형 공격(Memory Resident Attacks)이나 상태 오염의 지속을 방지합니다.

### 3.2 Ephemeral Keys (일회성 키)
*   OpenSeal은 장기 보관되는 "마스터 키"를 사용하지 않습니다.
*   매 실행 시 메모리 상에서 난수 기반의 `Ephemeral Keypair`를 생성하고, 서명 후 즉시 파기합니다.
*   따라서 키 관리 소홀로 인한 탈취 위험이 원천적으로 제거됩니다.

---

## 4. 검증자의 책임 (Verifier's Responsibility)

따라서 검증자는 반드시 **신뢰할 수 있는 채널(GitHub 등)**을 통해 원본 소스코드를 확보하고, 직접 해시를 계산해야 합니다. 공급자가 주는 해시를 맹신해서는 안 됩니다.

---

## 5. 확장 모델: Trustless AI Agent (Client-Side Verification)

OpenSeal의 검증 로직은 특정 플랫폼(HighStation 등)에 종속되지 않습니다. `openseal-verifier` 모듈(TypeScript/WASM)을 탑재한 AI 에이전트는 **중개자 없이 스스로** 서비스를 검증할 수 있습니다.

1.  **Discovery**: 블록체인이나 레지스트리에서 서비스의 `Endpoint`와 `Golden Truth (RootHash)`만 조회합니다.
2.  **P2P Verification**: 에이전트가 직접 난수(`Wax`)를 생성하여 요청하고, 응답 서명을 로컬에서 즉시 검증합니다.
3.  **Autonomous Trust**: 에이전트는 플랫폼을 신뢰할 필요 없이, 오직 **수학(암호학)과 코드(RootHash)**만을 기준으로 서비스 사용 여부를 자율적으로 결정합니다.

