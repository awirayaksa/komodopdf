#Requires -Version 5.1
<#
  Build script for Komodo PDF Windows releases.
  Produces:
    - MSI installer   (Setup)
    - NSIS installer  (Setup)
    - Portable ZIP    (No-install, runs from any folder/USB)
#>
$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$version = "0.1.0"
$productName = "Komodo PDF"
$exeName = "komodo-pdf.exe"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Komodo PDF Windows Release Builder" -ForegroundColor Cyan
Write-Host "Version: $version" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# 1. Build Tauri app
# ---------------------------------------------------------------------------
Write-Host "`n[1/4] Building Tauri app..." -ForegroundColor Yellow
Set-Location $projectRoot
npm run tauri build
if ($LASTEXITCODE -ne 0) { throw "Tauri build failed" }

# ---------------------------------------------------------------------------
# 2. Prepare release directory
# ---------------------------------------------------------------------------
Write-Host "`n[2/4] Preparing release directory..." -ForegroundColor Yellow
$releaseDir = Join-Path $projectRoot "release"
$portableDir = Join-Path $releaseDir "portable"
if (Test-Path $releaseDir) {
    Remove-Item -Recurse -Force $releaseDir
}
New-Item -ItemType Directory -Path $releaseDir | Out-Null
New-Item -ItemType Directory -Path $portableDir | Out-Null

# ---------------------------------------------------------------------------
# 3. Copy Setup installers (MSI + NSIS)
# ---------------------------------------------------------------------------
Write-Host "`n[3/4] Collecting setup installers..." -ForegroundColor Yellow

$msiDir = [System.IO.Path]::Combine($projectRoot, "src-tauri", "target", "release", "bundle", "msi")
$nsisDir = [System.IO.Path]::Combine($projectRoot, "src-tauri", "target", "release", "bundle", "nsis")

$msiFile = Get-ChildItem -Path $msiDir -Filter "*.msi" | Select-Object -First 1
if (-not $msiFile) { throw "MSI installer not found in $msiDir" }
Copy-Item $msiFile.FullName -Destination (Join-Path $releaseDir "Komodo-PDF-$version-x64.msi")
Write-Host "  -> MSI: $($msiFile.Name)"

$nsisFile = Get-ChildItem -Path $nsisDir -Filter "*.exe" | Select-Object -First 1
if (-not $nsisFile) { throw "NSIS installer not found in $nsisDir" }
Copy-Item $nsisFile.FullName -Destination (Join-Path $releaseDir "Komodo-PDF-$version-x64-Setup.exe")
Write-Host "  -> NSIS: $($nsisFile.Name)"

# ---------------------------------------------------------------------------
# 4. Assemble Portable version
# ---------------------------------------------------------------------------
Write-Host "`n[4/4] Assembling portable version..." -ForegroundColor Yellow

$targetReleaseDir = [System.IO.Path]::Combine($projectRoot, "src-tauri", "target", "release")

# Copy main executable
$srcExe = Join-Path $targetReleaseDir $exeName
$dstExe = Join-Path $portableDir "$productName.exe"
if (-not (Test-Path $srcExe)) { throw "Executable not found: $srcExe" }
Copy-Item $srcExe -Destination $dstExe
Write-Host "  -> Copied executable"

# Copy required DLLs from target/release
$dlls = Get-ChildItem -Path $targetReleaseDir -Filter "*.dll"
foreach ($dll in $dlls) {
    Copy-Item $dll.FullName -Destination $portableDir
    Write-Host "  -> Copied DLL: $($dll.Name)"
}

# Create portable marker file
$markerFile = Join-Path $portableDir "portable"
"# Place this file next to the executable to enable portable mode.`n# All data will be stored in the 'data' folder beside the executable." | Set-Content -Path $markerFile -Encoding UTF8
Write-Host "  -> Created portable marker"

# Create data directory with a README so it survives ZIP archiving
$dataDir = Join-Path $portableDir "data"
New-Item -ItemType Directory -Path $dataDir | Out-Null
"This folder stores Komodo PDF data when running in portable mode." | Set-Content -Path (Join-Path $dataDir "README.txt") -Encoding UTF8
Write-Host "  -> Created data/ directory"

# Zip the portable folder
$zipPath = Join-Path $releaseDir "Komodo-PDF-Portable-$version-x64.zip"
Compress-Archive -Path "$portableDir\*" -DestinationPath $zipPath -Force
Write-Host "  -> Created ZIP: Komodo-PDF-Portable-$version-x64.zip"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "Release artifacts built successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Get-ChildItem -Path $releaseDir -File | ForEach-Object {
    $size = if ($_.Length -gt 1MB) { "{0:N1} MB" -f ($_.Length / 1MB) } else { "{0:N1} KB" -f ($_.Length / 1KB) }
    Write-Host "  - $($_.Name) ($size)" -ForegroundColor White
}
Write-Host "`nDone. Artifacts are in: $releaseDir" -ForegroundColor Green
