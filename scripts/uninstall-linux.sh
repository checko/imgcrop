#!/usr/bin/env bash
# Remove the current user's Linux Image Cropper installation.
set -euo pipefail

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APP_DIR="$DATA_HOME/imgcrop"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
APPLICATIONS_DIR="$DATA_HOME/applications"
ICONS_DIR="$DATA_HOME/icons/hicolor/scalable/apps"

rm -f -- "$BIN_DIR/imgcrop" "$APPLICATIONS_DIR/com.imgcrop.ImageCropper.desktop" "$ICONS_DIR/imgcrop.svg"
rm -rf -- "$APP_DIR"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "$DATA_HOME/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "Image Cropper was removed from this user's application launcher."
