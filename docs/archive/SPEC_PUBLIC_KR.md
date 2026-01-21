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

| 필드 | 자료형 | 설명 | 검증 가능 여부 |
|:---:|:---:|:---|:---:|
| `pub_key` | `String (Hex)` | **실행자 정체성** (일회용 세션 공개키) | ✅ Public Assertion |
| `a_hash` | `String (Hex)` | **블라인드된 사전 상태값** (프로젝트 + Wax) | ✅ Public Assertion |
| `b_hash` | `String (Hex)` | **사후 상태 식별자 (Post-State ID)** | ✅ Public Assertion |
| `wax` | `String (Any)` | **챌린지 문맥** (유일성 & 검증용) | ✅ Public Assertion |
| `signature` | `String (Hex)` | 위 데이터들에 대한 OpenSeal 런타임의 전자서명 | ✅ Public Assertion |

### 2.2 결과 (Result)
실제 API 서버가 반환한 응답 데이터(JSON, String, Binary 등)입니다. OpenSeal은 이 데이터를 "값"이 아닌 "상태 전이의 증거"로 취급합니다.

---

## 3. 검증 프로세스 (Verification Process)

검증자는 다음 단계를 수행하여 유효성을 판단해야 합니다.

### Step 1: 소스코드-결과 바인딩 검증 (Code-Result Binding Check)
*   **개념**: "이 결과가 변조되지 않은 오리지널 소스코드에서 생성되었는가?"
*   **방법**: 서버(HighStation)는 등록된 소스코드 정보와 `Wax`를 결합하여 `a_hash`를 재계산하고, 제출된 `Seal`의 데이터들이 논리적으로 일치하는지 Assert합니다.
*   **보증**: `b_hash`가 유효하다면, 해당 결과(`Result`)는 반드시 해당 코드(`A-hash`)를 통해서만 물리적으로 생성될 수 있었음을 수학적으로 확신할 수 있습니다.

### Step 2: 실행 문맥 검증 (Challenge Check)
*   **개념**: "결과가 재사용(Replay)된 것이 아니고, 이번에 던진 질문(`Wax`)에 대한 답인가?"
*   **방법**: `Seal.wax`가 이번 요청에서 생성한 `Wax`와 일치하는지, 그리고 이전에 사용된 적이 없는지 확인합니다.

### Step 3: 서명 검증 (Authenticity Check)
*   **개념**: "이 봉인이 신뢰할 수 있는 OpenSeal 런타임에 의해 서명되었는가?"
*   **방법**: `Seal.signature`가 `Seal` 데이터 내용에 대해 유효한지 검증합니다.
*   **키 수명주기 (Security)**: 서명 키(`Pub-Key`)는 런타임 시작 시 메모리(RAM)에서만 생성되는 **일회성 세션 키(Ephemeral Session Key)**입니다. 디스크에 영구 **저장되지 않으며**, 런타임을 재시작하면 키가 변경되어 이전 키의 유출이 미래의 보안을 위협하지 않습니다(Forward Secrecy).

---

## 4. 실패 조건 (Failure Cases)

다음 중 하나라도 해당되면 **INVALID**로 판정해야 합니다.

1.  **Identity Mismatch**: 제출된 `A-hash`가 기대하는 식별자와 다름.
2.  **Binding Failure**: Seal이 실행 결과와 논리적으로 일치하지 않음.
3.  **Replay Attack**: 이미 사용된 `Wax`가 재사용됨. (재사용 시 검증은 실패함)
4.  **Signature Error**: 서명 검증 실패.

---

## 5. 결론
이 명세에 따라 구현된 검증기는 **"결과의 무결성"**을 수학적으로 확신할 수 있습니다.
단, 이 명세만으로는 유효한 `Seal`을 **생성(Generate)**할 수 없으며, 이는 OpenSeal 보안 모델의 핵심입니다.
