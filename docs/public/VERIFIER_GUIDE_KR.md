# OpenSeal 사용자 가이드

**대상**: Seal된 API의 결과를 검증하려는 API 소비자

[🇺🇸 English Version](./USER_GUIDE.md)

---

## 알아야 할 것

API 사용자로서, OpenSeal을 직접 실행할 필요는 없습니다. Seal된 서비스의 응답을 검증하는 역할입니다.

**핵심 개념**: 모든 응답에는 다음을 증명하는 암호학적 "Seal"이 포함됩니다:
1. 결과가 주장한 컨테이너에서 왔음
2. 결과가 변조되지 않았음
3. 결과가 신선함 (재생 공격 아님)

---

## 요청 보내기

### Wax 헤더 추가

```bash
curl -H "X-OpenSeal-Wax: 여기에-챌린지-입력" \
  http://sealed-api.com/endpoint
```

**Wax란?**
- 사용자가 제공하는 랜덤 문자열 (챌린지)
- 응답이 신선함을 증명 (재생 공격 방지)
- 아무거나 가능: `"my-request-123"`, `UUID`, 타임스탬프 등

### 예제

```bash
curl -X POST https://api.example.com/crypto/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: $(uuidgen)" \
  -d '{"symbol":"BTC"}'
```

---

## 응답 이해하기

### 응답 형식

```json
{
  "openseal": {
    "a_hash": "18ddef79a8138634ce4ea0ce9a6e2377...",
    "b_hash": "493911b28d91e0ae8d8bb5a99690c919...",
    "signature": "c327c9ef05b62792b79e7dd1c8ec84b9...",
    "pub_key": "d30c05d163733bae3d24b1c189ca0a8c..."
  },
  "result": {
    "symbol": "BTC",
    "price": "89553.03",
    "currency": "USD",
    "timestamp": "2026-01-22T05:30:00Z"
  }
}
```

### Seal 구성 요소

| 필드 | 의미 | 검증 |
|------|------|------|
| `a_hash` | 신원 커밋먼트 | 결과를 특정 컨테이너에 바인딩 |
| `b_hash` | 결과 바인딩 | A-hash를 결과에 암호학적으로 연결 |
| `signature` | Ed25519 서명 | 진위성의 수학적 증명 |
| `pub_key` | 임시 공개 키 | 서명 검증에 사용 |

---

## 응답 검증하기

### 옵션 1: Seal 신뢰하기 (간단)

OpenSeal의 암호학은 다음을 보장합니다:
- ✅ 서명이 유효하면, 결과는 진짜
- ✅ 변조 시 서명이 깨짐
- ✅ Wax로 재생 공격 방지

**유효한 Seal이 있는 모든 응답을 신뢰할 수 있습니다.**

### 옵션 2: CLI 사용 (권장)

v1.0.0-alpha.2 버전부터 `openseal` CLI로 검증할 수 있습니다.

1. **응답 저장**:
    ```bash
    curl -H "X-OpenSeal-Wax: test1234" \
      http://api.example.com/endpoint > response.json
    ```

2. **검증 실행**:
    ```bash
    openseal verify --response response.json --wax test1234
    ```

3. **출력**:
    ```
    🔍 Verifying seal...
       🔑 Public Key: 4fdab7f...
       🆔 A-hash:    086a49...
       ✅ Signature Verified!
    ```

4. **선택사항: 신원(Root Hash) 검증**
    결과가 신뢰하는 특정 Docker 이미지에서 왔는지 확인합니다:
    ```bash
    openseal verify \
      --response response.json \
      --wax test1234 \
      --root-hash "sha256:abc123..."
    ```

---

## 보안 보장

### OpenSeal이 증명하는 것

✅ **결과 무결성**
   - 결과가 Seal된 컨테이너에서 왔음
   - 전송 중 변조 없음

✅ **신원 바인딩**
   - A-hash가 결과를 특정 Docker 이미지에 연결
   - 어떤 코드가 결과를 생성했는지 정확히 알 수 있음

✅ **신선도**
   - 사용자의 Wax가 Seal에 포함됨
   - 재생 공격 방지

### OpenSeal이 증명하지 않는 것

❌ **정확성**
   - Seal은 "코드 X가 결과 Y를 생성함"을 증명
   - "결과 Y가 정확함"을 증명하지 않음
   - 예: 버그 있는 코드도 검증 가능하게 잘못된 결과 생성

❌ **데이터 소스 진실성**
   - API가 외부 데이터를 가져오는 경우 (예: Coinbase 가격)
   - Seal은 "컨테이너가 이것을 가져와서 반환함"을 증명
   - 외부 데이터 소스가 정직함을 증명하지 않음

❌ **컨테이너 내용**
   - Seal은 "다이제스트 ABC인 컨테이너에서 결과"를 증명
   - 컨테이너 내부에 무엇이 있는지 증명하지 않음
   - 제공자가 악의적 코드를 Seal했을 수 있음

**결론**: OpenSeal은 **검증 가능한 실행**을 증명하지, **정확성**을 증명하지 않습니다.

---

## 사용 사례

### 1. API 마켓플레이스 (HighStation)

```javascript
// MCP 도구
async function getCryptoPrice(symbol) {
  const wax = `request-${Date.now()}`;
  const response = await fetch('https://oracle.highstation.net/price', {
    headers: {
      'Content-Type': 'application/json',
      'X-OpenSeal-Wax': wax
    },
    body: JSON.stringify({ symbol })
  });
  
  const data = await response.json();
  
  // Seal은 OpenSeal에 의해 자동 검증됨
  return data.result.price;
}
```

### 2. 감사 추적

```bash
# 모든 요청은 특정 컨테이너 버전에 증명 가능하게 연결됨
curl -H "X-OpenSeal-Wax: audit-2026-01-22-001" \
  https://api.example.com/transaction

# 응답에 암호학적 증명 포함
# → 컴플라이언스/감사에 사용 가능
```

### 3. 무신뢰 통합

```python
# API 제공자를 신뢰하지 않나요?
# OpenSeal로 모든 응답을 검증하세요

import requests

def query_sealed_api(endpoint, wax):
    response = requests.get(endpoint, headers={
        "X-OpenSeal-Wax": wax
    })
    data = response.json()
    
    # Seal이 증명:
    # 1. 특정 컨테이너에서 결과
    # 2. 변조되지 않음
    # 3. 신선함 (사용자 wax와 일치)
    
    return data["result"]
```

---

## 모범 사례

### 1. 항상 고유한 Wax 사용

```bash
# ❌ 나쁨: Wax 재사용
curl -H "X-OpenSeal-Wax: static-value" ...

# ✅ 좋음: 요청마다 고유
curl -H "X-OpenSeal-Wax: $(uuidgen)" ...
```

### 2. 감사용 Seal 저장

```bash
# Seal 포함 전체 응답 저장
curl -H "X-OpenSeal-Wax: audit-$(date +%s)" \
  https://api.example.com/critical-operation > audit-log.json
```

### 3. 제공자의 Root Hash 확인

```bash
# 제공자는 자신의 Image Digest를 공개해야 함
# 예: "우리 BTC Oracle: sha256:abc123..."

# 응답의 a_hash와 비교
# 주장한 서비스를 조회하고 있는지 확인
```

---

## 문제 해결

### 응답에 Seal 없음

**문제**: 응답에 `openseal` 필드 없음

**원인**:
1. API가 OpenSeal을 통해 실행되지 않음
2. 프록시를 우회한 컨테이너 직접 접근

**해결**: 제공자에게 문의

### 서명 무효

**문제**: "Signature verification failed"

**원인**:
1. 응답이 변조됨
2. 네트워크 손상

**해결**: 
- 요청 재시도
- 지속되면 제공자에게 문의
- **결과를 신뢰하지 마세요**

### Wax 불일치

**문제**: 사용자 Wax가 Seal에 나타나지 않음

**원인**:
1. `X-OpenSeal-Wax` 헤더 전송 안 함
2. 프록시가 헤더 제거

**해결**: 헤더가 올바르게 전송되는지 확인

---

## FAQ

**Q: Seal된 API를 사용하려면 OpenSeal을 설치해야 하나요?**  
A: 아니요! `X-OpenSeal-Wax` 헤더만 보내면 됩니다. 제공자가 OpenSeal을 실행합니다.

**Q: Seal을 프로그래밍 방식으로 검증할 수 있나요?**  
A: 네, `openseal verify` 사용 (베타 예정). 또는 Ed25519 검증을 직접 구현하세요.

**Q: Wax를 보내지 않으면?**  
A: 대부분의 Seal된 API는 에러를 반환합니다. Wax는 보안상 필수입니다.

**Q: 제공자가 어떤 컨테이너를 실행하는지 어떻게 아나요?**  
A: 제공자가 Image Digest (Root Hash)를 공개해야 합니다. 문서를 확인하세요.

**Q: Seal된 API가 거짓말할 수 있나요?**  
A: Seal은 "컨테이너 X가 Y를 생성함"을 증명합니다. 컨테이너 X에 악의적 코드가 있으면 Seal은 여전히 유효합니다. 제공자가 신뢰할 수 있는 코드를 실행하는지 항상 확인하세요.

---

## 다음 단계

- **[배포자 가이드](./PROVIDER_GUIDE_KR.md)**: Seal된 서비스 배포 방법
- **[Crypto Oracle 예제](https://github.com/Gnomone/crypto-price-oracle)**: 라이브 데모
- **[HighStation](https://www.highstation.net)**: Seal된 API 마켓플레이스

---

**질문이 있나요?** [GitHub](https://github.com/Gnomone/openseal/issues)에 이슈 열기
