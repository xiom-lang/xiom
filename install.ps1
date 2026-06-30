# AXIOM Installer v0.10.0
# Run: powershell -ExecutionPolicy Bypass -File install.ps1

$ErrorActionPreference = "Stop"
Write-Host "=== AXIOM Installer v0.10.0 ===" -ForegroundColor Magenta

$axiomRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$binDir = "$env:LOCALAPPDATA\axiom\bin"

# Build axiomc in release mode
Write-Host "[1/4] Building axiomc (release)..." -ForegroundColor Cyan
Push-Location $axiomRoot
cargo build -p axiomc --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed" -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location

# Create bin directory
Write-Host "[2/4] Creating $binDir..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

# Copy binary
Write-Host "[3/4] Installing axiomc..." -ForegroundColor Cyan
Copy-Item "$axiomRoot\target\release\axiomc.exe" "$binDir\axiomc.exe" -Force

# Add to PATH
Write-Host "[4/4] Configuring PATH..." -ForegroundColor Cyan
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$binDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$binDir", "User")
    Write-Host "  Added to user PATH. Restart terminal to use 'axiomc'."
} else {
    Write-Host "  Already in PATH."
}

Write-Host ""
Write-Host "AXIOM installed successfully!" -ForegroundColor Green
Write-Host "Run 'axiomc --help' to verify."
Write-Host ""
