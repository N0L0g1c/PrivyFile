# Confirms the generated Windows installer includes bundled ExifTool files.
param(
    [string]$WxsPath = ""
)

$ErrorActionPreference = "Stop"

if (-not $WxsPath) {
    $repoRoot = Join-Path $PSScriptRoot ".."
    foreach ($targetRoot in @("target", "src-tauri\target")) {
        $searchRoot = Join-Path $repoRoot $targetRoot
        $WxsPath = Get-ChildItem -Path $searchRoot -Recurse -Filter "main.wxs" -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '\\wix\\' } |
            Select-Object -First 1 -ExpandProperty FullName
        if ($WxsPath) { break }
    }
}

if (-not $WxsPath -or -not (Test-Path $WxsPath)) {
    Write-Host "WiX file not found under target/ or src-tauri/target/ (skip installer verification)"
    exit 0
}

$content = Get-Content $WxsPath -Raw
if ($content -notmatch 'exiftool\.exe') {
    throw "Installer WiX source does not reference exiftool.exe"
}
if ($content -notmatch 'exiftool_files') {
    throw "Installer WiX source does not reference exiftool_files"
}

Write-Host "Installer bundle includes ExifTool (verified in $WxsPath)"
