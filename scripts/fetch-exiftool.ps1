# Downloads and installs ExifTool into src-tauri/binaries/win/ for Windows bundling.
# ExifTool is invoked as an external process (Artistic GPL) — see https://exiftool.org/
param(
    [string]$VersionFile = (Join-Path $PSScriptRoot "exiftool-version.txt"),
    [string]$DestDir = (Join-Path $PSScriptRoot "..\src-tauri\binaries\win")
)

$ErrorActionPreference = "Stop"

$version = (Get-Content $VersionFile -Raw).Trim()
if (-not $version) {
    throw "Missing ExifTool version in $VersionFile"
}

$dest = [System.IO.Path]::GetFullPath($DestDir)
$marker = Join-Path $dest ".exiftool-version"
$exePath = Join-Path $dest "exiftool.exe"
$filesDir = Join-Path $dest "exiftool_files"

if ((Test-Path $marker) -and (Get-Content $marker -Raw).Trim() -eq $version -and (Test-Path $exePath) -and (Test-Path $filesDir)) {
    Write-Host "ExifTool $version already present in $dest"
    exit 0
}

New-Item -ItemType Directory -Force -Path $dest | Out-Null

$zipUrl = "https://sourceforge.net/projects/exiftool/files/exiftool-${version}_64.zip/download"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("privyfile-exiftool-" + [guid]::NewGuid().ToString("n"))
$zipPath = Join-Path $tempRoot "exiftool.zip"

try {
    New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null
    Write-Host "Downloading ExifTool $version from $zipUrl"
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if ($curl) {
        & curl.exe -L -o $zipPath $zipUrl
        if ($LASTEXITCODE -ne 0) {
            throw "curl failed to download ExifTool (exit code $LASTEXITCODE)"
        }
    } else {
        Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing
    }

    $header = Get-Content $zipPath -Encoding Byte -TotalCount 2
    if ($header.Count -lt 2 -or $header[0] -ne 0x50 -or $header[1] -ne 0x4B) {
        throw "Downloaded file is not a valid zip archive (SourceForge mirror may have failed)"
    }

    $extractDir = Join-Path $tempRoot "extract"
    Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

    $sourceExe = Get-ChildItem -Path $extractDir -Recurse -Filter "exiftool(-k).exe" | Select-Object -First 1
    if (-not $sourceExe) {
        $sourceExe = Get-ChildItem -Path $extractDir -Recurse -Filter "exiftool*.exe" | Select-Object -First 1
    }
    if (-not $sourceExe) {
        throw "Could not find exiftool executable in downloaded archive"
    }

    $sourceFiles = Get-ChildItem -Path $extractDir -Recurse -Directory -Filter "exiftool_files" | Select-Object -First 1
    if (-not $sourceFiles) {
        throw "Could not find exiftool_files directory in downloaded archive"
    }

    if (Test-Path $exePath) { Remove-Item $exePath -Force }
    if (Test-Path $filesDir) { Remove-Item $filesDir -Recurse -Force }

    Copy-Item $sourceExe.FullName $exePath
    Copy-Item $sourceFiles.FullName $filesDir -Recurse

    Set-Content -Path $marker -Value $version -NoNewline
    Write-Host "Installed ExifTool $version to $dest"
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Verify the bundled binary runs from its install directory (matches app runtime)
Push-Location $dest
try {
    & .\exiftool.exe -ver | Out-Host
    if ($LASTEXITCODE -ne 0) {
        throw "ExifTool verification failed after install"
    }
    if (-not (Test-Path (Join-Path $dest "exiftool_files"))) {
        throw "exiftool_files directory missing after install"
    }
}
finally {
    Pop-Location
}
