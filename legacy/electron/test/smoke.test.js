const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const root = path.resolve(__dirname, '..');

test('application source files have valid JavaScript syntax', () => {
  for (const file of ['main.js', 'preload.js', 'renderer.js']) {
    assert.doesNotThrow(() => execFileSync(process.execPath, ['--check', path.join(root, file)]));
  }
});

test('interface includes opening, dragging, cropping, and saving controls', () => {
  const html = fs.readFileSync(path.join(root, 'index.html'), 'utf8');
  for (const id of ['open-button', 'save-button', 'image-stage', 'crop-box']) {
    assert.match(html, new RegExp(`id="${id}"`));
  }
  const renderer = fs.readFileSync(path.join(root, 'renderer.js'), 'utf8');
  assert.match(renderer, /dataTransfer\.files/);
  assert.match(renderer, /pointerdown/);
  assert.match(renderer, /saveCrop/);
});

test('ImageMagick can create the crop formats required by the application', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'imgcrop-test-'));
  const source = path.join(directory, 'source.png');
  const output = path.join(directory, 'sourcecrop1.png');
  execFileSync('convert', ['-size', '100x80', 'xc:#6558e8', source]);
  execFileSync('convert', [source, '-crop', '40x30+10+15', '+repage', output]);
  const dimensions = execFileSync('identify', ['-format', '%w x %h', output], { encoding: 'utf8' });
  assert.equal(dimensions, '40 x 30');
  fs.rmSync(directory, { recursive: true, force: true });
});
