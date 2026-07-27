const { contextBridge, ipcRenderer, webUtils } = require('electron');

contextBridge.exposeInMainWorld('imageCropper', {
  openDialog: () => ipcRenderer.invoke('image:open-dialog'),
  openPath: (filePath) => ipcRenderer.invoke('image:open-path', filePath),
  pathForFile: (file) => webUtils.getPathForFile(file),
  saveCrop: (payload) => ipcRenderer.invoke('image:save-crop', payload),
});
