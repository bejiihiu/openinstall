#!/usr/bin/env sh
# OpenInstall — one-line installer
# Usage: curl -sSf https://raw.githubusercontent.com/bejiihiu/openinstall/main/scripts/install.sh | sh

set -eu

REPO="bejiihiu/openinstall"
VERSION="${1:-latest}"

# Detect arch
case "$(uname -m)" in
  x86_64|amd64)  ARCH="x86_64-unknown-linux-gnu" ;;
  aarch64|arm64) ARCH="aarch64-unknown-linux-gnu" ;;
  *)             echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac

# Resolve latest version if needed
if [ "$VERSION" = "latest" ]; then
  echo "Fetching latest release..."
  VERSION=$(curl -sSfL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name":' \
    | sed 's/.*"tag_name": "\(.*\)",.*/\1/')
  if [ -z "$VERSION" ]; then
    echo "Failed to detect latest version"
    exit 1
  fi
  echo "Latest: $VERSION"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/installer-$ARCH"
DEST="${OPENINSTALL_DEST:-/usr/local/bin/installer}"

echo "Downloading installer-$ARCH..."
if command -v sudo >/dev/null 2>&1 && [ ! -w "$(dirname "$DEST")" ]; then
  curl -sSfL "$DOWNLOAD_URL" -o /tmp/installer
  sudo mv /tmp/installer "$DEST"
  sudo chmod +x "$DEST"
else
  curl -sSfL "$DOWNLOAD_URL" -o "$DEST"
  chmod +x "$DEST"
fi

echo "Installed: $DEST"
echo ""
echo "Usage:  installer detect"
echo "        installer openinstall://app?m=<manifest_url>"
