<#
.SYNOPSIS
    Builds the LimeStone Windows MSI installer and portable release packages.
#>

param(
    [string]$Version = "0.1.0-beta"
)

$ErrorActionPreference = "Stop"

Write-Host "==> [1/4] Building LimeStone release binary..." -ForegroundColor Cyan
# Respect a caller-provided CARGO_TARGET_DIR, otherwise use the workspace default.
if (-not $env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR = Join-Path $PSScriptRoot "..\target"
}
cargo build --release -p adjust-gui
if ($LASTEXITCODE -ne 0) { Write-Error "cargo build failed." }

Write-Host "==> [2/4] Deploying binary and icons..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path "bin" -Force | Out-Null
New-Item -ItemType Directory -Path "dist" -Force | Out-Null

Copy-Item "$env:CARGO_TARGET_DIR\release\adjust-gui.exe" -Destination "bin\Limestone.exe" -Force
Copy-Item "$env:CARGO_TARGET_DIR\release\adjust-gui.exe" -Destination "bin\adjust-gui.exe" -Force

Write-Host "==> [3/4] Building Windows MSI Installer with WiX Toolset..." -ForegroundColor Cyan
$msiOutput = "dist\LimeStone-$Version-x64.msi"
wix build wix\main.wxs -arch x64 -o $msiOutput
if ($LASTEXITCODE -ne 0) { Write-Error "wix build failed." }

if (Test-Path $msiOutput) {
    $sizeMb = [math]::Round((Get-Item $msiOutput).Length / 1MB, 2)
    Write-Host "==> [4/4] Successfully generated $msiOutput ($sizeMb MB)" -ForegroundColor Green
} else {
    Write-Error "Failed to generate MSI package."
}
