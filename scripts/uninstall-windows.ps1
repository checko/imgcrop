# Remove the current user's Windows Image Cropper installation.
$ErrorActionPreference = 'Stop'

$InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Image Cropper'
$Shortcut = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Image Cropper.lnk'

Remove-Item -Force -ErrorAction SilentlyContinue $Shortcut
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $InstallDir

Write-Host 'Image Cropper was removed from the Start menu.'
