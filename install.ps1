#Requires -Version 5.1
<#
.SYNOPSIS
    AXIOM Compiler v0.11.0 Installer
.DESCRIPTION
    Installs the AXIOM toolchain: axiomc, axiom fmt, axiom doc, axiom ffigen, axiom pkg, axiom lsp
.PARAMETER InstallDir
    Installation directory (default: %LOCALAPPDATA%\axiom)
.PARAMETER NoPath
    Skip adding to user PATH
.PARAMETER Shortcut
    Create desktop shortcut
.PARAMETER Unattended
    Run without interactive prompts (uses defaults)
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File install.ps1
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File install.ps1 -Unattended -Shortcut
.NOTES
    Run with: powershell -ExecutionPolicy Bypass -File install.ps1
#>

param(
    [string]$InstallDir = "",
    [switch]$NoPath,
    [switch]$Shortcut,
    [switch]$Unattended,
    [string]$BinaryPath = ""
)

$ErrorActionPreference = "Stop"
$axiomVersion = "0.11.0"
$axiomRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# ============================================================================
# Auto-install dependencies (only if building from source)
# ============================================================================
if (-not $BinaryPath) {
    Write-Host "Auto-installing missing dependencies..." -ForegroundColor Cyan
    Write-Host ""
    $depsScript = Join-Path $axiomRoot "install_deps.ps1"
    if (Test-Path $depsScript) {
        & $depsScript
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Dependency installation had issues — check output above." -ForegroundColor Yellow
        }
    } else {
        Write-Host "Dependency script not found at $depsScript" -ForegroundColor Yellow
    }

    # Verify critical deps are now available
    $depsOk = $true
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Host "  ✗ Rust not found. Install from https://rustup.rs" -ForegroundColor Red
        $depsOk = $false
    }
    if (-not (Get-Command clang -ErrorAction SilentlyContinue)) {
        Write-Host "  ✗ clang not found (needed to link native binaries)." -ForegroundColor Yellow
        Write-Host "    The compiler can emit .ll IR files without clang." -ForegroundColor DarkGray
        # NOT a hard failure — compiler can emit IR without clang
    }

    if (-not $depsOk) {
        Write-Host ""
        Write-Host "Critical dependencies missing — cannot build from source." -ForegroundColor Red
        Write-Host "Run: .\install_deps.ps1  to auto-install dependencies" -ForegroundColor Cyan
        Write-Host "Or use pre-built binaries: .\install.ps1 -BinaryPath .\release\axiom" -ForegroundColor Cyan
        exit 1
    }
    Write-Host ""
}

# ============================================================================
# Welcome
# ============================================================================
Clear-Host
Write-Host ""
Write-Host "  █████╗ ██╗  ██╗██╗ ██████╗ ███╗   ███╗" -ForegroundColor Magenta
Write-Host " ██╔══██╗╚██╗██╔╝██║██╔═══██╗████╗ ████║" -ForegroundColor Magenta
Write-Host " ███████║ ╚███╔╝ ██║██║   ██║██╔████╔██║" -ForegroundColor Magenta
Write-Host " ██╔══██║ ██╔██╗ ██║██║   ██║██║╚██╔╝██║" -ForegroundColor Magenta
Write-Host " ██║  ██║██╔╝ ██╗██║╚██████╔╝██║ ╚═╝ ██║" -ForegroundColor Magenta
Write-Host " ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝     ╚═╝" -ForegroundColor Magenta
Write-Host ""
Write-Host "  AXIOM Compiler v$axiomVersion" -ForegroundColor Cyan
Write-Host "  Safe, Verified, Precise — Systems Programming" -ForegroundColor DarkGray
Write-Host ""

# ============================================================================
# Installation directory
# ============================================================================
$defaultDir = "$env:LOCALAPPDATA\axiom"
if ($Unattended -or $InstallDir) {
    $installDir = $InstallDir
    if ([string]::IsNullOrWhiteSpace($installDir)) {
        $installDir = $defaultDir
    }
} else {
    Write-Host "Installation directory [$defaultDir]:" -ForegroundColor Yellow -NoNewline
    $installDir = Read-Host
    if ([string]::IsNullOrWhiteSpace($installDir)) {
        $installDir = $defaultDir
    }
}
$binDir = "$installDir\bin"

# ============================================================================
# Build (or use pre-built binaries)
# ============================================================================
if ($BinaryPath) {
    Write-Host "Installing pre-built binaries from: $BinaryPath" -ForegroundColor Cyan
    Write-Host ""
    $releaseDir = $BinaryPath
} else {
    Write-Host ""
    Write-Host "Building AXIOM toolchain (release mode)..." -ForegroundColor Cyan
    Write-Host ""

    Push-Location $axiomRoot
    $tools = @("axiomc", "axiom-fmt", "axiom-doc", "axiom-ffigen", "axiom-pkg", "axiom-lsp")
    $built = 0
    $total = $tools.Count

    foreach ($tool in $tools) {
        Write-Progress -Activity "Building AXIOM" -Status $tool -PercentComplete (($built / $total) * 100)
        cmd /c "cargo build -p $tool --release >nul 2>nul"
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Failed to build $tool" -ForegroundColor Red
            Pop-Location
            exit 1
        }
        $built++
    }
    Write-Progress -Activity "Building AXIOM" -Completed
    Pop-Location
    $releaseDir = "$axiomRoot\target\release"
}

# ============================================================================
# Install
# ============================================================================
Write-Host ""
Write-Host "Installing to $installDir..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$files = @(
    "axiomc.exe", "axiom-fmt.exe", "axiom-doc.exe",
    "axiom-ffigen.exe", "axiom-pkg.exe", "axiom-lsp.exe"
)
foreach ($file in $files) {
    Copy-Item "$releaseDir\$file" "$binDir\$file" -Force
    $tag = if ($BinaryPath) { " (pre-built)" } else { "" }
    Write-Host "  + $file$tag" -ForegroundColor DarkGray
}
Copy-Item "$axiomRoot\axiom.bat" "$binDir\axiom.bat" -Force
Copy-Item "$axiomRoot\resource\img\axiom-icon.ico" "$binDir\axiom-icon.ico" -Force

# ============================================================================
# PATH
# ============================================================================
if (-not $NoPath) {
    if ($Unattended) {
        $addPath = "y"
    } else {
        Write-Host ""
        Write-Host "Add to user PATH? [Y/n]:" -ForegroundColor Yellow -NoNewline
        $addPath = Read-Host
    }
    if ($addPath -ne "n" -and $addPath -ne "N") {
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$binDir*") {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$binDir", "User")
            Write-Host "  Added to PATH. Restart terminal to take effect." -ForegroundColor Green
        } else {
            Write-Host "  Already in PATH." -ForegroundColor DarkGray
        }
    }
}

# ============================================================================
# Desktop shortcut (optional)
# ============================================================================
if ($Unattended) {
    $createShortcut = if ($Shortcut) { "y" } else { "n" }
} else {
    Write-Host ""
    Write-Host "Create Desktop shortcut? [y/N]:" -ForegroundColor Yellow -NoNewline
    $createShortcut = Read-Host
}
if ($createShortcut -eq "y" -or $createShortcut -eq "Y") {
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\AXIOM CLI.lnk")
    $Shortcut.TargetPath = "cmd.exe"
    $Shortcut.Arguments = "/k `"$binDir\axiom.bat`" --help"
    $Shortcut.WorkingDirectory = $env:USERPROFILE
    $Shortcut.IconLocation = "$binDir\axiomc.exe,0"
    $Shortcut.Save()
    Write-Host "  Desktop shortcut created." -ForegroundColor Green
}

# ============================================================================
# Uninstaller
# ============================================================================
$uninstaller = @"
echo AXIOM Uninstaller
echo.
echo This will remove AXIOM from: $installDir
echo.
set /p confirm="Continue? [y/N]: "
if /i not "%confirm%"=="y" exit /b
rmdir /s /q "$installDir"
echo AXIOM has been removed.
echo.
echo NOTE: You may need to manually remove AXIOM from your system PATH.
echo   Settings > System > About > Advanced system settings > Environment Variables
pause
"@
Set-Content -Path "$binDir\uninstall.bat" -Value $uninstaller -Encoding ASCII

# ============================================================================
# Install stdlib + runtime
# ============================================================================
Write-Host ""
Write-Host "Installing standard library..." -ForegroundColor Cyan
$libDir = "$installDir\lib"
New-Item -ItemType Directory -Force -Path $libDir | Out-Null
Copy-Item "$axiomRoot\stdlib\*" "$libDir\" -Recurse -Force
Write-Host "  + stdlib -> $libDir" -ForegroundColor DarkGray

# Runtime
$rtDir = "$installDir\runtime"
New-Item -ItemType Directory -Force -Path $rtDir | Out-Null
Copy-Item "$axiomRoot\stdlib\runtime\*" "$rtDir\" -Force
Write-Host "  + runtime -> $rtDir" -ForegroundColor DarkGray

# ============================================================================
# Done
# ============================================================================
Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  AXIOM v$axiomVersion installed successfully!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Restart your terminal, then try:" -ForegroundColor White
Write-Host "    axiom --help" -ForegroundColor Cyan
Write-Host "    axiom compile hello.ax" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Uninstall:" -ForegroundColor DarkGray
Write-Host "    $binDir\uninstall.bat" -ForegroundColor DarkGray
Write-Host ""
