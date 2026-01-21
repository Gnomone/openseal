# OpenSeal v0.2.4 - Daemon Mode Support

## 🚀 New Features

### Daemon Mode for Production Deployments
OpenSeal can now run as a background daemon, perfect for production servers and long-running deployments.

```bash
# Run in daemon mode (background)
openseal run --app dist_opensealed --port 1999 --daemon

# Or with short flag
openseal run --app dist_opensealed --port 1999 -d

# Custom log file
openseal run --app dist_opensealed --port 1999 --daemon --log-file my-service.log
```

**Benefits:**
- ✅ Survives SSH disconnection
- ✅ Automatic log file redirection
- ✅ No need for screen/tmux/nohup
- ✅ Clean process management

**Monitoring:**
```bash
# View logs in real-time
tail -f openseal.log

# Stop the daemon
pkill -f 'openseal run'
```

## 🔧 Improvements

- **Enhanced Symlink Safety**: Prevents accidental deletion of source dependencies during rebuild
- **Rebuild Support**: Can now safely run `openseal build` multiple times without conflicts
- **Dependency Tracking**: Properly records all ghosted dependencies in manifest

## 📦 Installation

```bash
curl -fsSL https://raw.githubusercontent.com/Gnomone/openseal/main/install.sh | bash
```

## 🐛 Bug Fixes

- Fixed critical symlink handling to prevent data loss
- Fixed manifest recording for multiple dependencies
- Improved process cleanup to prevent zombie processes

---

**Full Changelog**: https://github.com/Gnomone/openseal/compare/v0.2.3...v0.2.4
