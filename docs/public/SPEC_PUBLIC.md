# 🔍 OpenSeal v2.0 공개 검증 명세 (Public Verification Spec)

> **⚠️ 경계 선언 (Boundary Statement)**
> 본 명세는 **검증을 위한 규칙(Verification Rules)**만을 정의하며,
> OpenSeal 실행 증명의 **생성 방법(Generation Recipe)**은 의도적으로 포함하지 않습니다.

---

## 1. 개요 (Overview)
OpenSeal 검증기(Verifier)는 주어진 결과(Result)와 봉인(Seal)이 **"특정 시점에, 특정 코드에 의해 정직하게 생성되었는지"**를 판단합니다.

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
| `a_hash` | **실행 전(Pre-execution)** 상태의 식별자 (프로젝트 정체성) | ✅ Public Verifiable |
| `b_hash` | **실행 후(Post-execution)** 상태의 봉인값 | ✅ Public Verifiable |
| `nonce` | 실행의 고유성을 보장하기 위해 주입된 난수 | ✅ Public Verifiable |
| `signature` | 위 데이터들에 대한 OpenSeal 런타임의 전자서명 | ✅ Public Verifiable |

### 2.2 결과 (Result)
실제 API 서버가 반환한 응답 데이터(JSON, String, Binary 등)입니다. OpenSeal은 이 데이터를 "값"이 아닌 "상태 전이의 증거"로 취급합니다.

---

## 3. 검증 프로세스 (Verification Process)

검증자는 다음 단계를 수행하여 유효성을 판단해야 합니다.

### Step 1: A-hash 검증 (Identity Check)
*   **개념**: "이 결과가 내가 아는 그 프로젝트에서 나왔는가?"
*   **방법**: 로컬 소스코드(또는 알려진 머클루트)와 `Seal.a_hash`가 일치하는지 비교합니다.
    *   `Local_Merkle_Root == Seal.a_hash`

### Step 2: B-hash 검증 (Binding Check)
*   **개념**: "결과가 조작되지 않았고, 해당 실행 맥락(Nonce)에서 나왔는가?"
*   **방법**: 제공된 `Result`, `Nonce`, `A-hash`를 결합하여 `Expected_B_hash`를 계산하고, `Seal.b_hash`와 비교합니다.
    *   `Function(Result, Nonce, A_hash) == Seal.b_hash`
    *   *Note: 구체적인 결합 함수는 구현체의 내부 로직에 따릅니다.*

### Step 3: 서명 검증 (Authenticity Check)
*   **개념**: "이 봉인이 신뢰할 수 있는 OpenSeal 런타임(또는 HighStation)에 의해 서명되었는가?"
*   **방법**: `Seal.signature`가 `Seal` 데이터 내용에 대해 유효한지 검증합니다.

---

## 4. 실패 조건 (Failure Cases)

다음 중 하나라도 해당되면 **INVALID**로 판정해야 합니다.

1.  **Identity Mismatch**: 제출된 `A-hash`가 기대하는 소스코드의 해시와 다름. (소스코드 변조 의심)
2.  **Binding Failure**: `Result`를 이용해 재계산한 `B-hash`가 제출된 값과 다름. (결과값 조작 의심)
3.  **Replay Attack**: 이미 사용된 `Nonce`가 재사용됨.
4.  **Signature Error**: 서명 검증 실패.

---

## 5. 결론
이 명세에 따라 구현된 검증기는 **"결과의 무결성"**을 수학적으로 확신할 수 있습니다.
단, 이 명세만으로는 유효한 `Seal`을 **생성(Generate)**할 수 없으며, 이는 OpenSeal 보안 모델의 핵심입니다.
