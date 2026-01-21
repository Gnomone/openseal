# 🔐 OpenSeal v1.0.0-alpha.1

[🇺🇸 English Version](./README.md)

**신뢰할 수 있는 컨테이너 실행기**: 모든 Docker 컨테이너를 암호학적 증명이 포함된 검증 가능한 API로 변환합니다.

> ⚠️ **알파 릴리즈**: 초기 프리뷰 버전입니다. 프로덕션 사용은 권장하지 않습니다.

---

## OpenSeal이란?

OpenSeal은 Docker 컨테이너를 **암호학적 신원**으로 래핑하여 모든 API 응답을:
- ✅ **검증 가능**: Ed25519 서명으로 진위 증명
- ✅ **변조 감지**: 수정 시 Seal 파괴
- ✅ **부인 방지**: 수학적 출처 증명

---

## 빠른 시작

### API 사용자용
```bash
# Seal된 API 조회
curl -H "X-OpenSeal-Wax: myChallenge" http://api.example.com/endpoint
```

암호학적 증명 포함 응답:
```json
{
  "openseal": {
    "a_hash": "...",
    "b_hash": "...",
    "signature": "...",
    "pub_key": "..."
  },
  "result": { "데이터": "결과" }
}
```

**[→ 제공자 가이드](./docs/public/PROVIDER_GUIDE_KR.md)** | **[→ Provider Guide (EN)](./docs/public/PROVIDER_GUIDE.md)**

### 개발자용 (시드 제공자)
```bash
# 1. API 생성
# 2. Dockerfile 작성
docker build -t my-api:v1 .

# 3. GitHub에 push
docker push ghcr.io/yourorg/my-api:v1

# 4. openseal.json 배포
openseal build --image ghcr.io/yourorg/my-api:v1
```

**[→ 시드 제공자 가이드](./docs/public/SEED_PROVIDER_GUIDE_KR.md)** | **[→ Seed Provider Guide (EN)](./docs/public/SEED_PROVIDER_GUIDE.md)**

### 검증자용

```bash
# Seal된 API 조회 및 검증
curl -H "X-OpenSeal-Wax: challenge" http://api.example.com/endpoint
```

**[→ 검증자 가이드](./docs/public/VERIFIER_GUIDE_KR.md)** | **[→ Verifier Guide (EN)](./docs/public/VERIFIER_GUIDE.md)**

---

## 예제: 암호화폐 가격 오라클

암호학적 증명이 포함된 검증된 암호화폐 가격:

```bash
# 제공자 측
git clone https://github.com/Gnomone/crypto-price-oracle.git
cd crypto-price-oracle
docker build -t crypto-oracle:v1 .
openseal build --image crypto-oracle:v1
openseal run --image crypto-oracle:v1 --port 8080

# 사용자 측
curl -X POST http://localhost:8080/api/v1/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: prove-it" \
  -d '{"symbol":"BTC"}'
```

**전체 예제**: [crypto-price-oracle](https://github.com/Gnomone/crypto-price-oracle)

---

## 작동 원리

```
클라이언트 → OpenSeal 프록시 → 컨테이너
            ↓
       1. A-hash 계산 (신원)
       2. 컨테이너로 전달
       3. B-hash 계산 (결과 바인딩)
       4. Ed25519 서명
            ↓
       Seal 포함 응답
```

**핵심 개념**:
- **Root Hash**: Docker Image Digest (불변 신원)
- **Wax**: 클라이언트 챌린지 (재생 공격 방지)
- **A-hash**: `Blake3(Root Hash || Wax)`
- **B-hash**: `b_G(A-hash, Wax, Result)` (비공개 함수)
- **Signature**: `Ed25519.sign(Wax||A||B||ResultHash)`

---

## 문서

### 사용자용
- **[사용자 가이드](./docs/public/USER_GUIDE_KR.md)**: Seal된 API 조회 및 검증 방법
- **[User Guide (EN)](./docs/public/USER_GUIDE.md)**

### 제공자용
- **[배포자 가이드](./docs/public/PROVIDER_GUIDE_KR.md)**: Seal된 서비스 배포 방법
- **[Provider Guide (EN)](./docs/public/PROVIDER_GUIDE.md)**

### 아키텍처 & 설계
- **[CHANGELOG](./CHANGELOG.md)**: 버전 히스토리
- **[V1 개발 지침](./docs/internal/v1/V1.0.0-ALPHA_DIRECTIVE_KR.md)**: 핵심 설계 결정
- **[보안 모델](./docs/internal/TERMINOLOGY_KR.md)**: 암호학 용어

---

## 왜 v1인가? (Docker 기반)

### v0 문제점
- ❌ 환경 파편화 (PATH, 의존성)
- ❌ 언어별 복잡성
- ❌ 재현성 문제

### v1 솔루션
- ✅ Docker = 표준화된 패키징
- ✅ 언어 독립적
- ✅ Image Digest = 암호학적 신원
- ✅ 산업 표준 격리

**[전체 설명](./docs/internal/v1/V1_TRANSITION_STRATEGY_KR.md)**

---

## 로드맵

- **v1.0.0-alpha.1** (현재): 핵심 기능 작동
- **v1.0.0-beta.1** (다음): 네트워크 화이트리스트, 레지스트리 강제, 이미지 스캐닝
- **v1.0.0** (안정): 프로덕션 준비

---

## 커뮤니티

- **GitHub**: https://github.com/Gnomone/openseal
- **HighStation**: Seal된 API 통합 플랫폼
- **Discord**: 준비 중

---

## 라이선스

MIT License - [LICENSE](./LICENSE) 참고

---

HighStation의 Trusted AI Infrastructure의 일환으로 ❤️를 담아 제작
