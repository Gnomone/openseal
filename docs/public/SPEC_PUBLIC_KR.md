# 🔍 OpenSeal v2.0 공개 검증 명세 (Public Verification Spec)

[🇺🇸 English Version](./SPEC_PUBLIC.md)

---
> 본 문서는 **OpenSeal의 검증 가능성(Verifiability)**을 설명하기 위한 것이며,
> 어떠한 경우에도 **유효한 Seal을 생성하거나 재현하는 방법**을 포함하지 않습니다.
> 생성 규칙, 결합 순서, 내부 상태 전이는 의도적으로 생략되거나 추상화됩니다.

---

## 1. 개요 (Overview)
OpenSeal 검증기(Verifier)는 주어진 결과(Result)와 봉인(Seal)이 **"등록된 실행 컨텍스트와 내부적으로 일관된 상태 전이를 나타내는지"**를 검증합니다.

### 검증 모델
```text
Verify(Result, Seal, PublicKeys) -> VALID | INVALID
```

---

## 2. 데이터 구조 (Data Structure)

### 2.1 봉인 (Seal)
OpenSeal 런타임이 반환하는 증명 객체입니다.

| 필드 | 설명 | 검증 가능 여부 |
|:---:|:---|:---:|
| `a_hash` | **사전 상태 식별자 (Pre-State ID)** | ✅ Public Assertion |
| `b_hash` | **사후 상태 식별자 (Post-State ID)** | ✅ Public Assertion |
| `nonce` | 실행의 **유일성(Uniqueness)**을 보장하기 위한 외부 식별자 | ✅ Public Assertion |
| `signature` | 위 데이터들에 대한 OpenSeal 런타임의 전자서명 | ✅ Public Assertion |

### 2.2 결과 (Result)
실제 API 서버가 반환한 응답 데이터(JSON, String, Binary 등)입니다. OpenSeal은 이 데이터를 "값"이 아닌 "상태 전이의 증거"로 취급합니다.

---

## 3. 검증 프로세스 (Verification Process)

검증자는 다음 단계를 수행하여 유효성을 판단해야 합니다.

### Step 1: A-hash 검증 (Identity Check)
*   **개념**: "이 결과가 내가 아는 그 프로젝트에서 나왔는가?"
*   **방법**: 로컬 소스코드(또는 알려진 머클루트)와 `Seal.a_hash`가 일치하는지 확인합니다.

### Step 2: B-hash 검증 (Binding Check)
*   **개념**: "결과가 조작되지 않았고, 해당 실행 맥락(Nonce)에서 나왔는가?"
*   **방법**: Verifier는 Seal에 포함된 검증 증명이 **실행 결과와 모순되지 않음을 확인(Assert)**합니다.
    *   *Note: Seal 내부 로직은 요청 단위로 변화하며, 외부에서 그 구조를 예측하거나 재사용할 수 없습니다.*

### Step 3: 서명 검증 (Authenticity Check)
*   **개념**: "이 봉인이 신뢰할 수 있는 OpenSeal 런타임에 의해 서명되었는가?"
*   **방법**: `Seal.signature`가 `Seal` 데이터 내용에 대해 유효한지 검증합니다.

---

## 4. 실패 조건 (Failure Cases)

다음 중 하나라도 해당되면 **INVALID**로 판정해야 합니다.

1.  **Identity Mismatch**: 제출된 `A-hash`가 기대하는 식별자와 다름.
2.  **Binding Failure**: Seal이 실행 결과와 논리적으로 일치하지 않음.
3.  **Replay Attack**: 이미 사용된 `Nonce`가 재사용됨. (재사용 시 검증은 실패함)
4.  **Signature Error**: 서명 검증 실패.

---

## 5. 결론
이 명세에 따라 구현된 검증기는 **"결과의 무결성"**을 수학적으로 확신할 수 있습니다.
단, 이 명세만으로는 유효한 `Seal`을 **생성(Generate)**할 수 없으며, 이는 OpenSeal 보안 모델의 핵심입니다.
