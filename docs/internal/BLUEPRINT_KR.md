# OpenSeal 시스템 블루프린트 (BLUEPRINT)

**문서 버전**: 1.0  
**대상 독자**: OpenSeal 개발자 및 기여자  
**목적**: 시스템 아키텍처 및 보안 메커니즘 이해

---

## 📐 시스템 개요

OpenSeal은 **투명하고 검증 가능한 API 무결성 보장 시스템**입니다.

### 핵심 원칙

1. **투명성**: 검증 로직은 공개, 누구나 검증 가능
2. **보안성**: Seal 생성 로직은 비공개, 위조 방지
3. **재현성**: 동일한 소스 = 동일한 Root Hash
4. **Freshness**: Wax(nonce)로 Replay Attack 방지

---

## 🏗️ 아키텍처 레이어

### 전체 구조

```
┌────────────────────────────────────────┐
│ GitHub (Truth Source)                  │
│ - 소스 코드 (공개)                      │
│ - openseal.json (Root Hash)            │
└────────────────────────────────────────┘
              ↓ openseal build
┌────────────────────────────────────────┐
│ dist_opensealed (Sealed Bundle)        │
│ - 소스 복사 (암호화 없음)               │
│ - node_modules: symlink                │
│ - openseal.json                        │
└────────────────────────────────────────┘
              ↓ openseal run
┌────────────────────────────────────────┐
│ Runtime (5 Layers)                     │
│ 1. Wax 추출                            │
│ 2. A-hash 생성                         │
│ 3. Internal App 실행                   │
│ 4. Seal 생성 (🔒 비공개)               │
│ 5. 응답 전송                           │
└────────────────────────────────────────┘
```

---

## 🔑 핵심 개념

### Root Hash vs Blinded A-hash

| 개념 | 정의 | 계산 | 특징 |
|------|------|------|------|
| **Root Hash** | 프로젝트 Identity | `MerkleRoot(Files)` | 정적 (프로젝트마다 고유) |
| **Blinded A-hash** | 요청별 Identity | `H(Root Hash ‖ Wax)` | 동적 (요청마다 다름) |

**예시**:
```
Root Hash:       14f38520... (고정)
Wax:            a1b2c3d4...
Blinded A-hash: e5f7a9... (매번 다름)
```

**용도**:
- Root Hash → GitHub 저장, HighStation 등록
- Blinded A-hash → B-hash 계산, Replay 방지

---

## 📊 런타임 5단계 레이어

### Layer 1: 요청 수신 및 Wax 추출

**역할**: HTTP 요청 처리, Wax(nonce) 추출

**입력**:
```http
GET /api/v1/price/BTC
X-OpenSeal-Wax: a1b2c3d4...
```

**처리**:
```rust
let wax = req.headers().get("X-OpenSeal-Wax")?;
```

**출력**: `wax: String`

**보안**: 🟢 공개 (openseal-runtime)

---

### Layer 2: Identity 계산

**역할**: Blinded A-hash 생성

**입력**:
- `root_hash: Hash` (startup 시 계산)
- `wax: String`

**처리**:
```rust
// startup 시 한 번
let root_hash = compute_project_identity(&dist_opensealed)?;

// 요청마다
let a_hash = compute_a_hash(&root_hash, &wax);
```

**compute_a_hash 구현**:
```rust
pub fn compute_a_hash(root_hash: &Hash, wax: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"OPENSEAL_BLINDED_IDENTITY");
    hasher.update(root_hash.as_bytes());
    hasher.update(wax.as_bytes());
    hasher.finalize()
}
```

**출력**: `a_hash: Hash` (Blinded A-hash)

**보안**: 🟢 공개 (openseal-core)

---

### Layer 3: Internal App 실행

**역할**: 원본 API 서버 호출

**입력**:
- `target_url: String` (예: localhost:4000)
- `path: String` (예: /api/v1/price/BTC)

**처리**:
```rust
let response = client
    .request(method, &target_url + path)
    .headers(headers)  // Wax 포함
    .send()
    .await?;

let result_bytes = response.bytes().await?;

// 정규화 (재현성 보장)
let result_json: Value = serde_json::from_str(&result_bytes)?;
let standardized = serde_json::to_string(&result_json)?;
```

**출력**: `standardized_bytes: &[u8]`

**보안**: 🟢 공개 (openseal-runtime)

---

### Layer 4: Seal 생성 🔒

**역할**: B-hash 계산 및 서명 생성

**입력**:
- `a_hash: Hash`
- `wax: String`
- `result: &[u8]`

**처리**:
```rust
use openseal_secret::compute_b_hash;  // 🔒 비공개!

// B-hash 계산
let b_hash = compute_b_hash(&a_hash, &wax, result);

// 서명 페이로드
let payload = format!("{}{}{}{}", 
    wax, 
    a_hash.to_hex(), 
    b_hash.to_hex(), 
    blake3::hash(result).to_hex()
);

// Ed25519 서명
let signature = signing_key.sign(payload.as_bytes());
```

**출력**: `Seal { signature, pub_key, a_hash, b_hash }`

**보안**: 🔒 **고도 보호**
- openseal-secret: GitHub 미포함
- 바이너리 정적 링크
- 기계어로 컴파일

---

### Layer 5: 응답 구성

**역할**: 최종 응답 래핑

**입력**:
- `result: Value`
- `seal: Seal`

**처리**:
```rust
serde_json::json!({
    "result": result,
    "openseal": seal
})
```

**출력**:
```json
{
  "result": { "symbol": "BTC", "price": "98500" },
  "openseal": {
    "signature": "...",
    "pub_key": "...",
    "a_hash": "...",
    "b_hash": "..."
  }
}
```

**보안**: 🟢 공개 (openseal-runtime)

---

## 🔐 보안 메커니즘

### 크레이트별 보안 수준

| 크레이트 | 공개 여부 | 역할 | 보호 방법 |
|----------|-----------|------|-----------|
| openseal-core | ✅ 공개 | Root Hash, 검증 | - (투명성) |
| openseal-runtime | ✅ 공개 | Proxy, Intercept | - (투명성) |
| openseal-cli | ✅ 공개 | Build, Verify | - (투명성) |
| **openseal-secret** | ❌ **비공개** | B-hash, 서명 | `.gitignore` + 바이너리 링크 |

### openseal-secret 보호

**.gitignore 설정**:
```gitignore
crates/openseal-secret/
```

**배포**:
```
소스 (로컬만) → Rust 컴파일 → 기계어 → openseal-linux
```

**검증**:
```bash
$ nm openseal-linux | grep compute_b_hash
00000000003fbe60 T _ZN15openseal_secret14compute_b_hash...
# ✅ 심볼 존재 (기계어로 링크됨)

$ git ls-files | grep openseal-secret
# (no output) ✅ GitHub에 없음
```

---

## 🔄 전체 실행 플로우

### Build Phase

```
1. Source 스캔
   └─ WalkBuilder + .opensealignore
      └─ node_modules 제외

2. Root Hash 계산
   └─ MerkleRoot(Files)
      → 14f38520...

3. dist_opensealed 생성
   ├─ 파일 복사
   ├─ node_modules: symlink
   └─ openseal.json 저장
```

### Runtime Phase

```
1. Startup
   ├─ dist_opensealed 스캔
   ├─ Root Hash 재계산
   └─ Ed25519 키 생성

2. 요청 처리 (각 API 호출마다)
   ├─ Wax 추출
   ├─ Blinded A-hash 생성
   ├─ Internal App 호출
   ├─ Seal 생성 (🔒)
   └─ 응답 전송
```

### Verification Phase

```
1. 응답 수신
   └─ { result, openseal }

2. Root Hash 확인
   └─ GitHub openseal.json

3. Blinded A-hash 재계산
   └─ H(Root Hash || Wax)

4. 서명 검증
   └─ Ed25519.verify(...)

5. 결과
   └─ ✅ 모든 검증 통과
```

---

## ⚙️ 주요 함수

### compute_project_identity

**목적**: Root Hash 계산

```rust
pub fn compute_project_identity(path: &Path) -> Result<ProjectIdentity> {
    // 1. WalkBuilder로 파일 스캔
    let walker = WalkBuilder::new(path)
        .git_ignore(false)  // .gitignore 무시
        .add_custom_ignore_filename(".opensealignore")
        .build();
    
    // 2. 각 파일 해시
    let hashes: Vec<Hash> = files.par_iter()
        .map(|f| blake3::hash(&fs::read(f)?))
        .collect()?;
    
    // 3. Merkle Root
    let root_hash = compute_merkle_root(&hashes);
    
    Ok(ProjectIdentity { root_hash, file_count: files.len() })
}
```

### compute_a_hash

**목적**: Blinded A-hash 생성

```rust
pub fn compute_a_hash(root_hash: &Hash, wax: &str) -> Hash {
    blake3::hash(&[
        b"OPENSEAL_BLINDED_IDENTITY",
        root_hash.as_bytes(),
        wax.as_bytes()
    ].concat())
}
```

### compute_b_hash (비공개)

**목적**: Result Binding

```rust
// openseal-secret/src/lib.rs (비공개!)
pub fn compute_b_hash(a_hash: &Hash, wax: &str, result: &[u8]) -> Hash {
    // 🔒 복잡한 난독화 로직
    // 역공학 방지
    // ...
}
```

---

## 🎯 설계 철학

### 왜 소스를 암호화하지 않는가?

**답변**: 암호화하면 검증 불가능

**OpenSeal의 철학**:
- "What code produced this result?"
- 사용자가 **소스를 읽고** 검증할 수 있어야 함
- 암호화 ❌ → 투명성 ✅

**대신 무엇을 보호하는가?**:
- ✅ Seal 생성 로직 (openseal-secret)
- ✅ 서명 키 (Ephemeral, 메모리만)

### 왜 Root Hash와 Blinded A-hash 둘 다 필요한가?

**Root Hash**:
- 프로젝트 Identity
- GitHub 저장, 공개
- HighStation 등록

**Blinded A-hash**:
- Replay Attack 방지
- Wax 없이는 재사용 불가
- B-hash Binding용

### 왜 매 실행마다 새 서명 키를 생성하는가?

**답변**: 키 유출 위험 최소화

**Ephemeral Key**:
- 매 `openseal run`마다 새로 생성
- 메모리에만 존재
- Runtime 종료 시 소멸

**장점**:
- 키 유출 시 피해 최소화 (해당 세션만)
- 검증자는 Public Key로 검증 가능

---

## 🛡️ 보안 고려사항

### Runtime 무결성 검증 (v0.2.6 구현!)

**현재**: ✅ **구현 완료**

**구현 내용**:
```rust
// openseal-runtime/src/lib.rs (startup 시)
pub async fn run_proxy_server(...) -> Result<()> {
    // 1. Live Hash 계산
    let live_identity = compute_project_identity(&dist_opensealed)?;
    
    // 2. Expected Hash 로드 (openseal.json)
    let manifest = load_openseal_json(&dist_opensealed)?;
    let expected_hash = extract_root_hash(&manifest)?;
    
    // 3. 무결성 검증
    if live_identity.root_hash != expected_hash {
        eprintln!("🚨 INTEGRITY VIOLATION DETECTED");
        eprintln!("Expected: {}", hex::encode(&expected_hash));
        eprintln!("Actual:   {}", live_identity.root_hash.to_hex());
        return Err(anyhow!("Integrity violation - Runtime aborted"));
    }
    
    println!("✅ Integrity Verified!");
    // ... 계속
}
```

**효과**:
- dist_opensealed 변조 시 **즉시 탐지**
- Runtime 시작 전 차단
- 악의적 코드 실행 방지

**동작 예시**:
```bash
# 정상 케이스
$ openseal run --app dist_opensealed --port 1999
   ✅ Live A-hash: 14f38520...
   ✅ Integrity Verified!
   🚀 OpenSeal Running

# 변조 케이스
$ vim dist_opensealed/src/index.ts  # 수정
$ openseal run --app dist_opensealed --port 1999
   🚨 INTEGRITY VIOLATION DETECTED
   Expected: 14f38520...
   Actual:   XXXXXXXX...
   Error: Runtime aborted
```

### .git 폴더 제외

**문제**: Git 메타데이터가 Hash에 포함

**해결**: `.opensealignore`에 추가
```
.git/
```

---

## 📚 용어 사전

| 용어 | 정의 |
|------|------|
| **Root Hash** | 프로젝트 고유 Identity (정적) |
| **Blinded A-hash** | Wax로 은폐된 Identity (동적) |
| **Wax** | 요청별 Nonce (Freshness 보장) |
| **B-hash** | Result-Identity Binding |
| **Seal** | 서명 + 메타데이터 |
| **Ghosting** | 의존성 Symlink (Hash 제외) |

---

## 🔄 버전 이력

### v0.2.6 (2026-01-22)
- ✅ Runtime 무결성 검증 구현
- dist_opensealed 변조 시 즉시 탐지
- openseal.json Expected Hash vs Live Hash 비교

### v0.2.5 (2026-01-22)
- Root Hash vs Blinded A-hash 개념 정립
- openseal-secret 비공개 확인
- Runtime 5단계 레이어 문서화

### v0.2.4 (2026-01-19)
- Daemon 모드 추가
- Ghosting 로직 개선
- Symlink 안전성 강화

---

**작성자**: OpenSeal Development Team  
**최종 수정**: 2026-01-22
