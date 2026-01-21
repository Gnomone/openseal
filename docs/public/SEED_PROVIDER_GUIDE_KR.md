# OpenSeal 배포자 가이드

**대상**: Seal된 API를 배포하려는 서비스 제공자

[🇺🇸 English Version](./PROVIDER_GUIDE.md)

---

## 개요

제공자로서, 기존 Docker 컨테이너를 OpenSeal로 래핑하여 모든 응답에 암호학적 증명을 추가할 수 있습니다.

**얻을 수 있는 것**:
- ✅ 모든 API 응답에 암호학적 Seal 포함
- ✅ 사용자가 결과가 컨테이너에서 왔음을 검증 가능
- ✅ 변조 감지: 수정 시 Seal 파괴
- ✅ 부인 방지: 출처의 수학적 증명

---

## 사전 요구사항

1. **Docker** 설치됨
2. **API**가 이미 컨테이너화됨
3. **OpenSeal CLI** 설치됨

### OpenSeal 설치

```bash
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash

# 또는 소스에서 빌드 (v1-dev 브랜치)
git clone https://github.com/Gnomone/openseal.git -b v1-dev
cd openseal
cargo build --release
cp target/release/openseal /usr/local/bin/
```

---

## 빠른 시작

### 1단계: API Dockerize (아직 안 했다면)

```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY . .
ENV PORT=3000
EXPOSE 3000
CMD ["npm", "start"]
```

```bash
docker build -t my-api:v1 .
```

### 2단계: 이미지 Seal

```bash
openseal build --image my-api:v1
```

`openseal.json` 생성:
```json
{
  "version": "1.0.0",
  "identity": {
    "root_hash": "sha256:7dabd9a9dd2d...",
    "seal_version": "2.0"
  },
  "image": {
    "reference": "my-api:v1",
    "digest": "sha256:7dabd9a9dd2d...",
    "created_at": "2026-01-22T05:30:00Z"
  }
}
```

### 3단계: OpenSeal Proxy와 함께 실행

```bash
openseal run --image my-api:v1 --port 8080
```

출력:
```
🐳 Starting container...
✅ Container ready
🔐 Starting OpenSeal Proxy on port 8080...
📡 Proxy Server Ready!
   Public: http://0.0.0.0:8080
   🔑 Public Key (Ephemeral): d30c05d163733bae...
```

### 4단계: 테스트

```bash
curl -H "X-OpenSeal-Wax: test123" http://localhost:8080/your-endpoint
```

Seal 포함 응답:
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

✅ **완료!** API가 Seal되었습니다.

---

## 배포 시나리오

### 시나리오 1: 로컬 개발

```bash
# 소스에서 빌드
cd my-api
docker build -t my-api:dev .
openseal build --image my-api:dev
openseal run --image my-api:dev --port 8080
```

### 시나리오 2: 프로덕션 서버

```bash
# 레지스트리에서 pull
docker pull ghcr.io/yourorg/my-api:v1.0.0
openseal build --image ghcr.io/yourorg/my-api:v1.0.0
openseal run --image ghcr.io/yourorg/my-api:v1.0.0 --port 8080
```

### 시나리오 3: Systemd 서비스

`/etc/systemd/system/my-sealed-api.service` 생성:

```ini
[Unit]
Description=My Sealed API (OpenSeal)
After=docker.service
Requires=docker.service

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/my-api
ExecStart=/usr/local/bin/openseal run --image my-api:v1 --port 8080
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable my-sealed-api
sudo systemctl start my-sealed-api
```

---

## 보안 모범 사례

### 1. 레지스트리 Digest 사용 (프로덕션)

```bash
# ❌ 개발 전용
openseal build --image my-api:latest

# ✅ 프로덕션
docker push my-org/my-api:v1.0.0
openseal build --image my-org/my-api@sha256:abc123...
```

### 2. Root Hash 공개

문서에 추가:

```markdown
## OpenSeal 검증

우리 API는 Seal된 컨테이너에서 실행됩니다:

**Image Digest (Root Hash)**:
```
sha256:7dabd9a9dd2de714b343b381ee81bbcb3c2bb55b85fecefc3127fd9eafa486b2
```

모든 응답에는 이 다이제스트에 연결된 암호학적 증명이 포함됩니다.
```

### 3. 서명 키 로테이션

OpenSeal은 세션마다 임시 키를 생성합니다. 주기적으로 재시작:

```bash
# 키 로테이션을 위해 매일 재시작
0 0 * * * systemctl restart my-sealed-api
```

---

## 예제: Crypto Price Oracle

전체 작동 예제: https://github.com/Gnomone/crypto-price-oracle

```bash
# Clone
git clone https://github.com/Gnomone/crypto-price-oracle.git
cd crypto-price-oracle

# Build
docker build -t crypto-oracle:v1 .

# Seal
openseal build --image crypto-oracle:v1

# Run
openseal run --image crypto-oracle:v1 --port 8080

# Test
curl -X POST http://localhost:8080/api/v1/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: test123" \
  -d '{"symbol":"BTC"}'
```

---

## 문제 해결

### 컨테이너가 시작되지 않음

```bash
# Docker 로그 확인
docker ps -a | grep openseal
docker logs <container-id>
```

**일반적인 문제**:
- 포트가 이미 사용 중
- 이미지를 찾을 수 없음
- 권한 부족

### Digest 불일치

```bash
# openseal.json 재생성
openseal build --image my-api:v1

# 검증
cat openseal.json
docker inspect my-api:v1 --format='{{.Id}}'
```

---

## 성능 고려사항

### Cold Start

- **컨테이너 시작**: ~3-5초
- **Health Check**: ~2-3초
- **총**: ~5-8초

**완화**: Daemon Mode 사용 (v1에서 이미 기본값)

### 런타임 오버헤드

- **Seal 생성**: 요청당 ~5ms
- **TLS 핸드셰이크와 비슷**

---

## v0에서 마이그레이션

v0.2.x는 v1.0.x와 호환되지 않습니다. 마이그레이션:

1. **애플리케이션 Dockerize**
2. v0 `dist/` 번들 제거
3. 위 빠른 시작 따라하기

**v0 보존**: 레포지토리에 `v0.2.63-final`로 태그됨.

---

## FAQ

**Q: 기존 Docker 이미지에 OpenSeal을 사용할 수 있나요?**  
A: 네! 코드 변경 없이 Seal하고 실행하면 됩니다.

**Q: Docker를 사용하지 않으면?**  
A: 먼저 Dockerfile을 만드세요. OpenSeal v1은 Docker가 필요합니다.

**Q: API 코드를 수정해야 하나요?**  
A: 아니요! OpenSeal은 기존 API를 투명하게 래핑합니다.

**Q: 사용자가 Seal을 어떻게 검증하나요?**  
A: `X-OpenSeal-Wax` 헤더를 보냅니다. OpenSeal이 나머지를 처리합니다.

**Q: 여러 Seal된 서비스를 실행할 수 있나요?**  
A: 네! 각각에 다른 포트를 사용하세요.

---

## 다음 단계

- **[사용자 가이드](./USER_GUIDE_KR.md)**: 사용자가 Seal된 API와 상호작용하는 방법
- **[예제 서비스](https://github.com/Gnomone/crypto-price-oracle)**: 전체 참조 구현
- **[HighStation](https://www.highstation.net)**: 서비스 등록 (직접 진행)

---

**도움이 필요하신가요?** [GitHub](https://github.com/Gnomone/openseal/issues)에 이슈 열기
