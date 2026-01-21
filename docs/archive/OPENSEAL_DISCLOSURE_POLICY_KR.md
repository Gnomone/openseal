# OpenSeal 공개 / 비공개 범위 지시문

[🇺🇸 English Version](./OPENSEAL_DISCLOSURE_POLICY.md)

---

## 0. 목적 (Purpose)

본 문서는 OpenSeal 프로젝트에서
**공개되어야 하는 영역(Open)**과
**의도적으로 비공개로 유지되는 영역(Sealed)**을 명확히 정의함으로써,

* 검증 가능성
* 오픈소스 신뢰성
* 구현 탈취 방지
* 플랫폼 독립성 유지

를 동시에 달성하는 것을 목적으로 한다.

---

## 1. OpenSeal의 역할 정의 (Non-Negotiable)

OpenSeal은 다음을 **하지 않는다**:

* API 실행 ❌
* 요청 오케스트레이션 ❌
* 네트워크 호출 ❌
* 런타임 제공 ❌

OpenSeal은 오직 다음을 수행한다:

> **“어떤 실행 결과가, 사전에 고정된 코드 정체성에 의해 생성되었는지를 검증하는 프로토콜 및 검증기”**

---

## 2. 공개 범위 (Public / Open)

다음 항목은 **완전 공개(Open Source)** 대상이며, 누구나 열람·검증·포크할 수 있다.

### 2.1 개념 및 이론 (Conceptual Layer)

* 실행 정체성 개념 (Execution Identity)
* **Execution Identity Model (A-Hash, B-Hash)**
* **“결과는 단순한 반환값이 아니라, 검증 가능한 실행 주장(Execution Claim)이다”**라는 정의
* 위조 가능성에 대한 위협 모델 (Threat Model)

📌 *검증에 필요한 이론은 숨기지 않는다.*

---

### 2.2 명세 (Specification)

* A-Hash 정의 방식 (Input Spec)
* B-Hash 정의 방식 (Output Structure - Not Derivation)
* 검증 입력/출력 포맷
* 실패 조건 (invalid cases)

```text
Input: (Result, A, B)
Output: VALID | INVALID
```

📌 *검증 검사(Check) 로직은 공개하되, 생성(Generation) 로직은 분리한다.*

---

### 2.3 검증기 (Verifier)

* verify() 알고리즘 (Public Interface)
* 테스트 벡터
* 레퍼런스 구현 (Python / JS 등)

📌 *누구나 “이 결과가 유효한지” 판단할 수 있어야 한다.*

---

### 2.4 보안 가정 (Security Assumptions)

* 어떤 공격은 막고
* 어떤 공격은 범위 밖인지
* 왜 실행 재현이 불가능한지 (Blackbox Nature)

📌 *투명성은 신뢰의 전제다.*

---

## 3. 비공개 범위 (Sealed / Non-Public)

다음 항목은 **OpenSeal의 설계상 의도적으로 비공개**이며,
본 프로젝트의 핵심 보호 대상이다.

---

### 3.1 실행 캡슐 내부 구현 (Execution Capsule)

* **A-Hash 및 B-Hash와 관련된 내부 처리 구조**
* 내부 상태 처리 방식
* 중간 해시 / 메모리 / 스택 처리
* 실행 중 삽입되는 은닉 상태

📌 *“검증 가능” ≠ “재현 가능”*

---

### 3.2 내부 바인딩 함수 구조 (Internal Binding Structure)

* 요청 시점에 주입되는 비공개 엔트로피에 따른 내부 구조
* 내부 연산 규칙 및 분기
* 난독화 및 변형 방식

📌 *이 영역이 노출될 경우, 위조 결과 생성이 가능해진다.*

---

### 3.3 난수 분배 및 동기화 메커니즘

* R의 생성 시점
* 전달 방식
* 하드웨어/환경 결합 여부

📌 *이 정보는 공격자에게 실행 동기화 힌트를 제공한다.*

---

### 3.4 오케스트레이션 및 런타임 결합

* WASM / Native / Hybrid 구조
* Fetch / I/O 결합 방식
* 실행 환경 하드닝

📌 *이는 OpenSeal의 범위가 아니라, 실행 플랫폼의 영역이다.*

---

## 4. 중요한 경계 선언 (Critical Boundary Statement)

> **OpenSeal은 “검증 규칙”을 공개한다.
> OpenSeal은 “증명을 생성하는 방법”을 공개하지 않는다.**

이 경계는 의도적이며,
이를 침범하는 구현은 OpenSeal의 참조 구현이 아니다.

---

## 5. 플랫폼 구현에 대한 입장 (Platform Neutrality)

OpenSeal은 특정 플랫폼(예: HighStation)에 종속되지 않는다.

그러나:

* OpenSeal을 사용하여 **증명을 생성하는 실행 환경**은
  플랫폼 고유 구현이 될 수 있으며,
* 해당 구현은 OpenSeal 오픈소스 범위에 포함되지 않는다.

📌 *OpenSeal은 표준이고, 플랫폼은 선택이다.*

---

## 6. 라이선스 및 기여 가이드라인

* OpenSeal Spec / Verifier는 오픈소스 라이선스로 제공된다.
* 실행 생성 로직을 요구하는 PR은 거절될 수 있다.
* 본 지시문을 침해하는 포크는 “OpenSeal 호환”을 주장할 수 없다.

---

## 7. 최종 선언 (Final Declaration)

> OpenSeal은
> **“모든 사람이 검증할 수 있지만,
> 아무나 만들어낼 수는 없는 실행 증명”**을 목표로 한다.

이 공개/비공개 경계는
기술적 한계가 아니라 **설계 철학**이다.
