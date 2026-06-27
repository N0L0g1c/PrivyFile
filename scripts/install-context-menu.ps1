# Installs Windows Explorer context menu entries for PrivyFile.
param(
    [string]$AppPath = (Join-Path $PSScriptRoot "..\src-tauri\target\release\privyfile.exe")
)

$shellKey = "HKCU:\Software\Classes\*\shell\PrivyFileClean"
New-Item -Path $shellKey -Force | Out-Null
Set-ItemProperty -Path $shellKey -Name "(Default)" -Value "Clean metadata with PrivyFile"
Set-ItemProperty -Path $shellKey -Name "Icon" -Value $AppPath
New-Item -Path "$shellKey\command" -Force | Out-Null
Set-ItemProperty -Path "$shellKey\command" -Name "(Default)" -Value "`"$AppPath`" clean `"%1`""

$shredKey = "HKCU:\Software\Classes\*\shell\PrivyFileShred"
New-Item -Path $shredKey -Force | Out-Null
Set-ItemProperty -Path $shredKey -Name "(Default)" -Value "Shred with PrivyFile"
New-Item -Path "$shredKey\command" -Force | Out-Null
Set-ItemProperty -Path "$shredKey\command" -Name "(Default)" -Value "`"$AppPath`" shred `"%1`""

Write-Host "PrivyFile context menu installed for current user."
