# build-sidecar.ps1
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir

$BinariesDir = Join-Path $ProjectDir "src-tauri\binaries"
New-Item -ItemType Directory -Force -Path $BinariesDir | Out-Null

Write-Host "=> Compiling sidecar binary with bun..."
& "$env:USERPROFILE\.bun\bin\bun.exe" build (Join-Path $ProjectDir "sidecar\server.js") `
  --compile `
  --outfile (Join-Path $BinariesDir "sidecar-server-x86_64-pc-windows-msvc.exe")

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to compile sidecar"
    exit 1
}

Write-Host "=> Sidecar binary built successfully!"
Get-Item (Join-Path $BinariesDir "sidecar-server-x86_64-pc-windows-msvc.exe") | Select-Object Name, Length
