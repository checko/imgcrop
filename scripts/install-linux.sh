#!/usr/bin/env bash
# Build (if needed) and install Image Cropper for the current Linux user.
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
BINARY="$PROJECT_DIR/target/release/imgcrop"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APP_DIR="$DATA_HOME/imgcrop"
RUNTIME_LIB_DIR="$APP_DIR/lib"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
APPLICATIONS_DIR="$DATA_HOME/applications"
ICONS_DIR="$DATA_HOME/icons/hicolor/scalable/apps"
DESKTOP_FILE="$APPLICATIONS_DIR/com.imgcrop.ImageCropper.desktop"

if [[ ! -x "$BINARY" ]]; then
  command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo is required to build Image Cropper." >&2; exit 1; }
  (cd "$PROJECT_DIR" && cargo build --release)
fi

RUNTIME_PLUGIN_DIR="$APP_DIR/libheif-plugins"
mkdir -p "$APP_DIR" "$RUNTIME_LIB_DIR" "$RUNTIME_PLUGIN_DIR" "$BIN_DIR" "$APPLICATIONS_DIR" "$ICONS_DIR"
install -m 755 "$BINARY" "$APP_DIR/imgcrop"
install -m 644 "$PROJECT_DIR/assets/imgcrop.svg" "$ICONS_DIR/imgcrop.svg"

copy_shared_dependencies() {
  local object="$1"
  while IFS= read -r library; do
    [[ -n "$library" && -f "$library" ]] || continue
    install -m 755 "$library" "$RUNTIME_LIB_DIR/$(basename "$library")"
  done < <(ldd "$object" | awk '/=> \/.*lib(heif|de265|x265|aom|dav1d|jpeg|openjp2|sharpyuv|numa)\.so/ { print $3 }')
}

# Bundle libheif and its decoder plugin. The launcher sets LIBHEIF_PLUGIN_PATH
# so HEIC decoding does not depend on a system-wide ImageMagick or libheif install.
copy_shared_dependencies "$BINARY"
HEIF_LIBRARY="$(ldd "$BINARY" | awk '/libheif\.so/ { print $3; exit }')"
if [[ -z "$HEIF_LIBRARY" || ! -f "$HEIF_LIBRARY" ]]; then
  echo "The release binary could not locate libheif; HEIC support cannot be bundled." >&2
  exit 1
fi
HEIF_PLUGIN_SOURCE="$(dirname "$(readlink -f "$HEIF_LIBRARY")")/libheif/plugins"
if [[ -d "$HEIF_PLUGIN_SOURCE" ]]; then
  while IFS= read -r plugin; do
    install -m 755 "$plugin" "$RUNTIME_PLUGIN_DIR/$(basename "$plugin")"
    copy_shared_dependencies "$plugin"
  done < <(find "$HEIF_PLUGIN_SOURCE" -maxdepth 1 -type f -name 'libheif-*.so' -print)
else
  echo "No libheif plugin directory was found at $HEIF_PLUGIN_SOURCE." >&2
  exit 1
fi

cat > "$BIN_DIR/imgcrop" <<EOF
#!/usr/bin/env bash
export LD_LIBRARY_PATH="$RUNTIME_LIB_DIR\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
export LIBHEIF_PLUGIN_PATH="$RUNTIME_PLUGIN_DIR"
exec "$APP_DIR/imgcrop" "\$@"
EOF
chmod 755 "$BIN_DIR/imgcrop"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Image Cropper
Comment=Open, crop, and save images
Exec=$BIN_DIR/imgcrop %F
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
  gtk-update-icon-cache -f "$DATA_HOME/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "Installed Image Cropper. Find it in the application launcher or run: imgcrop"
