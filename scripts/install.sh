#!/usr/bin/env sh
# OpenInstall — one-line installer
# Usage: curl -sSf https://raw.githubusercontent.com/bejiihiu/openinstall/main/scripts/install.sh | sh

set -eu

REPO="bejiihiu/openinstall"
VERSION="${1:-latest}"

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  RESET='\033[0m'
else
  GREEN='' YELLOW='' CYAN='' BOLD='' RESET=''
fi

info()  { printf "${CYAN}▸${RESET} %s\n" "$*"; }
ok()    { printf "${GREEN}✓${RESET} %s\n" "$*"; }
warn()  { printf "${YELLOW}⚠${RESET} %s\n" "$*"; }
die()   { printf "${YELLOW}✗${RESET} %s\n" "$*" >&2; exit 1; }

# Detect arch
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)  ARCH="x86_64-unknown-linux-gnu" ;;
  aarch64|arm64) ARCH="aarch64-unknown-linux-gnu" ;;
  *)             die "Unsupported architecture: $ARCH" ;;
esac

info "Detected: $ARCH"

# Resolve latest version if needed
if [ "$VERSION" = "latest" ]; then
  info "Fetching latest release..."
  VERSION=$(curl -sSfL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name":' \
    | sed 's/.*"tag_name": "\(.*\)",.*/\1/')
  if [ -z "$VERSION" ]; then
    die "Failed to detect latest version. Check your network connection."
  fi
fi

ok "Version: $VERSION"

# Download
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/installer-$ARCH"
DEST="${OPENINSTALL_DEST:-/usr/local/bin/installer}"

info "Downloading installer..."
TMPFILE=$(mktemp /tmp/openinstall.XXXXXX)
trap 'rm -f "$TMPFILE"' EXIT

if command -v sudo >/dev/null 2>&1 && [ ! -w "$(dirname "$DEST")" ]; then
  curl -sSfL "$DOWNLOAD_URL" -o "$TMPFILE" || die "Download failed: $DOWNLOAD_URL"
  sudo mv "$TMPFILE" "$DEST"
  sudo chmod +x "$DEST"
else
  curl -sSfL "$DOWNLOAD_URL" -o "$TMPFILE" || die "Download failed: $DOWNLOAD_URL"
  mv "$TMPFILE" "$DEST"
  chmod +x "$DEST"
fi

ok "Installed: $DEST"

# Auto-register desktop entry + URI handlers (best effort, GUI builds only)
info "Registering desktop entry..."
if "$DEST" gui --register-desktop 2>/dev/null; then
  ok "Desktop entry registered — OpenInstall now appears in your app menu"
  ok "URI schemes registered: openinstall://, openinstaller://, linuxinstall://"
else
  warn "GUI not available in this build (aarch64). CLI-only — desktop entry skipped."
  info "You can still use: installer install <manifest>"
fi

# Done
printf "\n"
printf "${BOLD}OpenInstall is ready.${RESET}\n"
printf "\n"
printf "  ${CYAN}installer detect${RESET}              — show your system info\n"
printf "  ${CYAN}installer install <manifest>${RESET}  — install an app from a manifest\n"
printf "  ${CYAN}installer gui${RESET}                 — open the graphical installer\n"
printf "\n"
printf "Try it: ${CYAN}installer detect${RESET}\n"
