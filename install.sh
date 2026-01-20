#!/bin/bash
set -e

echo "=================================================="
echo "         OpenSeal Installer (v0.1.0)              "
echo "=================================================="

# 1. Download Location
DOWNLOAD_URL="https://github.com/kjyyoung/openseal/releases/latest/download/openseal"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="openseal"

echo "⬇️  Downloading OpenSeal binary..."
curl -L -o "$BINARY_NAME" "$DOWNLOAD_URL"

# 2. Make Executable
chmod +x "$BINARY_NAME"

# 3. Install
echo "📦 Installing to $INSTALL_DIR..."
# Check if we have write access, otherwise use sudo
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
    echo "   (Sudo permission required)"
    sudo mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi

echo ""
echo "✅ Installation Complete!"
echo "--------------------------------------------------"
openseal --version
echo "--------------------------------------------------"
