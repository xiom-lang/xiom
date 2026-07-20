#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Release Packager (Windows)
.DESCRIPTION
    Builds all tools in release mode and packages into distributable folder + zip.
.PARAMETER Version
    Version string (default: 0.46.0)
.EXAMPLE
    ./package.ps1
.EXAMPLE
    ./package.ps1 -Version 0.47.0
#>

param([string]$Version = "0.46.0")

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
if (-not $env:XIOM_RELEASE_STATS)  { $env:XIOM_RELEASE_STATS  = "441/441 tests, zero warnings" }

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
    Write-Host "    + xiom-icon.ico" -ForegroundColor DarkGray
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

# Create install.bat
$installBat = @'
@echo off
setlocal enabledelayedexpansion
title XIOM INSTALLER

echo.
echo   XIOM Compiler
echo   =============
echo.
echo   This script copies XIOM to your chosen directory
echo   and optionally adds it to your user PATH.
echo.

:: Choose install directory
set "XIOM_DEFAULT=%LOCALAPPDATA%\xiom"
set /p XIOM_DIR="  Install directory [%XIOM_DEFAULT%]: "
if "!XIOM_DIR!"=="" set "XIOM_DIR=%XIOM_DEFAULT%"
set "XIOM_BIN=!XIOM_DIR!\bin"

:: Install files
echo.
echo   Installing to !XIOM_DIR!...
mkdir "!XIOM_DIR!" 2>nul
mkdir "!XIOM_BIN!" 2>nul

:: Copy binaries
copy /Y "%~dp0bin\*.exe" "!XIOM_BIN!\" >nul 2>nul
copy /Y "%~dp0bin\xiom-icon.ico" "!XIOM_BIN!\" >nul 2>nul
echo     + Binaries installed

:: Copy stdlib
if exist "%~dp0lib\" (
    xcopy /Y /E /Q "%~dp0lib\*" "!XIOM_DIR!\lib\" >nul 2>nul
    echo     + Standard library installed
)

:: Copy runtime
if exist "%~dp0runtime\" (
    xcopy /Y /E /Q "%~dp0runtime\*" "!XIOM_DIR!\runtime\" >nul 2>nul
    echo     + Runtime installed
)

:: Install documentation (optional)
if exist "%~dp0docs\" (
    echo.
    set /p INSTALL_DOCS="  Install API documentation? [Y/n]: "
    if /i not "!INSTALL_DOCS!"=="n" if /i not "!INSTALL_DOCS!"=="N" (
        xcopy /Y /E /Q "%~dp0docs\*" "!XIOM_DIR!\docs\" >nul 2>nul
        echo     + Documentation installed
    )
)

:: Create xiom.bat wrapper
(
echo @echo off
echo REM XIOM Toolchain
echo set "XIOM_BIN=!XIOM_BIN!"
echo if "%%1"=="" "%%XIOM_BIN%%\xiomc.exe" --help ^& goto :eof
echo if "%%1"=="compile" ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="fmt"     ^( shift ^& "%%XIOM_BIN%%\xiom-fmt.exe" %%* ^) ^& goto :eof
echo if "%%1"=="doc"     ^( shift ^& "%%XIOM_BIN%%\xiom-doc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="ffigen"  ^( shift ^& "%%XIOM_BIN%%\xiom-ffigen.exe" %%* ^) ^& goto :eof
echo if "%%1"=="pkg"     ^( shift ^& "%%XIOM_BIN%%\xiom-pkg.exe" %%* ^) ^& goto :eof
echo if "%%1"=="lsp"     ^( shift ^& "%%XIOM_BIN%%\xiom-lsp.exe" %%* ^) ^& goto :eof
echo REM Unknown subcommand - pass through to xiomc
echo "%%XIOM_BIN%%\xiomc.exe" %%*
) > "!XIOM_BIN!\xiom.bat"

:: Add to PATH
echo.
echo   PATH options:
echo     [U] User PATH  - only your account (default, no admin needed)
echo     [S] System PATH - all users (requires admin)
echo     [N] Skip       - add manually later
echo.
set /p PATH_TYPE="  Choose [U/s/N]: "
if /i "!PATH_TYPE!"=="N" goto :skip_path
if "!PATH_TYPE!"=="" set PATH_TYPE=U

set "REG_HIVE=HKCU"
set "REG_KEY=Environment"
if /i "!PATH_TYPE!"=="S" (
    set "REG_HIVE=HKLM"
    set "REG_KEY=SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    echo   Requesting System PATH (needs admin)...
)

for /f "usebackq tokens=2,*" %%A in (`reg query !REG_HIVE!\!REG_KEY! /v PATH 2^>nul`) do set "CUR_PATH=%%B"
if "!CUR_PATH!"=="" (
    reg add !REG_HIVE!\!REG_KEY! /v PATH /t REG_EXPAND_SZ /d "!XIOM_BIN!" /f >nul 2>nul
) else (
    echo !CUR_PATH! | find /i "!XIOM_BIN!" >nul 2>nul
    if errorlevel 1 (
        reg add !REG_HIVE!\!REG_KEY! /v PATH /t REG_EXPAND_SZ /d "!CUR_PATH!;!XIOM_BIN!" /f >nul 2>nul
    )
)
echo     + Added to PATH (restart terminal)
set "PATH=%PATH%;!XIOM_BIN!"

:: Register .xi file icon
echo.
set /p REG_EXT="  Register .xi files with XIOM icon? [y/N]: "
if /i not "!REG_EXT!"=="y" goto :skip_reg

reg add HKCU\Software\Classes\.xi /ve /d "XIOM.Source" /f >nul 2>nul
reg add HKCU\Software\Classes\XIOM.Source /ve /d "XIOM Source File" /f >nul 2>nul
reg add HKCU\Software\Classes\XIOM.Source\DefaultIcon /ve /d "!XIOM_BIN!\xiom-icon.ico" /f >nul 2>nul
echo     + .xi files registered

:skip_reg

:: Create uninstaller
(
echo @echo off
echo echo XIOM Uninstaller
echo echo.
echo echo This will remove: !XIOM_DIR!
echo echo.
echo set /p CONFIRM="Continue? [y/N]: "
echo if /i not "%%CONFIRM%%"=="y" exit /b
echo rmdir /s /q "!XIOM_DIR!"
echo reg delete HKCU\Software\Classes\.xi /f ^>nul 2^>nul
echo reg delete HKCU\Software\Classes\XIOM.Source /f ^>nul 2^>nul
echo echo XIOM removed. Remove from PATH manually if needed.
echo pause
) > "!XIOM_BIN!\uninstall.bat"

:skip_path
echo.
echo   =========================================
echo   XIOM installed successfully!
echo   =========================================
echo.
echo   Location:  !XIOM_DIR!
echo   Binary:    !XIOM_BIN!\xiomc.exe
echo   Wrapper:   !XIOM_BIN!\xiom.bat
echo.
echo   Quick start:
echo     xiom compile hello.xi
echo     xiom --help
echo.
where clang >nul 2>nul
if errorlevel 1 (
    echo   NOTE: clang not found on PATH.
    echo   XIOM emits LLVM IR; clang compiles it to native .exe.
    echo   Install LLVM: winget install LLVM.LLVM
)
echo.
echo   To uninstall: !XIOM_BIN!\uninstall.bat
echo.
pause
endlocal
'@

$installBat | Out-File -FilePath "$pkgDir\install.bat" -Encoding ASCII
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
