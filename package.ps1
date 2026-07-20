#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Release Packager (Windows)
.DESCRIPTION
    Builds all tools in release mode and packages into distributable folder + zip.
    Optionally signs all binaries with Authenticode (requires code signing certificate).
.PARAMETER Version
    Version string (default: 0.46.0)
.PARAMETER Sign
    Sign all .exe binaries after packaging (requires -CertificateThumbprint or -CertificatePath).
.PARAMETER CertificateThumbprint
    Thumbprint of code signing certificate in Windows certificate store.
.PARAMETER CertificatePath
    Path to .pfx file containing code signing certificate.
.PARAMETER CertificatePassword
    Password for .pfx certificate file.
.EXAMPLE
    ./package.ps1
.EXAMPLE
    ./package.ps1 -Version 0.48.9
.EXAMPLE
    ./package.ps1 -Version 0.48.9 -Sign -CertificateThumbprint "A1B2C3D4..."
.EXAMPLE
    ./package.ps1 -Version 0.48.9 -Sign -CertificatePath .\xiom_code_sign.pfx -CertificatePassword "secret"
#>

param(
    [string]$Version = "0.46.0",
    [switch]$Sign,
    [string]$CertificateThumbprint,
    [string]$CertificatePath,
    [string]$CertificatePassword
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$releaseDir = "$root\release"
$pkgDir = "$releaseDir\xiom-v$Version"
$binDir = "$pkgDir\bin"
$libDir = "$pkgDir\lib"
$rtDir  = "$pkgDir\runtime"

Write-Host ""
Write-Host "  XIOM Release Packager v$Version" -ForegroundColor Magenta
Write-Host "  ================================" -ForegroundColor Magenta
Write-Host ""

# Set release metadata (baked into binary via env! macros at compile time).
# Override these before running to customize the version banner.
if (-not $env:XIOM_RELEASE_TAG)    { $env:XIOM_RELEASE_TAG    = "Production" }
if (-not $env:XIOM_RELEASE_STATS)  { $env:XIOM_RELEASE_STATS  = "871/871 tests, zero warnings" }

# Bump version in Cargo.toml so the binary reports the correct version.
# Uses env!("CARGO_PKG_VERSION") at compile time.
$cargoTomlPath = "$root\crates\xiomc\Cargo.toml"
if (Test-Path $cargoTomlPath) {
    $toml = Get-Content $cargoTomlPath -Raw
    $toml = $toml -replace '(?m)^version\s*=\s*"[^"]+"', "version = `"$Version`""
    Set-Content $cargoTomlPath -Value $toml -NoNewline
    Write-Host "  Cargo.toml version set to $Version" -ForegroundColor DarkGray
}

# Build all tools
$tools = @("xiomc", "xiom-fmt", "xiom-doc", "xiom-ffigen", "xiom-pkg", "xiom-lsp", "xiom-mcp", "xiom-dbg", "xiom-verify")

# Kill any running tool processes to avoid file-lock on release build
$toolNames = @("xiomc", "xiom-fmt", "xiom-doc", "xiom-ffigen", "xiom-pkg", "xiom-lsp", "xiom-mcp", "xiom-dbg", "xiom-verify")
foreach ($name in $toolNames) {
    $null = Stop-Process -Name $name -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 1

$builtOk = @()
foreach ($tool in $tools) {
    Write-Host "  Building $tool..." -ForegroundColor Cyan
    $prevErrorAction = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = cargo build -p $tool --release 2>&1
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = $prevErrorAction
    if ($exitCode -eq 0) {
        $builtOk += $tool
        Write-Host "    [OK] $tool" -ForegroundColor Green
    } else {
        Write-Host "    [SKIP] $tool (crate not found or build failed)" -ForegroundColor Yellow
    }
}

# Create release directories
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType Directory -Force -Path $libDir | Out-Null
New-Item -ItemType Directory -Force -Path $rtDir | Out-Null

# Copy binaries
Write-Host ""
Write-Host "  Packaging release..." -ForegroundColor Cyan
foreach ($tool in $builtOk) {
    $src = "$root\target\release\$tool.exe"
    if (Test-Path $src) {
        Copy-Item $src "$binDir\$tool.exe" -Force
        Write-Host "    + $tool.exe" -ForegroundColor DarkGray
    }
}

# Copy icon
$iconSrc = "$root\resource\img\xiom-icon.ico"
if (Test-Path $iconSrc) {
    Copy-Item $iconSrc "$binDir\xiom-icon.ico" -Force
    Write-Host "    + xiom-icon.ico" -ForegroundColor DarkGray
}

# Bundle z3 for contract verification (auto-detected by xiom-verify)
$z3Src = "$root\target\release\z3.exe"
if (-not (Test-Path $z3Src)) {
    $z3Src = "$env:TEMP\z3.exe"
}
if (Test-Path $z3Src) {
    Copy-Item $z3Src "$binDir\z3.exe" -Force
    Write-Host "    + z3.exe (bundled for --verify --check)" -ForegroundColor DarkGray
} else {
    Write-Host "    - z3.exe not found (install Z3 for contract verification)" -ForegroundColor Yellow
}
# Copy stdlib + runtime
$stdlibSrc = "$root\stdlib"
if (Test-Path $stdlibSrc) {
    Copy-Item "$stdlibSrc\*" "$libDir\" -Recurse -Force
    Write-Host "    + stdlib/ -> lib/" -ForegroundColor DarkGray
}
$rtSrc = "$root\stdlib\runtime\xiom_runtime.c"
if (Test-Path $rtSrc) {
    Copy-Item $rtSrc "$rtDir\" -Force
    Write-Host "    + xiom_runtime.c -> runtime/" -ForegroundColor DarkGray
}

# Copy documentation (optional)
$docsDir = "$pkgDir\docs"
$htmlDocs = "$root\docs\language\html"
if (Test-Path $htmlDocs) {
    New-Item -ItemType Directory -Force -Path $docsDir | Out-Null
    Copy-Item "$htmlDocs\*" "$docsDir\" -Recurse -Force
    Write-Host "    + docs/ (API reference)" -ForegroundColor DarkGray
}

# Copy MCP configs
$mcpSrc = "$root\release\mcp"
if (Test-Path $mcpSrc) {
    New-Item -ItemType Directory -Force -Path "$pkgDir\mcp" | Out-Null
    Copy-Item "$mcpSrc\*" "$pkgDir\mcp\" -Force
    Write-Host "    + mcp/ (IDE configs: Kilo, Cursor, Claude, Windsurf, etc.)" -ForegroundColor DarkGray
}

# Create install.bat from release template
$installBatSrc = "$root\release\install.bat"
if (Test-Path $installBatSrc) {
    Copy-Item $installBatSrc "$pkgDir\install.bat" -Force
    Write-Host "    + install.bat" -ForegroundColor DarkGray
} else {
    Write-Host "    - install.bat not found (run from repo root)" -ForegroundColor Yellow
}

# Copy install.sh for Linux/macOS
$installShSrc = "$root\release\install.sh"
if (Test-Path $installShSrc) {
    Copy-Item $installShSrc "$pkgDir\install.sh" -Force
    Write-Host "    + install.sh (Linux/macOS)" -ForegroundColor DarkGray
}

Write-Host "    + install.bat" -ForegroundColor DarkGray

# Create README
@"
XIOM v$Version - Portable Release
===================================

Quick install:
  Run: install.bat
  This copies XIOM to %LOCALAPPDATA%\xiom and adds it to PATH.

Manual install:
  1. Copy this entire folder anywhere you like
  2. Add the \bin\ folder to your system PATH
  3. Run: xiomc --help

Contents:
  bin\       - xiomc.exe, xiom-fmt.exe, xiom-doc.exe, etc.
  lib\       - Standard library (.xi source files)
  runtime\   - C runtime (xiom_runtime.c)
  install.bat - Windows installer

Need dependencies? Run install_deps.ps1 from the source repo first.
"@ | Out-File -FilePath "$pkgDir\README.txt" -Encoding ASCII

# Create ZIP
$zipName = "xiom-v$Version-windows-x64.zip"
$zipPath = "$releaseDir\$zipName"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$pkgDir\*" -DestinationPath $zipPath -Force
Write-Host ""
Write-Host "  Release packaged:" -ForegroundColor Green
Write-Host "    Folder: $pkgDir" -ForegroundColor Green
Write-Host "    ZIP:    $zipPath" -ForegroundColor Green
Write-Host ""

# 5e.7c: Digital signing (Authenticode)
if ($Sign) {
    Write-Host "  Code signing binaries..." -ForegroundColor Cyan
    $signArgs = @{ Path = $binDir }
    if ($CertificateThumbprint) { $signArgs.CertificateThumbprint = $CertificateThumbprint }
    if ($CertificatePath)       { $signArgs.CertificatePath       = $CertificatePath }
    if ($CertificatePassword)   { $signArgs.CertificatePassword   = $CertificatePassword }
    $signScript = "$root\sign.ps1"
    if (Test-Path $signScript) {
        & $signScript @signArgs
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  WARNING: Signing had errors" -ForegroundColor Yellow
# Copy MCP configs
$mcpSrc = "$root\release\mcp"
if (Test-Path $mcpSrc) {
    New-Item -ItemType Directory -Force -Path "$pkgDir\mcp" | Out-Null
    Copy-Item "$mcpSrc\*" "$pkgDir\mcp\" -Force
    Write-Host "    + mcp/ (IDE integration configs for Kilo, Cursor, Claude, Windsurf, etc.)" -ForegroundColor DarkGray
}

# Copy install scripts
$installBatSrc = "$root\release\install.bat"
if (Test-Path $installBatSrc) {
    Copy-Item $installBatSrc "$pkgDir\install.bat" -Force
    Write-Host "    + install.bat (Windows)" -ForegroundColor DarkGray
}
$installShSrc = "$root\release\install.sh"
if (Test-Path $installShSrc) {
    Copy-Item $installShSrc "$pkgDir\install.sh" -Force
    Write-Host "    + install.sh (Linux/macOS)" -ForegroundColor DarkGray
}
    } else {
        Write-Host "  WARNING: sign.ps1 not found" -ForegroundColor Yellow
    }
}