const state = {
  image: null,
  crop: null,
  pointerAction: null,
};

const openButton = document.querySelector('#open-button');
const emptyOpenButton = document.querySelector('#empty-open-button');
const replaceButton = document.querySelector('#replace-button');
const saveButton = document.querySelector('#save-button');
const emptyState = document.querySelector('#empty-state');
const editor = document.querySelector('#editor');
const image = document.querySelector('#image');
const canvas = document.querySelector('#canvas-wrap');
const cropBox = document.querySelector('#crop-box');
const fileName = document.querySelector('#file-name');
const imageSize = document.querySelector('#image-size');
const toast = document.querySelector('#toast');

function message(text, isError = false) {
  toast.textContent = text;
  toast.style.background = isError ? '#a52d4d' : '#263043';
  toast.classList.add('show');
  clearTimeout(message.timeout);
  message.timeout = setTimeout(() => toast.classList.remove('show'), 3600);
}

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

function normalizedCrop(crop) {
  const x = Math.min(crop.startX, crop.endX);
  const y = Math.min(crop.startY, crop.endY);
  return { x, y, width: Math.abs(crop.endX - crop.startX), height: Math.abs(crop.endY - crop.startY) };
}

function getCanvasPoint(event) {
  const bounds = canvas.getBoundingClientRect();
  return {
    x: clamp(event.clientX - bounds.left, 0, bounds.width),
    y: clamp(event.clientY - bounds.top, 0, bounds.height),
  };
}

function renderCrop() {
  if (!state.crop || state.crop.width < 3 || state.crop.height < 3) {
    cropBox.classList.add('hidden');
    saveButton.disabled = true;
    return;
  }
  const { x, y, width, height } = state.crop;
  cropBox.classList.remove('hidden');
  cropBox.style.left = `${x}px`;
  cropBox.style.top = `${y}px`;
  cropBox.style.width = `${width}px`;
  cropBox.style.height = `${height}px`;
  saveButton.disabled = false;
}

function setCrop(startX, startY, endX, endY) {
  state.crop = normalizedCrop({ startX, startY, endX, endY });
  renderCrop();
}

async function openImage(filePath) {
  try {
    const loaded = await window.imageCropper.openPath(filePath);
    state.image = loaded;
    state.crop = null;
    image.src = loaded.preview;
    fileName.textContent = loaded.name;
    imageSize.textContent = `${loaded.width} × ${loaded.height} px`;
    emptyState.classList.add('hidden');
    editor.classList.remove('hidden');
  } catch (error) {
    message(error.message || 'Unable to open this image.', true);
  }
}

async function showOpenDialog() {
  try {
    const loaded = await window.imageCropper.openDialog();
    if (!loaded) return;
    state.image = loaded;
    state.crop = null;
    image.src = loaded.preview;
    fileName.textContent = loaded.name;
    imageSize.textContent = `${loaded.width} × ${loaded.height} px`;
    emptyState.classList.add('hidden');
    editor.classList.remove('hidden');
  } catch (error) {
    message(error.message || 'Unable to open this image.', true);
  }
}

function initialCrop() {
  const width = image.clientWidth;
  const height = image.clientHeight;
  const paddingX = width * 0.1;
  const paddingY = height * 0.1;
  setCrop(paddingX, paddingY, width - paddingX, height - paddingY);
}

image.addEventListener('load', initialCrop);
openButton.addEventListener('click', showOpenDialog);
emptyOpenButton.addEventListener('click', showOpenDialog);
replaceButton.addEventListener('click', showOpenDialog);

canvas.addEventListener('pointerdown', (event) => {
  if (!state.image || event.button !== 0) return;
  const point = getCanvasPoint(event);
  const handle = event.target.closest('.handle');
  const insideCrop = event.target === cropBox || event.target.closest('#crop-box');
  canvas.setPointerCapture(event.pointerId);

  if (handle && state.crop) {
    state.pointerAction = { type: 'handle', corner: handle.dataset.corner };
  } else if (insideCrop && state.crop) {
    state.pointerAction = { type: 'move', origin: point, crop: { ...state.crop } };
  } else {
    state.pointerAction = { type: 'draw', start: point };
    setCrop(point.x, point.y, point.x, point.y);
  }
  event.preventDefault();
});

canvas.addEventListener('pointermove', (event) => {
  if (!state.pointerAction) return;
  const point = getCanvasPoint(event);
  const width = image.clientWidth;
  const height = image.clientHeight;
  const action = state.pointerAction;

  if (action.type === 'draw') {
    setCrop(action.start.x, action.start.y, point.x, point.y);
  } else if (action.type === 'move') {
    const dx = point.x - action.origin.x;
    const dy = point.y - action.origin.y;
    const crop = action.crop;
    state.crop = {
      x: clamp(crop.x + dx, 0, width - crop.width),
      y: clamp(crop.y + dy, 0, height - crop.height),
      width: crop.width,
      height: crop.height,
    };
    renderCrop();
  } else if (action.type === 'handle') {
    const crop = state.crop;
    const opposite = {
      nw: { x: crop.x + crop.width, y: crop.y + crop.height },
      ne: { x: crop.x, y: crop.y + crop.height },
      sw: { x: crop.x + crop.width, y: crop.y },
      se: { x: crop.x, y: crop.y },
    }[action.corner];
    setCrop(opposite.x, opposite.y, clamp(point.x, 0, width), clamp(point.y, 0, height));
  }
});

function releasePointer(event) {
  if (!state.pointerAction) return;
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
  state.pointerAction = null;
}
canvas.addEventListener('pointerup', releasePointer);
canvas.addEventListener('pointercancel', releasePointer);

saveButton.addEventListener('click', async () => {
  if (!state.image || !state.crop) return;
  const scaleX = state.image.width / image.clientWidth;
  const scaleY = state.image.height / image.clientHeight;
  const crop = {
    x: state.crop.x * scaleX,
    y: state.crop.y * scaleY,
    width: state.crop.width * scaleX,
    height: state.crop.height * scaleY,
  };
  saveButton.disabled = true;
  saveButton.textContent = 'Saving…';
  try {
    const outputPath = await window.imageCropper.saveCrop({ filePath: state.image.filePath, crop });
    message(`Saved ${outputPath.split(/[/\\]/).pop()}`);
  } catch (error) {
    message(error.message || 'Unable to save the cropped image.', true);
  } finally {
    saveButton.disabled = false;
    saveButton.textContent = 'Save Crop';
  }
});

let dragDepth = 0;
window.addEventListener('dragenter', (event) => {
  event.preventDefault();
  dragDepth += 1;
  document.body.classList.add('is-dropping');
});
window.addEventListener('dragover', (event) => event.preventDefault());
window.addEventListener('dragleave', (event) => {
  event.preventDefault();
  dragDepth -= 1;
  if (dragDepth <= 0) {
    dragDepth = 0;
    document.body.classList.remove('is-dropping');
  }
});
window.addEventListener('drop', (event) => {
  event.preventDefault();
  dragDepth = 0;
  document.body.classList.remove('is-dropping');
  const file = event.dataTransfer.files[0];
  const filePath = file ? window.imageCropper.pathForFile(file) : null;
  if (filePath) openImage(filePath);
  else message('Please drop an image file from your computer.', true);
});
