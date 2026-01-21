# 🧠 OpenSeal: 내부 로직 상세 사양 (Internal Logic)

본 문서는 OpenSeal v2.0의 핵심 엔진 내부에서 작동하는 기밀 로직을 기록합니다. 이 로직들은 보안상의 이유로 퍼블릭 명세(OSIP)에는 포함되지 않으며, 오직 인가된 개발자 및 시스템 보호를 위해 관리됩니다.

///
네, 헷갈릴 수 있는 부분입니다. 구조를 설명해 드릴게요.

결론부터 말씀드리면: **"봉투(dist_opensealed) 안에 내용물(dist)이 들어있는 구조"**입니다.

실행 구조
openseal build ... --output dist_opensealed
👉 이 명령어는 프로젝트 전체를 복사해서 dist_opensealed라는 폴더(봉투)를 만듭니다.
text
/app/dist_opensealed/  (봉인된 봉투)
├── package.json
├── openseal.json
└── dist/              (실제 컴파일된 코드 - 내용물)
    └── index.js
실행 명령 (
Dockerfile
 안에서)
openseal run --app dist_opensealed
👉 OpenSeal은 dist_opensealed 안으로 들어갑니다. (cd dist_opensealed)
거기서 node dist/index.js를 실행합니다.
아까 난 에러 이유
.gitignore
 때문에 dist 폴더가 복사되지 않았습니다.
그래서 봉투(dist_opensealed)는 만들어졌는데, 열어보니 정작 알맹이(dist/index.js)가 없었던 겁니다.
이제 !dist 설정을 추가했으니, 봉투 안에 내용물이 잘 들어갈 겁니다.


정리해 드리겠습니다.

요청 (Request) -> 서버로 보낼 때:
Header에 담습니다. (X-OpenSeal-Wax: ...)
Body를 건드리지 않습니다.
응답 (Response) -> 클라이언트가 받을 때:
Body에 담겨 옵니다. (OpenSeal Wrapper)
원래 결과는 result 안에, 서명 정보는 openseal 안에 들어갑니다.
[코드 확인 결과] openseal-runtime/src/lib.rs (169-172번 줄):

rust
let final_response = serde_json::json!({
    "result": result_json,  // 원래 오라클이 주는 가격 데이터
    "openseal": seal        // 서명, 해시값 등
});
즉, 보낼 때는 헤더에 넣고, 받을 때는 바디를 까보면 됩니다.

////



---

## 1. B-hash 생성 엔진 (b_G 함수)

`b_G` 함수는 실행 결과(`Result`)를 코드 정체성(`A-hash`) 및 세션 난수(`Wax`)에 암호학적으로 바인딩하는 OpenSeal의 심장부입니다.

### 1.1 하드닝된 키 유도 (Hardened Key Derivation)
B-hash를 생성하기 위해 단순한 해싱을 쓰지 않고, 매 요청마다 **다성성(Polymorphic) 연산 키**를 유도합니다.

1.  **Context Mixing**: 프로젝트 루트 해시(`A-hash`)를 기반으로 고유한 컨텍스트 시드를 생성합니다.
2.  **Non-linear Wax Expansion**: 검증자가 보낸 `Wax`의 각 바이트를 위치 기반 염금(Salt)과 혼합하여 비선형적으로 확장합니다. 이는 난수 값뿐만 아니라 **난수의 길이와 비트 배열**에 따라서도 연산 경로가 바뀌게 만듭니다.
3.  **Recursive State Evolution**: 중간 해시 결과물과 `A`, `Wax`를 교차 해싱하여 내부 상태를 끊임없이 진화(Evolve)시킵니다.
4.  **Alternating Bitwise Mix**: 최종적으로 `A-hash`와 중간 상태값들을 덧셈(Wrapping add)과 XOR 연산으로 교차 혼합하여 최종 32바이트 물리 키를 생성합니다.

### 1.2 키 해싱 (Keyed Hashing)
유도된 32바이트 하드닝 키를 사용하여 결과값(`Result`)에 대해 **Keyed BLAKE3** 해싱을 수행합니다. 
- 결과: `B = blake3::keyed_hash(derived_key, result)`

### 1.3 보안 가치
이 과정은 단순히 결과를 요약하는 것을 넘어, **"이 로직을 모르는 공격자는 절대 동일한 B-hash를 만들어낼 수 없다"**는 강력한 계산적 장벽을 생성합니다. 이는 OpenSeal이 TEE 없이도 소프트웨어 수준에서 높은 수준의 무결성을 보장할 수 있는 핵심 근거입니다.

---

## 2. CLI 검증 명령어 (`openseal verify`)

`openseal verify` 명령어는 개발 및 테스트 환경에서 OpenSeal 응답의 무결성을 확인하는 도구입니다.

### 2.1 입력 및 동작

**명령어 형식:**
```bash
openseal verify --response result.json --wax "난수값" --root-hash "예상-A-hash"
```

**입력 파일 구조 (`result.json`):**
```json
{
  "result": { /* 실제 API 응답 */ },
  "openseal": {
    "signature": "...",
    "pub_key": "...",
    "a_hash": "...",
    "b_hash": "..."
  }
}
```

### 2.2 검증 로직 (`openseal-core::verify_seal`)

1. **서명 검증**: `openseal.pub_key`를 사용하여 `signature`가 `(A, B, Wax, result_hash)` 페이로드에 대해 유효한지 확인합니다.
2. **Wax 일치**: 명령줄에서 제공된 `--wax` 값이 응답에 포함된 Wax와 일치하는지 확인합니다.
3. **코드 정체성**: (선택) `--root-hash`가 제공되면, `openseal.a_hash`가 `compute_a_hash(root_hash, wax)`와 일치하는지 확인합니다.

### 2.3 보안 고려사항

- **공개 검증 로직**: `verify_seal` 함수는 `openseal-core`에 공개되어 있으며, 누구나 감사할 수 있습니다.
- **플랫폼 독립성**: 검증은 중앙 플랫폼 없이 작동하며, API 소비자가 직접 실행할 수 있습니다.
- **개발 도구**: 이 명령어는 공급자의 자가 테스트 및 디버깅용이며, 프로덕션에서는 소비자가 독립 검증기를 사용합니다.

---

## 3. 향후 발전 계획 (Future Evolution)
- **Instruction-level Polymorphism**: A-hash 값에 따라 연산 순서 자체가 실제 기계어 수준에서 바뀌는 VM 기반 난독화 엔진 도입 예정.
