# 🛡️ OpenSeal 안전 가드레일 (Safety Guardrails)

OpenSeal은 실수로 홈 디렉토리나 시스템 폴더와 같은 의도치 않은 위치를 "봉인(Seal)"하는 것을 방지하기 위해 안전 가드레일을 제공합니다.

## 1. 프로젝트 자동 탐지
`openseal build` 실행 시, CLI는 해당 위치에 다음과 같은 표준 프로젝트 파일이 있는지 자동으로 확인합니다:
- `package.json` (Node.js)
- `Cargo.toml` (Rust)
- `requirements.txt` / `pyproject.toml` (Python)
- `go.mod` (Go)
- `.git` (버전 관리 시스템)
- `.opensealignore` (기존 OpenSeal 설정)

## 2. 대화형 경고
위의 파일들이 발견되지 않을 경우, OpenSeal은 즉시 중단하고 사용자에게 진행 여부를 묻습니다:
> `⚠️ WARNING: 표준 프로젝트 파일이 탐지되지 않았습니다. 그래도 진행할까요? (y/N)`

## 3. 권장 실습 (Best Practices)
- **루트 실행**: 항상 API 서비스의 최상위(Root) 디렉토리에서 `openseal` 명령어를 실행하세요.
- **소스 경로 명시**: 하위 디렉토리에서 빌드해야 할 경우 `-s` 또는 `--source` 옵션을 사용하여 정확한 위치를 지정하되, 해당 위치가 올바른 프로젝트인지 다시 한번 확인하세요.
- **`.opensealignore` 확인**: 빌드 결과물이나 민감한 파일(.env)이 무시 목록에 포함되어 있는지 확인하세요.
