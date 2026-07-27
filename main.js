const { app, BrowserWindow, dialog, ipcMain } = require('electron');
const { execFile } = require('node:child_process');
const { promisify } = require('node:util');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');

const execFileAsync = promisify(execFile);
const convertCommand = process.platform === 'win32' ? 'magick' : 'convert';
const identifyCommand = process.platform === 'win32' ? 'magick' : 'identify';

function createWindow() {
  const window = new BrowserWindow({
    width: 1120,
    height: 760,
    minWidth: 820,
    minHeight: 600,
    backgroundColor: '#f6f7fb',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  window.loadFile('index.html');
}

async function command(args) {
  try {
    return await execFileAsync(convertCommand, args, { maxBuffer: 32 * 1024 * 1024 });
  } catch (error) {
    const detail = error.stderr || error.message;
    throw new Error(`ImageMagick could not process this image: ${detail}`);
  }
}

async function identify(args) {
  try {
    const identifyArgs = process.platform === 'win32' ? ['identify', ...args] : args;
    return await execFileAsync(identifyCommand, identifyArgs, { maxBuffer: 32 * 1024 * 1024 });
  } catch (error) {
    const detail = error.stderr || error.message;
    throw new Error(`ImageMagick could not inspect this image: ${detail}`);
  }
}

async function prepareImage(filePath) {
  await fs.access(filePath);
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'imgcrop-'));
  const previewPath = path.join(tempDir, 'preview.png');
  await command([filePath, '-auto-orient', '-flatten', previewPath]);
  const preview = await fs.readFile(previewPath);
  const { stdout } = await identify(['-format', '%w %h', previewPath]);
  const [width, height] = stdout.trim().split(/\s+/).map(Number);

  if (!width || !height) throw new Error('The image dimensions could not be determined.');

  return {
    filePath,
    name: path.basename(filePath),
    width,
    height,
    preview: `data:image/png;base64,${preview.toString('base64')}`,
  };
}

ipcMain.handle('image:open-dialog', async () => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    title: 'Open an image',
    properties: ['openFile'],
    filters: [
      { name: 'Images', extensions: ['heic', 'heif', 'jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'tif', 'tiff', 'avif', 'svg', 'ico', 'jp2'] },
      { name: 'All files', extensions: ['*'] },
    ],
  });
  return canceled ? null : prepareImage(filePaths[0]);
});

ipcMain.handle('image:open-path', (_event, filePath) => prepareImage(filePath));

async function nextCropPath(directory, base, extension) {
  let number = 1;
  let outputPath;
  do {
    outputPath = path.join(directory, `${base}crop${number}${extension}`);
    number += 1;
  } while (await fs.access(outputPath).then(() => true).catch(() => false));
  return outputPath;
}

ipcMain.handle('image:save-crop', async (_event, { filePath, crop }) => {
  const sourceExtension = path.extname(filePath);
  const base = path.basename(filePath, sourceExtension);
  const directory = path.dirname(filePath);
  // This ImageMagick build can read HEIC/HEIF but cannot encode it, so retain
  // full HEIC input support while producing a widely usable JPEG crop.
  const outputExtension = ['.heic', '.heif'].includes(sourceExtension.toLowerCase()) ? '.jpg' : sourceExtension;
  let outputPath = await nextCropPath(directory, base, outputExtension);

  const width = Math.max(1, Math.round(crop.width));
  const height = Math.max(1, Math.round(crop.height));
  const x = Math.max(0, Math.round(crop.x));
  const y = Math.max(0, Math.round(crop.y));
  const cropArgs = [filePath, '-auto-orient', '-crop', `${width}x${height}+${x}+${y}`, '+repage'];

  try {
    await command([...cropArgs, outputPath]);
  } catch (error) {
    if (outputExtension === '.png') throw error;
    // A selected image type may be readable but not writable by ImageMagick.
    // Fall back to PNG rather than losing the user's crop.
    outputPath = await nextCropPath(directory, base, '.png');
    await command([...cropArgs, outputPath]);
  }
  return outputPath;
});

app.whenReady().then(() => {
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
