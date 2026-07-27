# Image Cropper

A Linux desktop image-cropping application built with Electron and ImageMagick. It provides a simple visual workflow for opening an image, drawing a crop selection, refining it with corner handles, and saving numbered crops beside the original file.

![Image Cropper icon](assets/imgcrop.svg)

## Features

- Open images through the native **Open File** dialog.
- Open files by dragging and dropping them onto the window.
- Crop by drawing a rectangle, dragging the selection, or adjusting any of its four corners.
- Save crops as `namecrop1.ext`, `namecrop2.ext`, and so on, without overwriting existing files.
- Open common ImageMagick-supported formats, including HEIC/HEIF, PNG, JPEG, WebP, TIFF, GIF, BMP, AVIF, and SVG.
- Install a per-user application-panel launcher on Linux.

> The local ImageMagick backend used by this project can read HEIC/HEIF. If it cannot encode HEIC/HEIF on the system, crops from those source files are saved as JPEG.

## Requirements

- Linux
- Node.js and npm
- ImageMagick, including the format delegates required for the images you intend to open

## Development

Install dependencies and start the application:

```bash
npm install
npm start
```

Run the smoke tests:

```bash
npm test
```

## Install in the application panel

Install Image Cropper for the current user:

```bash
./scripts/install.sh
```

The installer creates:

- `~/.local/share/applications/com.imgcrop.ImageCropper.desktop`
- `~/.local/share/icons/hicolor/scalable/apps/imgcrop.svg`
- `~/.local/bin/imgcrop`

You can then launch **Image Cropper** from your desktop environment’s application panel.

To remove this user-level launcher integration while preserving the project directory and dependencies:

```bash
./scripts/uninstall.sh
```

## License

This project is licensed under the [MIT License](LICENSE).
