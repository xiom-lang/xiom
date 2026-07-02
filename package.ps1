#Requires -Version 5.1
<#
.SYNOPSIS
    AXIOM Release Packager
.DESCRIPTION
    Builds all tools in release mode and packages into a distributable portable folder + zip.
    The release folder can be used directly (copy to any location, add to PATH)
    or installed via the included install.bat.
.PARAMETER Version
    Version string for the package filename (default: 0.20.0)
.EXAMPLE
    ./package.ps1
.EXAMPLE
    ./package.ps1 -Version 0.20.0
#>

param([string]$Version = "0.20.0")

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$releaseDir = "$root\release"
$pkgDir = "$releaseDir\axiom-v$Version"
$binDir = "$pkgDir\bin"
$libDir = "$pkgDir\lib"
$rtDir  = "$pkgDir\runtime"

Write-Host ""
Write-Host "  AXIOM Release Packager v$Version" -ForegroundColor Magenta
Write-Host "  ================================" -ForegroundColor Magenta
Write-Host ""

# ── Build all tools ────────────────────────────────────────────────────
$tools = @("axiomc", "axiom-fmt", "axiom-doc", "axiom-ffigen", "axiom-pkg", "axiom-lsp")
foreach ($tool in $tools) {
    Write-Host "  Building $tool..." -ForegroundColor Cyan
    cargo build -p $tool --release
    if ($LASTEXITCODE -ne 0) { throw "Build failed for $tool" }
}

# ── Create release directory structure ─────────────────────────────────
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType Directory -Force -Path $libDir | Out-Null
New-Item -ItemType Directory -Force -Path $rtDir | Out-Null

# ── Copy binaries ──────────────────────────────────────────────────────
Write-Host ""
Write-Host "  Packaging release..." -ForegroundColor Cyan
foreach ($tool in $tools) {
    Copy-Item "$root\target\release\$tool.exe" "$binDir\$tool.exe" -Force
    Write-Host "    + $tool.exe" -ForegroundColor DarkGray
}

# ── Copy icon ──────────────────────────────────────────────────────────
Copy-Item "$root\resource\img\axiom-icon.ico" "$binDir\axiom-icon.ico" -Force
Write-Host "    + axiom-icon.ico" -ForegroundColor DarkGray

# ── Copy stdlib + runtime ──────────────────────────────────────────────
Copy-Item "$root\stdlib\*" "$libDir\" -Recurse -Force
Write-Host "    + stdlib/ -> lib/" -ForegroundColor DarkGray
Copy-Item "$root\stdlib\runtime\axiom_runtime.c" "$rtDir\" -Force
Write-Host "    + axiom_runtime.c -> runtime/" -ForegroundColor DarkGray

# ── Copy documentation ─────────────────────────────────────────────────
$docsDir = "$pkgDir\docs"
if (Test-Path "$root\docs\language\html") {
    New-Item -ItemType Directory -Force -Path $docsDir | Out-Null
    Copy-Item "$root\docs\language\html\*" "$docsDir\" -Recurse -Force
    Write-Host "    + docs/ (API reference)" -ForegroundColor DarkGray
}

# ── Create install.bat (portable CLI installer) ────────────────────────
@"
@echo off
setlocal enabledelayedexpansion
title AXIOM v$Version Installer

echo.
echo   AXIOM Compiler v$Version
echo   ========================
echo.
echo   This installer copies AXIOM to your chosen directory
echo   and optionally adds it to your user PATH.
echo.

:: ── Choose install directory ──────────────────────────────────────────
set "AXIOM_DEFAULT=%LOCALAPPDATA%\axiom"
set /p AXIOM_DIR="  Install directory [%AXIOM_DEFAULT%]: "
if "!AXIOM_DIR!"=="" set "AXIOM_DIR=%AXIOM_DEFAULT%"
set "AXIOM_BIN=!AXIOM_DIR!\bin"

:: ── Install files ─────────────────────────────────────────────────────
echo.
echo   Installing to !AXIOM_DIR!...
mkdir "!AXIOM_DIR!" 2>nul
mkdir "!AXIOM_BIN!" 2>nul

:: Copy binaries
copy /Y "%~dp0bin\*.exe" "!AXIOM_BIN!\" >nul 2>nul
copy /Y "%~dp0bin\axiom-icon.ico" "!AXIOM_BIN!\" >nul 2>nul
echo     + Binaries installed

:: Copy stdlib
if exist "%~dp0lib\" (
    xcopy /Y /E /Q "%~dp0lib\*" "!AXIOM_DIR!\lib\" >nul 2>nul
    echo     + Standard library installed
)

:: Copy runtime
if exist "%~dp0runtime\" (
    xcopy /Y /E /Q "%~dp0runtime\*" "!AXIOM_DIR!\runtime\" >nul 2>nul
    echo     + Runtime installed
)

:: Install documentation (optional)
if exist "%~dp0docs\" (
    echo.
    set /p INSTALL_DOCS="  Install API documentation? [Y/n]: "
    if /i not "!INSTALL_DOCS!"=="n" if /i not "!INSTALL_DOCS!"=="N" (
        xcopy /Y /E /Q "%~dp0docs\*" "!AXIOM_DIR!\docs\" >nul 2>nul
        echo     + Documentation installed ^(open !AXIOM_DIR!\docs\index.html^)
    )
)

:: ── Create axiom.bat wrapper ──────────────────────────────────────────
(
echo @echo off
echo REM AXIOM Toolchain v$Version
echo set "AXIOM_BIN=!AXIOM_BIN!"
echo if "%%1"=="" "%%AXIOM_BIN%%\axiomc.exe" --help ^& goto :eof
echo if "%%1"=="compile" ^( shift ^& "%%AXIOM_BIN%%\axiomc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="fmt"     ^( shift ^& "%%AXIOM_BIN%%\axiom-fmt.exe" %%* ^) ^& goto :eof
echo if "%%1"=="doc"     ^( shift ^& "%%AXIOM_BIN%%\axiom-doc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="ffigen"  ^( shift ^& "%%AXIOM_BIN%%\axiom-ffigen.exe" %%* ^) ^& goto :eof
echo if "%%1"=="pkg"     ^( shift ^& "%%AXIOM_BIN%%\axiom-pkg.exe" %%* ^) ^& goto :eof
echo if "%%1"=="lsp"     ^( shift ^& "%%AXIOM_BIN%%\axiom-lsp.exe" %%* ^) ^& goto :eof
echo REM Unknown subcommand — pass through to axiomc
echo "%%AXIOM_BIN%%\axiomc.exe" %%*
) > "!AXIOM_BIN!\axiom.bat"

:: ── Add to PATH ───────────────────────────────────────────────────────
echo.
echo   PATH options:
echo     [U] User PATH  - only your account ^(default, no admin needed^)
echo     [S] System PATH - all users ^(requires admin^)
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
    echo   Requesting System PATH ^(needs admin^)...
)

for /f "usebackq tokens=2,*" %%A in (`reg query !REG_HIVE!\!REG_KEY! /v PATH 2^>nul`) do set "CUR_PATH=%%B"
if "!CUR_PATH!"=="" (
    reg add !REG_HIVE!\!REG_KEY! /v PATH /t REG_EXPAND_SZ /d "!AXIOM_BIN!" /f >nul 2>nul
) else (
    echo !CUR_PATH! | find /i "!AXIOM_BIN!" >nul 2>nul
    if errorlevel 1 (
        reg add !REG_HIVE!\!REG_KEY! /v PATH /t REG_EXPAND_SZ /d "!CUR_PATH!;!AXIOM_BIN!" /f >nul 2>nul
    )
)
echo     + Added to PATH ^(restart terminal^)
:: Refresh PATH in current cmd session
set "PATH=%PATH%;!AXIOM_BIN!"

:: ── Register .ax file icon ────────────────────────────────────────────
echo.
set /p REG_EXT="  Register .ax files with AXIOM icon? [y/N]: "
if /i not "!REG_EXT!"=="y" goto :skip_reg

reg add HKCU\Software\Classes\.ax /ve /d "AXIOM.Source" /f >nul 2>nul
reg add HKCU\Software\Classes\AXIOM.Source /ve /d "AXIOM Source File" /f >nul 2>nul
reg add HKCU\Software\Classes\AXIOM.Source\DefaultIcon /ve /d "!AXIOM_BIN!\axiom-icon.ico" /f >nul 2>nul
echo     + .ax files registered

:skip_reg

:: ── Create uninstaller ────────────────────────────────────────────────
(
echo @echo off
echo echo AXIOM Uninstaller v$Version
echo echo.
echo echo This will remove: !AXIOM_DIR!
echo echo.
echo set /p CONFIRM="Continue? [y/N]: "
echo if /i not "%%CONFIRM%%"=="y" exit /b
echo rmdir /s /q "!AXIOM_DIR!"
echo reg delete HKCU\Software\Classes\.ax /f ^>nul 2^>nul
echo reg delete HKCU\Software\Classes\AXIOM.Source /f ^>nul 2^>nul
echo echo AXIOM removed. Remove from PATH manually if needed.
echo pause
) > "!AXIOM_BIN!\uninstall.bat"

:: ── Done ──────────────────────────────────────────────────────────────
echo.
echo   =========================================
echo   AXIOM v$Version installed successfully!
echo   =========================================
echo.
echo   Location:  !AXIOM_DIR!
echo   Binary:    !AXIOM_BIN!\axiomc.exe
echo   Wrapper:   !AXIOM_BIN!\axiom.bat
echo.
echo   Quick start:
echo     axiom compile hello.ax
echo     axiom --help
echo.
:: Check for clang (runtime dependency for native compilation)
where clang >nul 2>nul
if errorlevel 1 (
    echo   NOTE: clang not found on PATH.
    echo   AXIOM emits LLVM IR; clang compiles it to native .exe.
    echo   Install LLVM: winget install LLVM.LLVM
    echo   Or: https://github.com/llvm/llvm-project/releases
    echo   Without clang, use: axiomc --emit-ir file.ax ^(prints IR^)
)
echo.
echo   To uninstall: !AXIOM_BIN!\uninstall.bat
echo.
pause
endlocal
"@ | Out-File -FilePath "$pkgDir\install.bat" -Encoding ASCII
Write-Host "    + install.bat" -ForegroundColor DarkGray

# ── Create README ──────────────────────────────────────────────────────
@"
AXIOM v$Version — Portable Release
===================================

Quick install:
  Run: install.bat
  This copies AXIOM to %LOCALAPPDATA%\axiom and adds it to PATH.

Manual install:
  1. Copy this entire folder anywhere you like
  2. Add the \bin\ folder to your system PATH
  3. Run: axiomc --help

Contents:
  bin\       — axiomc.exe, axiom-fmt.exe, axiom-doc.exe, etc.
  lib\       — Standard library (.ax source files)
  runtime\   — C runtime (axiom_runtime.c)
  install.bat — Windows installer

Need dependencies? Run install_deps.ps1 from the source repo first.
"@ | Out-File -FilePath "$pkgDir\README.txt" -Encoding ASCII

# ── Create ZIP ─────────────────────────────────────────────────────────
$zipName = "axiom-v$Version-windows-x64.zip"
$zipPath = "$releaseDir\$zipName"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$pkgDir\*" -DestinationPath $zipPath -Force
Write-Host ""
Write-Host "  Release packaged:" -ForegroundColor Green
Write-Host "    Folder: $pkgDir" -ForegroundColor Green
Write-Host "    ZIP:    $zipPath" -ForegroundColor Green
Write-Host ""
Write-Host "  Install from this release:" -ForegroundColor Cyan
Write-Host "    .\install.ps1 -BinaryPath '$pkgDir'" -ForegroundColor White
Write-Host "  Or run the portable installer:" -ForegroundColor Cyan
Write-Host "    $pkgDir\install.bat" -ForegroundColor White
Write-Host ""
