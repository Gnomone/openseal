#!/bin/bash
set -e

# Configuration
REPO="Gnomone/openseal"
VERSION="v0.2.1" # Hardcoded for now, or fetch latest
BINARY_NAME="openseal-linux"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"

echo "🔐 OpenSeal Installer"
echo "   Target: $DOWNLOAD_URL"

# 1. Download
echo "   ⬇️  Downloading binary..."
curl -L $DOWNLOAD_URL -o openseal

# 2. Verify (Optional checksum in future)

# 3. Install
echo "   ⚙️  Installing to /usr/local/bin..."
chmod +x openseal
if [ -w /usr/local/bin ]; then
    mv openseal /usr/local/bin/openseal
else
    echo "   🔒 Elevating permissions (sudo)..."
    sudo mv openseal /usr/local/bin/openseal
fi

echo "   ✅ Installation Complete!"
echo "   Try running: openseal --help"
