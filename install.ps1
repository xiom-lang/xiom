#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Compiler v0.11.0 Installer
.DESCRIPTION
    Installs the XIOM toolchain: xiomc, xiom fmt, xiom doc, xiom ffigen, xiom pkg, xiom lsp
.PARAMETER InstallDir
    Installation directory (default: %LOCALAPPDATA%\xiom)
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
    [switch]$RegisterExt,
    [string]$BinaryPath = ""
)

$ErrorActionPreference = "Stop"
$xiomVersion = "0.20.0"
$xiomRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# ============================================================================
# Auto-install dependencies (only if building from source)
# ============================================================================
if (-not $BinaryPath) {
    Write-Host "Auto-installing missing dependencies..." -ForegroundColor Cyan
    Write-Host ""
    $depsScript = Join-Path $xiomRoot "install_deps.ps1"
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
    if (Get-Command nasm -ErrorAction SilentlyContinue) {
        Write-Host "  ✓ nasm found — hardware-accelerated crypto/memcpy enabled" -ForegroundColor Green
    } else {
        Write-Host "  - nasm not found (optional - install with: winget install NASM.NASM)" -ForegroundColor DarkGray
        Write-Host "    Without NASM, stdlib falls back to C software implementations." -ForegroundColor DarkGray
    }

    if (-not $depsOk) {
        Write-Host ""
        Write-Host "Critical dependencies missing — cannot build from source." -ForegroundColor Red
        Write-Host "Run: .\install_deps.ps1  to auto-install dependencies" -ForegroundColor Cyan
        Write-Host "Or use pre-built binaries: .\install.ps1 -BinaryPath .\release\xiom" -ForegroundColor Cyan
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
Write-Host "  XIOM Compiler v$xiomVersion" -ForegroundColor Cyan
Write-Host "  Safe, Verified, Precise — Systems Programming" -ForegroundColor DarkGray
Write-Host ""

# ============================================================================
# Installation directory
# ============================================================================
$defaultDir = "$env:LOCALAPPDATA\xiom"
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
    Write-Host "Building XIOM toolchain (release mode)..." -ForegroundColor Cyan
    Write-Host ""

    Push-Location $xiomRoot
    $tools = @("xiomc", "xiom-fmt", "xiom-doc", "xiom-ffigen", "xiom-pkg", "xiom-lsp")
    $built = 0
    $total = $tools.Count

    foreach ($tool in $tools) {
        Write-Progress -Activity "Building XIOM" -Status $tool -PercentComplete (($built / $total) * 100)
        cmd /c "cargo build -p $tool --release >nul 2>nul"
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Failed to build $tool" -ForegroundColor Red
            Pop-Location
            exit 1
        }
        $built++
    }
    Write-Progress -Activity "Building XIOM" -Completed
    Pop-Location
    $releaseDir = "$xiomRoot\target\release"
}

# ============================================================================
# Install
# ============================================================================
Write-Host ""
Write-Host "Installing to $installDir..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$files = @(
    "xiomc.exe", "xiom-fmt.exe", "xiom-doc.exe",
    "xiom-ffigen.exe", "xiom-pkg.exe", "xiom-lsp.exe"
)
foreach ($file in $files) {
    Copy-Item "$releaseDir\$file" "$binDir\$file" -Force
    $tag = if ($BinaryPath) { " (pre-built)" } else { "" }
    Write-Host "  + $file$tag" -ForegroundColor DarkGray
}
Copy-Item "$xiomRoot\xiom.bat" "$binDir\xiom.bat" -Force
Copy-Item "$xiomRoot\resource\img\xiom-icon.ico" "$binDir\xiom-icon.ico" -Force

# ============================================================================
# PATH (User or System)
# ============================================================================
if (-not $NoPath) {
    if ($Unattended) {
        $pathType = "u"
    } else {
        Write-Host ""
        Write-Host "Add to PATH? [U]ser (default) / [S]ystem (admin) / [N]o:" -ForegroundColor Yellow -NoNewline
        $pathType = Read-Host
    }
    if ($pathType -ne "n" -and $pathType -ne "N") {
        $isSystem = ($pathType -eq "s" -or $pathType -eq "S")
        if ($isSystem) {
            $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
            if (-not $isAdmin) {
                Write-Host "  System PATH requires admin. Adding to user PATH instead." -ForegroundColor Yellow
                $isSystem = $false
            }
        }
        $target = if ($isSystem) { "Machine" } else { "User" }
        $currentPath = [Environment]::GetEnvironmentVariable("Path", $target)
        if ($currentPath -notlike "*$binDir*") {
            [Environment]::SetEnvironmentVariable("Path", "$currentPath;$binDir", $target)
            Write-Host "  Added to $target PATH. Restart terminal to take effect." -ForegroundColor Green
        } else {
            Write-Host "  Already in $target PATH." -ForegroundColor DarkGray
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
    $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\XIOM CLI.lnk")
    $Shortcut.TargetPath = "cmd.exe"
    $Shortcut.Arguments = "/k `"$binDir\xiom.bat`" --help"
    $Shortcut.WorkingDirectory = $env:USERPROFILE
    $Shortcut.IconLocation = "$binDir\xiomc.exe,0"
    $Shortcut.Save()
    Write-Host "  Desktop shortcut created." -ForegroundColor Green
}

# ============================================================================
# .xi file association (Windows)
# ============================================================================
if ($Unattended) {
    $registerExt = if ($RegisterExt) { "y" } else { "n" }
} else {
    Write-Host ""
    Write-Host "Register .xi files with XIOM icon? (admin required) [y/N]:" -ForegroundColor Yellow -NoNewline
    $registerExt = Read-Host
}
if ($registerExt -eq "y" -or $registerExt -eq "Y") {
    try {
        $regPath = "HKCU:\Software\Classes\.xi"
        New-Item -Path $regPath -Force | Out-Null
        Set-ItemProperty -Path $regPath -Name "(Default)" -Value "XIOM.Source" -Type String
        New-Item -Path "HKCU:\Software\Classes\XIOM.Source" -Force | Out-Null
        Set-ItemProperty -Path "HKCU:\Software\Classes\XIOM.Source" -Name "(Default)" -Value "XIOM Source File" -Type String
        New-Item -Path "HKCU:\Software\Classes\XIOM.Source\DefaultIcon" -Force | Out-Null
        Set-ItemProperty -Path "HKCU:\Software\Classes\XIOM.Source\DefaultIcon" -Name "(Default)" -Value "$binDir\xiom-icon.ico" -Type String
        Write-Host "  .xi files now show XIOM icon in Explorer." -ForegroundColor Green
        Write-Host "  (Registered under HKCU — no admin required, current user only)" -ForegroundColor DarkGray
    } catch {
        Write-Host "  Could not register .xi file association: $_" -ForegroundColor Yellow
    }
}

# ============================================================================
# Uninstaller
# ============================================================================
$uninstaller = @"
@echo off
echo XIOM Uninstaller v$xiomVersion
echo.
echo This will remove XIOM from: $installDir
echo.
set /p confirm="Continue? [y/N]: "
if /i not "%confirm%"=="y" exit /b
rmdir /s /q "$installDir"
reg delete "HKCU\Software\Classes\.xi" /f >nul 2>nul
reg delete "HKCU\Software\Classes\XIOM.Source" /f >nul 2>nul
echo XIOM has been removed.
echo.
echo NOTE: You may need to manually remove XIOM from your system PATH.
echo   Settings ^> System ^> About ^> Advanced system settings ^> Environment Variables
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
Copy-Item "$xiomRoot\stdlib\*" "$libDir\" -Recurse -Force
Write-Host "  + stdlib -> $libDir" -ForegroundColor DarkGray

# Runtime
$rtDir = "$installDir\runtime"
New-Item -ItemType Directory -Force -Path $rtDir | Out-Null
Copy-Item "$xiomRoot\stdlib\runtime\*" "$rtDir\" -Force
Write-Host "  + runtime -> $rtDir" -ForegroundColor DarkGray

# Documentation (optional)
$docsDir = "$installDir\docs"
if (Test-Path "$xiomRoot\docs\language\html") {
    if ($Unattended) {
        $installDocs = "y"
    } else {
        Write-Host ""
        Write-Host "Install API documentation? [Y/n]:" -ForegroundColor Yellow -NoNewline
        $installDocs = Read-Host
    }
    if ($installDocs -ne "n" -and $installDocs -ne "N") {
        New-Item -ItemType Directory -Force -Path $docsDir | Out-Null
        Copy-Item "$xiomRoot\docs\language\html\*" "$docsDir\" -Recurse -Force
        Write-Host "  + docs -> $docsDir" -ForegroundColor DarkGray
        Write-Host "    (Open $docsDir\index.html in your browser)" -ForegroundColor DarkGray
    }
}

# ============================================================================
# Done
# ============================================================================
Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  XIOM v$xiomVersion installed successfully!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Restart your terminal, then try:" -ForegroundColor White
Write-Host "    xiom --help" -ForegroundColor Cyan
Write-Host "    xiom compile hello.xi" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Uninstall:" -ForegroundColor DarkGray
Write-Host "    $binDir\uninstall.bat" -ForegroundColor DarkGray
Write-Host ""
