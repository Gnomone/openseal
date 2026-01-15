# 🔐 OpenSeal: Atomic Project Sealing (v2.0)

> **"OpenSeal does not modify application code. It replaces the caller."**

OpenSeal은 API 서버 전체의 무결성을 보장하기 위한 **호출 경계 보호(Call Boundary Protection)** 표준입니다. 

---

## 1. 핵심 철학 (The Philosophy)

### ① 사건(Event) vs 관(Case)
*   **사건 (Event)**: 비즈니스 로직의 실제 실행. OpenSeal은 실행 환경을 독점하여 이를 하나의 '사건'으로 캡슐화합니다.
*   **관 (Case)**: 데이터를 운반하는 껍데기(Django, Express 등). 사건의 정체성에는 관여하나, 내부 데이터에는 접근할 수 없습니다.

### ② 반환값의 재정의: "단일 실행 주장 (Atomic Event Assertion)"
*   OpenSeal 환경에서 실행되는 코드의 `return` 값은 외부로 나가는 데이터가 아닙니다.
*   이는 **캡슐 내부의 상태를 증명하는 신호**이며, 런타임에 의해 즉시 흡수되어 봉인(`B-hash`)으로 변환된 후 비로소 세상 밖으로 나갑니다.

---

## 2. 보안 아키텍처: 호출자 독점 (Caller Monopoly)

### ① 실행 컨텍스트 장악
*   OpenSeal은 소스코드를 수정하는 대신, 코드가 실행되는 **런타임 컨텍스트**를 완전히 장악합니다.
*   **Execution Isolation**: 부모 프로세스(OpenSeal)가 자식 프로세스(App)의 입출력과 메모리 경계를 엄격히 통제합니다.

### ② 동적 검증 함수 (Dynamic Verification)
*   봉인 로직은 요청 시점마다 동적으로 변화하는 **비결정적 구조**를 가집니다.
*   공격자가 실행 중인 상태를 관측하더라도, 다음 요청에서는 내부 검증 로직이 달라지므로 재사용 및 사후 위조가 불가능합니다.

---

## 3. 구현 전략 (The Strategy)

### 🔑 소스코드 무수정 (Zero-Edit)
*   개발자는 평소대로 코딩합니다.
*   OpenSeal은 API의 **호출 경계(Call Boundary)**를 감싸서, "실행하지 않으면 결과에 대응하는 봉인을 제작할 수 없는 구조"를 강제합니다.

### 🔑 경제적 무결성 (Economic Integrity)
*   본 모델은 루트(ROOT) 권한을 가진 공격자가 실시간으로 메모리를 계측하여 조작하는 것을 '불가능'하다고 주장하지 않습니다.
*   대신, **"위조 행위의 비용을 정직한 실행 비용 이상으로 키우는 것"**을 목표로 하여 실무적인 무결성을 완성합니다.

---

> **OpenSeal: The return value is never trusted as data — it is consumed as a state assertion inside a sealed runtime.**

---

## 4. 동작 흐름 (Intuitive Flow)

사용자(개발자) 관점에서 OpenSeal은 다음과 같이 단순하게 동작합니다.

```mermaid
graph TD
    A[📂 Source Code Repo] -->|openseal build| B[📦 Sealed Bundle]
    B -->|Identity Check| C{OpenSeal Runtime}
    
    subgraph Caller Monopoly [호출자 독점 영역]
        C -->|Spawn| D[🔒 Child Process (API Server)]
        E[User Request] -->|Context Injection| C
        C --Proxy--> D
        D --Result--> C
        C -->|P(Event)| F[Proof Binding]
    end

    F -->|Response + Seal| G[Client]
```

1.  **빌드 (`openseal build`)**:
    *   레포지토리의 소스코드를 스캔하여 `A-hash`(운영전 식별자)를 확정하고 패키징합니다.
    *   API 서버 자체는 수정되지 않습니다.

2.  **실행 (`openseal run`)**:
    *   OpenSeal은 부모 프로세스로서 API 서버를 자식 프로세스로 실행하고 격리합니다.
    *   외부 접근을 차단하고, 오직 OpenSeal을 통해서만 통신할 수 있습니다.

3.  **봉인 (Sealing)**:
    *   **입력**: API 서버는 실행 시 고유 식별자(`Nonce`)를 주입받습니다.
    *   **실행**: 코드는 레포지토리 원본 그대로 실행됨을 보장받습니다.
    *   **출력**: 실행 결과는 런타임에 의해 **"실행하지 않고는 결과를 위조할 수 없음"**을 증명하는 봉인으로 변환됩니다.
