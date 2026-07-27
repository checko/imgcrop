#!/usr/bin/env bash
# Install Image Cropper for the current Linux desktop user.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
APP_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
APP_ID="com.imgcrop.ImageCropper"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
APPLICATIONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"
LAUNCHER="$BIN_DIR/imgcrop"
DESKTOP_FILE="$APPLICATIONS_DIR/$APP_ID.desktop"
ICON_FILE="$ICONS_DIR/imgcrop.svg"

if [[ ! -x "$APP_DIR/node_modules/.bin/electron" ]]; then
  if ! command -v npm >/dev/null 2>&1; then
    echo "Electron is not installed and npm was not found. Install Node.js/npm, then rerun this script." >&2
    exit 1
  fi
  echo "Installing application dependencies..."
  (cd "$APP_DIR" && npm install --no-audit --no-fund)
fi

mkdir -p "$BIN_DIR" "$APPLICATIONS_DIR" "$ICONS_DIR"

cat > "$LAUNCHER" <<EOF
#!/usr/bin/env bash
exec "$APP_DIR/node_modules/.bin/electron" "$APP_DIR" "\$@"
EOF
chmod 755 "$LAUNCHER"

install -m 644 "$APP_DIR/assets/imgcrop.svg" "$ICON_FILE"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Image Cropper
Comment=Open, crop, and save images
Exec=$LAUNCHER
Icon=imgcrop
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
EOF
chmod 644 "$DESKTOP_FILE"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "Image Cropper has been installed. Find ‘Image Cropper’ in your application launcher."
echo "Desktop entry: $DESKTOP_FILE"
