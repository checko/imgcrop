# Build (if needed) and install Image Cropper for the current Windows user.
$ErrorActionPreference = 'Stop'

$ProjectDir = Split-Path -Parent $PSScriptRoot
$Binary = Join-Path $ProjectDir 'target\release\imgcrop.exe'
$InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Image Cropper'
$StartMenu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$Shortcut = Join-Path $StartMenu 'Image Cropper.lnk'

# Match the dynamic x64-windows vcpkg triplet whose DLLs are copied below.
$env:VCPKGRS_DYNAMIC = '1'

if (-not (Test-Path $Binary)) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'Rust/Cargo is required to build Image Cropper.'
    }
    Push-Location $ProjectDir
    try { cargo build --release } finally { Pop-Location }
}

New-Item -ItemType Directory -Force -Path $InstallDir, $StartMenu | Out-Null
Copy-Item -Force $Binary (Join-Path $InstallDir 'imgcrop.exe')

# libheif-rs locates libheif through vcpkg on Windows. Copy all vcpkg runtime DLLs
# so the installed app remains portable after it has been built.
if ($env:VCPKG_ROOT) {
    $RuntimeDir = Join-Path $env:VCPKG_ROOT 'installed\x64-windows\bin'
    if (Test-Path $RuntimeDir) {
        Copy-Item -Force (Join-Path $RuntimeDir '*.dll') $InstallDir
    }
}

$WshShell = New-Object -ComObject WScript.Shell
$Link = $WshShell.CreateShortcut($Shortcut)
$Link.TargetPath = Join-Path $InstallDir 'imgcrop.exe'
$Link.WorkingDirectory = $InstallDir
$Link.IconLocation = "$($Link.TargetPath),0"
$Link.Description = 'Open, crop, and save images'
$Link.Save()

Write-Host 'Installed Image Cropper. Find it in the Start menu.'
