# Verifies privyfile-core discovers bundled ExifTool and uses it to clean images.
$ErrorActionPreference = "Stop"

$root = Split-Path $PSScriptRoot -Parent
$bundle = Join-Path $root "src-tauri\binaries\win"
$exe = Join-Path $bundle "exiftool.exe"
$files = Join-Path $bundle "exiftool_files"

if (-not (Test-Path $exe) -or -not (Test-Path $files)) {
    throw "Bundled ExifTool missing. Run: npm run fetch-exiftool"
}

$testFile = Join-Path $env:TEMP ("privyfile-runtime-" + [guid]::NewGuid().ToString("n") + ".jpg")
$outputDir = Join-Path $env:TEMP ("privyfile-runtime-out-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$minimalJpeg = [Convert]::FromBase64String("/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wAARCAABAAEDASIAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAUAQEAAAAAAAAAAAAAAAAAAAAA/8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAwDAQACEQMRAD8A0f/Z")
[System.IO.File]::WriteAllBytes($testFile, $minimalJpeg)

Push-Location $bundle
try {
    & .\exiftool.exe "-GPSLatitude=47.6" "-GPSLongitude=-122.3" -overwrite_original $testFile | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to seed test image with GPS metadata"
    }
}
finally {
    Pop-Location
}

$env:PRIVYFILE_EXIFTOOL_DIR = $bundle

Push-Location $root
try {
    $targetDir = Join-Path $root "target"
    cargo build -q --release -p privyfile-cli --target-dir $targetDir
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to build privyfile-cli"
    }

    $cli = Join-Path $targetDir "release\privyfile-cli.exe"
    if (-not (Test-Path $cli)) {
        throw "CLI binary not found at $cli"
    }

    $before = & $cli metadata $testFile --json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw "CLI metadata failed"
    }
    $hasGps = @($before.tags | Where-Object { $_.category -eq "gps" -or $_.name -match "GPS" }).Count -gt 0
    if (-not $hasGps) {
        throw "Test image has no GPS tags before clean"
    }

    $cleanOut = & $cli clean $testFile --output $outputDir
    if ($LASTEXITCODE -ne 0) {
        throw "CLI clean failed: $cleanOut"
    }

    $cleaned = Get-ChildItem $outputDir -Filter "*-cleaned*.jpg" | Select-Object -First 1
    if (-not $cleaned) {
        throw "Clean did not produce an output file"
    }

    $after = & $cli metadata $cleaned.FullName --json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw "CLI metadata after clean failed"
    }
    $gpsAfter = @($after.tags | Where-Object { $_.category -eq "gps" -or $_.name -match "GPS" }).Count
    if ($gpsAfter -gt 0) {
        throw "GPS tags remain after clean; ExifTool may not have been used"
    }
}
finally {
    Pop-Location
    Remove-Item Env:PRIVYFILE_EXIFTOOL_DIR -ErrorAction SilentlyContinue
    Remove-Item $testFile -Force -ErrorAction SilentlyContinue
    Remove-Item $outputDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "ExifTool runtime clean test passed"
