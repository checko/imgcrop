#!/usr/bin/env bash
# Remove the current user's Image Cropper launcher integration.
set -euo pipefail

APP_ID="com.imgcrop.ImageCropper"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
APPLICATIONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"
DESKTOP_FILE="$APPLICATIONS_DIR/$APP_ID.desktop"
ICON_FILE="$ICONS_DIR/imgcrop.svg"
LAUNCHER="$BIN_DIR/imgcrop"

rm -f -- "$DESKTOP_FILE" "$ICON_FILE" "$LAUNCHER"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "Image Cropper has been removed from the application launcher."
echo "The project directory and its dependencies were left untouched."
