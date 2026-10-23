# VoiceForge Release Packaging Script
$ErrorActionPreference = "Stop"

Write-Host "Packaging VoiceForge v1.0.0 for Windows x64..." -ForegroundColor Cyan

$distDir = "target\dist\VoiceForge"
if (Test-Path $distDir) {
    Remove-Item -Path $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir -Force | Out-Null

Copy-Item "target\release\VoiceForge.exe" -Destination $distDir
Copy-Item "README.md" -Destination $distDir
Copy-Item "THIRD_PARTY_LICENSES.md" -Destination $distDir
Copy-Item "CHANGELOG.md" -Destination $distDir

$zipPath = "target\dist\VoiceForge-v1.0.0-windows-x64.zip"
if (Test-Path $zipPath) {
    Remove-Item -Path $zipPath -Force
}

Compress-Archive -Path "$distDir\*" -DestinationPath $zipPath
Write-Host "Portable package created at: $zipPath" -ForegroundColor Green

# Check if ISCC is installed to compile the full installer
$iscc = (Get-Command iscc.exe -ErrorAction SilentlyContinue)
if ($iscc) {
    Write-Host "Compiling Inno Setup installer..." -ForegroundColor Cyan
    & iscc.exe installer.iss
} else {
    Write-Host "Inno Setup (iscc.exe) not in PATH. installer.iss is ready for compilation." -ForegroundColor Yellow
}
