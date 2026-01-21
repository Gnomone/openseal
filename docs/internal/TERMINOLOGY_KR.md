# OpenSeal 핵심 용어 정의 (TERMINOLOGY)

**최종 수정**: 2026-01-22  
**목적**: 모든 용어의 정확한 정의 및 혼란 제거

---

## 📚 용어 사전

### 1. Root Hash (루트 해시)

**정의**: 프로젝트 소스 코드의 Merkle Tree Root Hash

**계산 방법**:
```rust
// openseal-core/src/lib.rs
pub fn compute_project_identity(path: &Path) -> Result<ProjectIdentity> {
    // 1. 모든 파일 스캔 (.opensealignore 적용)
    let files: Vec<PathBuf> = walk_files(path);
    
    // 2. 각 파일 Blake3 해시
    let hashes: Vec<Hash> = files.par_iter()
        .map(|f| blake3::hash(&fs::read(f)?))
        .collect()?;
    
    // 3. Merkle Root 계산
    let root_hash = compute_merkle_root(&hashes);
    
    Ok(ProjectIdentity { root_hash, file_count: files.len() })
}
```

**특징**:
- **정적** (프로젝트마다 고유값)
- **재현 가능** (동일 소스 = 동일 Hash)
- **Wax 무관** (Wax 없이 계산)

**예시**:
```
Root Hash: 14f385205520053548c8554925dce762bb61ecebfa3a9f1cd2e41d28ccb73a29
Files: 1630
```

**사용 목적**:
1. **프로젝트 Identity**: GitHub에 저장 (openseal.json)
2. **HighStation 등록**: Service 등록 시 사용
3. **검증 기준**: Truth Hash로 활용

**왜 필요한가?**:
- 소스 코드의 무결성 보장
- "What code produced this result?" 답변
- 변조 감지 (소스 변경 시 Hash 변경)

---

### 2. Wax (왁스)

**정의**: 요청별로 생성되는 일회용 난수 (Nonce)

**생성 주체**: **Client (검증자)**

**전달 방법**:
```http
GET /api/v1/price/BTC
X-OpenSeal-Wax: a1b2c3d4e5f67890...
```

**형식**:
- 16진수 문자열
- 길이: 16~64자 권장
- 예시: `"a1b2c3d4e5f67890"`

**사용 목적**:
1. **Freshness 보장**: 매 요청마다 다른 값
2. **Replay Attack 방지**: 이전 응답 재사용 불가
3. **요청-응답 Binding**: 특정 요청에 대한 응답임을 증명

**생성 예시**:
```javascript
// Client
const wax = crypto.randomBytes(16).toString('hex');
// → "a1b2c3d4e5f67890abcdef1234567890"
```

**왜 필요한가?**:
- 공격자가 과거 Seal을 재사용하는 것을 방지
- 각 요청마다 고유한 Seal 생성

---

### 3. Blinded A-hash (블라인드 A 해시)

**정의**: Root Hash와 Wax를 결합한 동적 Identity

**계산 방법**:
```rust
// openseal-core/src/lib.rs:215-220
pub fn compute_a_hash(root_hash: &Hash, wax: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"OPENSEAL_BLINDED_IDENTITY");  // Salt
    hasher.update(root_hash.as_bytes());          // Root Hash
    hasher.update(wax.as_bytes());                // Wax
    hasher.finalize()
}
```

**입력**:
- `root_hash`: Root Hash (정적)
- `wax`: Wax (동적)

**출력**:
```
A-hash = Blake3("OPENSEAL_BLINDED_IDENTITY" || Root Hash || Wax)
```

**특징**:
- **동적** (Wax마다 다름)
- **일방향** (A-hash로부터 Root Hash 추출 불가)
- **Binding** (Root Hash + Wax 결합)

**예시**:
```
Root Hash: 14f38520...
Wax:       a1b2c3d4...
→ A-hash:  e5f7a9b2... (매번 다름)
```

**사용 목적**:
1. **B-hash 계산**: b_G 함수의 입력
2. **Root Hash 은폐**: Root Hash 직접 노출 방지
3. **서명 생성**: Signature payload에 포함

**왜 필요한가?**:
- Root Hash를 직접 사용하면 Wax 효과 상실
- Blinding으로 요청별 고유 Identity 생성

---

### 4. b_G 함수 (B-hash 생성 함수)

**정의**: Result와 Identity를 Binding하는 비공개 해시 함수

**위치**: `crates/openseal-secret/src/lib.rs` (🔒 비공개)

**시그니처**:
```rust
pub fn compute_b_hash(
    a_hash: &Hash,     // Blinded A-hash
    wax: &str,         // Wax
    result: &[u8]      // API 응답 결과
) -> Hash
```

**입력**:
1. `a_hash`: Blinded A-hash
2. `wax`: Wax (중복이지만 보안 강화)
3. `result`: 정규화된 API 응답 bytes

**출력**:
```
B-hash = b_G(A-hash, Wax, Result)
```

**특징**:
- **비공개**: GitHub에 소스 미포함
- **난독화**: 역공학 방지
- **바이너리**: openseal-linux에 정적 링크

**예시**:
```
A-hash:  e5f7a9b2...
Wax:     a1b2c3d4...
Result:  {"symbol":"BTC","price":"98500"}
→ B-hash: 9f3e2a7c...
```

**사용 목적**:
1. **Result Binding**: 응답과 Identity 결합
2. **위조 방지**: b_G 함수 없이는 B-hash 생성 불가
3. **Seal 고유성**: 동일 결과도 Wax마다 다른 B-hash

**왜 비공개인가?**:
- 공개 시 공격자가 임의 B-hash 생성 가능
- 비공개로 Seal 위조 방지

---

### 5. B-hash (결과 바인딩 해시)

**정의**: b_G 함수의 출력, Result-Identity Binding Hash

**계산 위치**: Runtime (openseal-runtime)

**실제 코드**:
```rust
// openseal-runtime/src/lib.rs:145
use openseal_secret::compute_b_hash;

let b_hash = compute_b_hash(&a_hash, &wax_hex, standardized_bytes);
let b_hash_hex = b_hash.to_hex().to_string();
```

**입력**:
- `a_hash`: Blinded A-hash
- `wax_hex`: Wax (문자열)
- `standardized_bytes`: 정규화된 결과 (`serde_json::to_string`)

**특징**:
- **고유성**: (A-hash, Wax, Result) 조합마다 고유
- **검증 가능**: 검증자가 재계산 가능 (openseal-core의 검증용 함수)
- **Opaque**: B-hash만으로는 Result 추출 불가

**예시**:
```
Request:
  Wax: a1b2c3d4...
  Path: /api/v1/price/BTC

Response:
  Result: {"symbol":"BTC","price":"98500"}
  B-hash: 9f3e2a7c...
```

**사용 목적**:
1. **Binding 증명**: 이 결과가 이 요청에 대한 것임을 증명
2. **서명 생성**: Signature payload에 포함
3. **검증**: 검증자가 B-hash 재계산하여 확인

---

### 6. Signature (서명)

**정의**: Ed25519 디지털 서명

**계산 위치**: Runtime

**실제 코드**:
```rust
// openseal-runtime/src/lib.rs:148-154
// 1. Result Hash (검증용)
let result_hash = blake3::hash(standardized_bytes).to_hex().to_string();

// 2. Signature Payload 구성
let sign_payload = format!("{}{}{}{}", 
    wax_hex,        // Wax
    a_hash_hex,     // Blinded A-hash
    b_hash_hex,     // B-hash
    result_hash     // Result Hash (Blake3)
);

// 3. Ed25519 서명
let sig = state.signing_key.sign(sign_payload.as_bytes());
```

**Payload 구성**:
```
Payload = Wax || A-hash || B-hash || Blake3(Result)
```

**예시**:
```
Wax:         a1b2c3d4...
A-hash:      e5f7a9b2...
B-hash:      9f3e2a7c...
Result Hash: c4d3e2f1... (Blake3)

→ Payload:   a1b2c3d4e5f7a9b29f3e2a7cc4d3e2f1...
→ Signature: <64 bytes Ed25519 서명>
```

**서명 키**:
- **Ephemeral Private Key**: 매 Runtime 시작마다 새로 생성
- **Public Key**: 응답에 포함 (검증용)

**사용 목적**:
1. **무결성 증명**: Payload가 변조되지 않았음을 증명
2. **출처 인증**: Runtime이 생성한 Seal임을 증명
3. **Non-repudiation**: 부인 방지

**검증 방법**:
```rust
// openseal-core/src/lib.rs:248 (verify_seal 함수)
let verifying_key = VerifyingKey::from_bytes(&pub_key_bytes)?;
let signature = Signature::from_bytes(&sig_bytes)?;

// Payload 재구성
let payload = format!("{}{}{}{}", wax, a_hash, b_hash, result_hash);

// 서명 검증
verifying_key.verify(payload.as_bytes(), &signature)?;
```

---

## 🔄 전체 플로우

### Build Phase

```
Source → WalkBuilder → Merkle Tree → Root Hash
                                          ↓
                                    openseal.json
```

### Runtime Phase (API 요청마다)

```
1. Wax 수신
   Client → Runtime: "a1b2c3d4..."

2. Blinded A-hash 계산
   A-hash = compute_a_hash(Root Hash, Wax)
          = Blake3("OPENSEAL_BLINDED_IDENTITY" || Root Hash || Wax)

3. Internal App 호출
   Runtime → Internal App → Result

4. B-hash 계산 (🔒 비공개)
   B-hash = compute_b_hash(A-hash, Wax, Result)

5. Signature 생성
   Payload = Wax || A-hash || B-hash || Blake3(Result)
   Signature = Ed25519.sign(Payload, Private Key)

6. Seal 구성
   {
     "signature": "<signature>",
     "pub_key": "<public_key>",
     "a_hash": "<a_hash>",
     "b_hash": "<b_hash>"
   }

7. 응답
   {
     "result": { ... },
     "openseal": { ... }
   }
```

### Verification Phase

```
1. Root Hash 가져오기
   GitHub openseal.json → Root Hash

2. Expected A-hash 계산
   Expected A-hash = compute_a_hash(Root Hash, Wax)

3. Response A-hash 비교
   Response A-hash == Expected A-hash ?

4. Signature 검증
   Payload 재구성 → Ed25519.verify(...)

5. 결과
   ✅ 모든 검증 통과 → Verified
   ❌ 하나라도 실패 → Rejected
```

---

## 📊 용어 비교표

| 용어 | 입력 | 출력 | 정적/동적 | 공개/비공개 |
|------|------|------|-----------|-------------|
| **Root Hash** | Source Files | Hash | 정적 | 공개 |
| **Wax** | - (Client 생성) | Nonce | 동적 | 공개 |
| **Blinded A-hash** | Root Hash + Wax | Hash | 동적 | 공개 |
| **b_G 함수** | - | - | - | 🔒 비공개 |
| **B-hash** | A-hash + Wax + Result | Hash | 동적 | 공개 |
| **Signature** | Wax + A-hash + B-hash + Result Hash | 서명 | 동적 | 공개 |

---

## 🎯 핵심 포인트

### 1. Wax는 누가 생성하는가?

✅ **Client (검증자)**가 생성

**이유**:
- Client가 Nonce를 제어해야 Replay 방지 보장
- Server가 생성하면 조작 가능

### 2. A-hash는 무엇을 위한 것인가?

✅ **Root Hash + Wax Binding**

**목적**:
- Root Hash를 직접 노출하지 않음
- 요청마다 다른 Identity 생성

### 3. B-hash는 무엇으로 만드는가?

✅ **A-hash + Wax + Result**

**정확한 입력**: `compute_b_hash(&a_hash, &wax, result_bytes)`

### 4. 서명은 무엇으로 만드는가?

✅ **Wax + A-hash + B-hash + Blake3(Result)**

**Payload**:
```
format!("{}{}{}{}", wax, a_hash_hex, b_hash_hex, result_hash)
```

---

## 🔒 보안 설계

### 공개 vs 비공개

**공개 (검증 가능성)**:
- Root Hash 계산 로직
- Blinded A-hash 계산 로직
- 서명 검증 로직

**비공개 (위조 방지)**:
- b_G 함수 (compute_b_hash)
- Private Signing Key (Ephemeral)

### 왜 이렇게 설계했는가?

**투명성**:
- 검증자가 모든 Hash를 재계산 가능
- Signature 검증 가능

**보안성**:
- b_G 함수 없이는 B-hash 생성 불가
- B-hash 없이는 유효한 Signature 생성 불가

---

## 📝 FAQ

### Q: Root Hash와 A-hash가 다른가?

✅ **다릅니다!**
- Root Hash: 정적 (Wax 없음)
- A-hash: 동적 (Wax 포함)

### Q: Wax를 왜 A-hash와 B-hash 둘 다에 사용하는가?

✅ **보안 강화 및 명시성**
- A-hash: Root Hash Blinding
- B-hash: 중복이지만 b_G 함수 보안 강화

### Q: 서명만 있으면 B-hash는 필요 없는가?

❌ **둘 다 필요합니다!**
- B-hash: Result-Identity Binding 증명
- Signature: B-hash 포함한 전체 Payload 무결성 증명

---

**문서 버전**: 1.0  
**최종 검토**: 2026-01-22  
**다음 업데이트**: 용어 사용 혼란 발견 시
