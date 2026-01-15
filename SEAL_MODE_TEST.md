# 환경별 Seal 모드 테스트 가이드

## 준비사항
OpenSeal v2.0-rc15부터 환경별 Seal 반환 모드를 지원합니다.

## 모드 설명

### Development 모드 (기본값)
- **활성화**: 환경 변수 미설정 또는 `OPENSEAL_MODE=development`
- **반환**: 전체 Seal (`signature`, `wax`, `pub_key`, `a_hash`, `b_hash`)
- **용도**: 디버깅, 로컬 개발, 검증 로직 테스트

### Production 모드
- **활성화**: `OPENSEAL_MODE=production`
- **반환**: `signature`만
- **용도**: 프로덕션 배포, 보안 강화, 데이터 최소화

## 테스트 방법

### 1. Development 모드 테스트
```bash
# 환경 변수 없이 실행 (기본값)
cd openseal/crates/openseal-cli
cargo run -- run ./example-app --port 7325

# 또는 명시적으로
OPENSEAL_MODE=development cargo run -- run ./example-app --port 7325
```

**예상 응답**:
```json
{
  "result": { ... },
  "openseal": {
    "signature": "a1b2c3...",
    "wax": "d4e5f6...",
    "pub_key": "g7h8i9...",
    "a_hash": "j0k1l2...",
    "b_hash": "m3n4o5..."
  }
}
```

### 2. Production 모드 테스트
```bash
OPENSEAL_MODE=production cargo run -- run ./example-app --port 7325
```

**예상 응답**:
```json
{
  "result": { ... },
  "openseal": {
    "signature": "a1b2c3..."
  }
}
```

## 검증자 구현 가이드

Production 모드에서 검증자는 다음 정보로 서명을 재구성해야 합니다:

```javascript
// 1. 필요한 정보 준비
const wax = "본인이 보낸 챌린지";
const projectRoot = "서비스 소스코드";
const result = received_result;
const pub_key = "사전 등록된 노드 공개키";

// 2. a_hash 계산
const a_hash = Hash(projectRoot + wax);

// 3. b_hash 계산 (g_B 로직 필요)
const b_hash = g_B(a_hash, wax, result);

// 4. 서명 페이로드 재구성
const payload = wax + a_hash + b_hash + Hash(result);

// 5. 서명 검증
const isValid = verify_signature(pub_key, payload, received_signature);
```

## 주의사항
- **기본값이 Development**이므로 프로덕션 배포 시 **반드시** `OPENSEAL_MODE=production` 설정 필요
- Production 모드에서는 디버깅 정보가 없으므로 검증 실패 시 원인 파악이 어려움
- 개발 단계에서 검증 로직을 완전히 검증한 후 Production 모드 사용 권장
