#Requires -Version 5.1
<#
.SYNOPSIS
    AXIOM Release Packager
.DESCRIPTION
    Builds all tools in release mode and packages into a distributable zip.
.PARAMETER Version
    Version string for the package filename (default: 0.11.0)
.EXAMPLE
    ./package.ps1
.EXAMPLE
    ./package.ps1 -Version 0.11.0
#>

param([string]$Version = "0.11.0")

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$releaseDir = "$root\release"
$binDir = "$releaseDir\axiom"

Write-Host "Building AXIOM v$Version release..." -ForegroundColor Magenta

# Build all tools
$tools = @("axiomc", "axiom-fmt", "axiom-doc", "axiom-ffigen", "axiom-pkg", "axiom-lsp")
foreach ($tool in $tools) {
    Write-Host "  Building $tool..." -ForegroundColor Cyan
    cargo build -p $tool --release
    if ($LASTEXITCODE -ne 0) { throw "Build failed for $tool" }
}

# Package
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
foreach ($tool in $tools) {
    Copy-Item "$root\target\release\$tool.exe" "$binDir\$tool.exe" -Force
}
Copy-Item "$root\axiom.bat" "$binDir\axiom.bat" -Force

# Copy stdlib + runtime
Copy-Item "$root\stdlib" "$binDir\stdlib" -Recurse -Force
New-Item -ItemType Directory -Path "$binDir\runtime" -Force | Out-Null
Copy-Item "$root\stdlib\runtime\axiom_runtime.c" "$binDir\runtime\" -Force

# Create install.bat for pre-built
@"
@echo off
echo AXIOM v$Version — Pre-built Installation
echo.
echo Installing to: %%LOCALAPPDATA%%\axiom
echo.
mkdir "%%LOCALAPPDATA%%\axiom\bin" 2>nul
xcopy /Y "%~dp0*" "%%LOCALAPPDATA%%\axiom\" /E
echo.
echo AXIOM installed! Add to PATH:
echo   %%LOCALAPPDATA%%\axiom\bin
echo.
echo Or run: %%LOCALAPPDATA%%\axiom\bin\axiom.bat --help
pause
"@ | Out-File -FilePath "$binDir\install.bat" -Encoding ASCII

# Create zip
$zipName = "axiom-v$Version-windows-x64.zip"
Compress-Archive -Path "$binDir\*" -DestinationPath "$releaseDir\$zipName" -Force

Write-Host ""
Write-Host "Release packaged: $releaseDir\$zipName" -ForegroundColor Green
Write-Host "Install with: install.ps1 -BinaryPath $binDir" -ForegroundColor Cyan
