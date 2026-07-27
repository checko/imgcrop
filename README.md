# Image Cropper

A compact desktop image cropper written in Rust. It uses a native desktop window—without Node.js, npm, Electron, Chromium, or ImageMagick at runtime—and performs crop operations directly on the decoded pixels in memory.

![Image Cropper icon](assets/imgcrop.svg)

## Features

- Native Linux and Windows desktop application.
- Native file picker and drag-and-drop opening.
- Rectangular crop selection: draw a new region, drag it to move it, or adjust any of its four corner handles.
- Direct in-process crop algorithm—no external `convert` command.
- Numbered output names: `imagecrop1.ext`, `imagecrop2.ext`, and so on.
- JPEG, PNG, GIF, WebP, BMP, TIFF, ICO, and AVIF support through Rust crates.
- HEIC/HEIF support through the `libheif` codec runtime. Linux and Windows installer scripts bundle the runtime libraries alongside the compiled application. HEIC and HEIF crops save as JPEG for broad compatibility.

## Build and run

### Linux

Requirements: a Rust toolchain, a C/C++ compiler, `pkg-config`, the `libheif` development package, and the normal graphics development packages required by the window system.

```bash
cargo run
```

Build an optimized binary:

```bash
cargo build --release
```

### Windows

Requirements: the Rust MSVC toolchain, Visual Studio Build Tools with C++ support, CMake, and [vcpkg](https://github.com/microsoft/vcpkg).

Install the HEIC codec dependency once in vcpkg:

```powershell
vcpkg install libheif:x64-windows
$env:VCPKG_ROOT = 'C:\path\to\vcpkg'
$env:VCPKGRS_DYNAMIC = '1'
cargo run --release
```

Build an optimized executable:

```powershell
cargo build --release
```

## Install

The install scripts build the release binary if needed, then install it for the current user. End users run the compiled binary and bundled runtime files—they do not need Node.js or npm.

### Linux application panel

```bash
./scripts/install-linux.sh
```

This installs the executable under `~/.local/share/imgcrop`, adds an `imgcrop` launcher in `~/.local/bin`, and creates a desktop entry in `~/.local/share/applications`.

To remove it:

```bash
./scripts/uninstall-linux.sh
```

### Windows Start menu

From PowerShell:

```powershell
.\scripts\install-windows.ps1
```

The script copies `target\release\imgcrop.exe` to `%LOCALAPPDATA%\Programs\Image Cropper`, copies vcpkg runtime DLLs when `VCPKG_ROOT` is configured, and creates an **Image Cropper** Start-menu shortcut.

To remove it:

```powershell
.\scripts\uninstall-windows.ps1
```

## Tests

```bash
cargo test
```

## License

This project is licensed under the [MIT License](LICENSE).
