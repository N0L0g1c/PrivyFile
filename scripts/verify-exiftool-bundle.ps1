# Verifies ExifTool is present and runnable before release builds.
param(
    [string]$BundleRoot = (Join-Path $PSScriptRoot "..\src-tauri\binaries\win")
)

$ErrorActionPreference = "Stop"

$dest = [System.IO.Path]::GetFullPath($BundleRoot)
$exePath = Join-Path $dest "exiftool.exe"
$filesDir = Join-Path $dest "exiftool_files"

if (-not (Test-Path $exePath)) {
    throw "Missing bundled ExifTool executable: $exePath"
}
if (-not (Test-Path $filesDir)) {
    throw "Missing bundled ExifTool support files: $filesDir"
}

$fileCount = (Get-ChildItem $filesDir -Recurse -File | Measure-Object).Count
if ($fileCount -lt 100) {
    throw "ExifTool support bundle looks incomplete ($fileCount files in exiftool_files)"
}

Push-Location $dest
try {
    $version = & .\exiftool.exe -ver
    if ($LASTEXITCODE -ne 0) {
        throw "Bundled ExifTool failed to run"
    }

    # Smoke-test read/write with support files loaded from bundle directory
    $testFile = Join-Path $env:TEMP ("privyfile-exiftool-verify-" + [guid]::NewGuid().ToString("n") + ".jpg")
    $minimalJpeg = [Convert]::FromBase64String("/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wAARCAABAAEDASIAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAUAQEAAAAAAAAAAAAAAAAAAAAA/8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAwDAQACEQMRAD8A0f/Z")
    [System.IO.File]::WriteAllBytes($testFile, $minimalJpeg)
    & .\exiftool.exe "-GPSLatitude=47.6" "-GPSLongitude=-122.3" -overwrite_original $testFile | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "ExifTool failed to write metadata (exiftool_files may be missing or broken)"
    }
    $tagCheck = & .\exiftool.exe -GPSLatitude -s3 $testFile
    if ($LASTEXITCODE -ne 0 -or -not $tagCheck) {
        throw "ExifTool failed to read metadata after write"
    }
}
finally {
    Pop-Location
    if ($testFile -and (Test-Path $testFile)) {
        Remove-Item $testFile -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "Bundled ExifTool OK (version $version)"
