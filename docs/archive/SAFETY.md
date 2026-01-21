# 🛡️ OpenSeal Safety Guardrails

To prevent accidental "Sealing" of unintended directories (like your home or system folders), OpenSeal implements safety guardrails.

## 1. Project Detection
When you run `openseal build`, the CLI automatically checks for the presence of standard project indicators:
- `package.json` (Node.js)
- `Cargo.toml` (Rust)
- `requirements.txt` / `pyproject.toml` (Python)
- `go.mod` (Go)
- `.git` (Version Control)
- `.opensealignore` (Existing OpenSeal config)

## 2. Interactive Warning
If none of these indicators are found, OpenSeal will pause and ask for your confirmation before proceeding:
> `⚠️ WARNING: No standard project files detected. Do you want to proceed? (y/N)`

## 3. Best Practices
- **Run at Root**: Always execute `openseal` commands at the top-level directory of your API service.
- **Explicit Source**: Use `-s` or `--source` if you need to build from a subdirectory, but ensure the target is indeed a valid project.
- **Check `.opensealignore`**: Ensure that temporary build artifacts and sensitive files (.env) are listed in your ignore file.
