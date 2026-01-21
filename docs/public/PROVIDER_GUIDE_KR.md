# OpenSeal 제공자 가이드

**대상**: Seal된 API를 운영하는 서버 운영자 (개발자 아님)

[🇺🇸 English Version](./PROVIDER_GUIDE.md)

---

## 개요

**일반 제공자(Normal Provider)**로서, 당신은:
- ✅ GitHub/Registry에서 미리 빌드된 Docker 이미지 pull
- ✅ OpenSeal 실행하여 서비스 Seal
- ✅ Seal된 API 서버 운영

당신이 할 필요 **없는** 것:
- ❌ 코드 작성
- ❌ Docker 이미지 빌드
- ❌ API 내부 이해

---

## 사전 요구사항

1. **Docker** 설치됨
2. **OpenSeal CLI** 설치됨

### OpenSeal 설치

```bash
curl -L https://github.com/Gnomone/openseal/releases/latest/download/install.sh | bash
```

---

## 빠른 시작: Crypto Price Oracle 실행

### 1단계: Docker 이미지 Pull

```bash
# GitHub Container Registry에서 (Actions 빌드 후)
docker pull ghcr.io/gnomone/crypto-price-oracle:latest

# 또는 소스에서 빌드
git clone https://github.com/Gnomone/crypto-price-oracle.git
cd crypto-price-oracle
docker build -t crypto-price-oracle:v1 .
```

### 2단계: 이미지 Seal

```bash
openseal build --image ghcr.io/gnomone/crypto-price-oracle:latest
```

`openseal.json` 생성:
```json
{
  "version": "1.0.0",
  "identity": {
    "root_hash": "sha256:...",
    "seal_version": "2.0"
  }
}
```

### 3단계: Seal된 서비스 실행

```bash
openseal run --image ghcr.io/gnomone/crypto-price-oracle:latest --port 8080
```

출력:
```
🐳 Starting container...
✅ Container ready
🔐 Starting OpenSeal Proxy on port 8080...
📡 Proxy Server Ready!
   Public: http://0.0.0.0:8080
```

### 4단계: 테스트

```bash
curl -X POST http://localhost:8080/api/v1/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: test123" \
  -d '{"symbol":"BTC"}'
```

✅ **완료!** Seal된 API가 실행 중입니다.

---

## 프로덕션 배포

### 옵션 1: Systemd 서비스

`/etc/systemd/system/crypto-oracle.service` 생성:

```ini
[Unit]
Description=Crypto Price Oracle (OpenSeal)
After=docker.service
Requires=docker.service

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/crypto-oracle
ExecStart=/usr/local/bin/openseal run --image ghcr.io/gnomone/crypto-price-oracle:latest --port 8080
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

활성화 및 시작:
```bash
sudo systemctl enable crypto-oracle
sudo systemctl start crypto-oracle
sudo systemctl status crypto-oracle
```

### 옵션 2: Docker Compose

```yaml
version: '3.8'
services:
  oracle:
    image: ghcr.io/gnomone/crypto-price-oracle:latest
    ports:
      - "8080:8080"
    restart: unless-stopped
```

---

## 모니터링

### 로그 보기

```bash
# Systemd
sudo journalctl -u crypto-oracle -f

# Docker
docker logs -f <container-id>
```

### Health Check

```bash
curl http://localhost:8080/health
```

---

## 서비스 업데이트

```bash
# 1. 새 이미지 pull
docker pull ghcr.io/gnomone/crypto-price-oracle:latest

# 2. 기존 서비스 중지
sudo systemctl stop crypto-oracle

# 3. Seal 재생성
openseal build --image ghcr.io/gnomone/crypto-price-oracle:latest

# 4. 재시작
sudo systemctl start crypto-oracle
```

---

## 문제 해결

### 포트 이미 사용 중

```bash
sudo lsof -i :8080
sudo kill -9 <PID>
```

### 컨테이너 시작 안 됨

```bash
docker ps -a
docker logs <container-id>
```

### 이미지를 찾을 수 없음

```bash
# 이미지 존재 확인
docker images | grep crypto-price-oracle

# 재 pull
docker pull ghcr.io/gnomone/crypto-price-oracle:latest
```

---

## 다음 단계

- **[시드 제공자 가이드](./SEED_PROVIDER_GUIDE_KR.md)**: 자신만의 Seal된 서비스를 만들고 싶다면
- **[검증자 가이드](./VERIFIER_GUIDE_KR.md)**: 사용자가 Seal을 검증하는 방법

---

**질문이 있나요?** [GitHub](https://github.com/Gnomone/openseal/issues)에 이슈 열기
