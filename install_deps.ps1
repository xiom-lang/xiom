#Requires -Version 5.1
<#
.SYNOPSIS
    AXIOM Dependency Auto-Installer — Windows
.DESCRIPTION
    Detects OS, checks for required build/runtime dependencies,
    and auto-installs any that are missing using winget, choco, or direct download.
    Dependencies: Rust (rustc/cargo), LLVM (clang), C++ Build Tools (linker), Git
.NOTES
    Run with: powershell -ExecutionPolicy Bypass -File install_deps.ps1
#>

$ErrorActionPreference = "Stop"
$script:installedCount = 0
$script:skippedCount = 0
$script:failedCount = 0

# ============================================================================
# Helpers
# ============================================================================

function Write-Header { Write-Host "`n  $args" -ForegroundColor Cyan }
function Write-Ok     { Write-Host "    [OK] $args" -ForegroundColor Green }
function Write-Warn   { Write-Host "   [WARN] $args" -ForegroundColor Yellow }
function Write-Fail   { Write-Host "   [FAIL] $args" -ForegroundColor Red }
function Write-Info   { Write-Host "   [INFO] $args" -ForegroundColor DarkGray }

function Test-Command($name) {
    return $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

function Test-Winget {
    return $null -ne (Get-Command winget -ErrorAction SilentlyContinue)
}

function Test-Choco {
    return $null -ne (Get-Command choco -ErrorAction SilentlyContinue)
}

function Invoke-WingetInstall($id, $name) {
    Write-Info "Installing $name via winget..."
    $result = cmd /c "winget install --id $id --silent --accept-package-agreements --accept-source-agreements 2>&1"
    if ($LASTEXITCODE -eq 0) {
        Write-Info "winget reported success for $name (may need terminal restart)"
        return $true
    }
    Write-Warn "winget install of $name returned code $LASTEXITCODE"
    Write-Info "Output: $result"
    return $false
}

function Invoke-ChocoInstall($pkg, $name) {
    Write-Info "Installing $name via chocolatey..."
    $result = choco install $pkg -y --limit-output 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Info "choco reported success for $name"
        return $true
    }
    return $false
}

# Refresh PATH from registry (new installs won't be visible until next session otherwise)
function Update-SessionPath {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("Path","User")
}

# ============================================================================
# Banner
# ============================================================================
Clear-Host
Write-Host ""
Write-Host "  AXIOM Dependency Installer (Windows)" -ForegroundColor Magenta
Write-Host "  --------------------------------------" -ForegroundColor Magenta
Write-Host ""
Write-Host "  Detecting package managers..." -ForegroundColor Cyan

$hasWinget = Test-Winget
$hasChoco  = Test-Choco
Write-Info "winget:   $(if ($hasWinget) { 'available' } else { 'not found' })"
Write-Info "choco:    $(if ($hasChoco)  { 'available' } else { 'not found' })"
Write-Info "manual:   fallback (direct download)"
Write-Host ""

# ============================================================================
# 1. Rust (rustc + cargo)
# ============================================================================
Write-Header "1. Rust (rustc + cargo)"

if (Test-Command "rustc") {
    $ver = (rustc --version 2>$null)
    Write-Ok "already installed — $ver"
    $script:skippedCount++
} else {
    Write-Info "Rust not found — installing..."
    $installed = $false

    # Try winget first
    if ($hasWinget -and -not $installed) {
        $installed = Invoke-WingetInstall "Rustlang.Rustup" "Rust"
    }
    # Try choco
    if ($hasChoco -and -not $installed) {
        $installed = Invoke-ChocoInstall "rust" "Rust"
    }
    # Fallback: direct rustup-init download
    if (-not $installed) {
        Write-Info "Downloading rustup-init.exe..."
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupExe = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing
        Write-Info "Running rustup-init (unattended)..."
        $proc = Start-Process -FilePath $rustupExe -ArgumentList "-y --default-toolchain stable" -Wait -PassThru -NoNewWindow
        Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
        if ($proc.ExitCode -eq 0) {
            Write-Ok "Rust installed via rustup"
            $env:Path += ";$env:USERPROFILE\.cargo\bin"
            $script:installedCount++
            $installed = $true
        }
    }

    if ($installed) {
        $script:installedCount++
    } else {
        Write-Fail "Could not install Rust. Install manually: https://rustup.rs"
        $script:failedCount++
    }
}

# ============================================================================
# 2. LLVM / clang
# ============================================================================
Write-Header "2. LLVM / clang (required to compile IR to native binary)"

if (Test-Command "clang") {
    $ver = (clang --version 2>$null | Select-Object -First 1)
    Write-Ok "already installed — $ver"
    $script:skippedCount++
} else {
    Write-Info "clang not found — installing..."
    $installed = $false

    # Try winget first (LLVM ships clang)
    if ($hasWinget -and -not $installed) {
        $installed = Invoke-WingetInstall "LLVM.LLVM" "LLVM/clang"
    }
    # Try choco
    if ($hasChoco -and -not $installed) {
        $installed = Invoke-ChocoInstall "llvm" "LLVM/clang"
    }
    # Fallback: direct LLVM download
    if (-not $installed) {
        $llvmVersion = "19.1.0"
        $llvmUrl = "https://github.com/llvm/llvm-project/releases/download/llvmorg-$llvmVersion/LLVM-$llvmVersion-win64.exe"
        $llvmExe = "$env:TEMP\LLVM-$llvmVersion-win64.exe"
        Write-Info "Downloading LLVM $llvmVersion..."
        Invoke-WebRequest -Uri $llvmUrl -OutFile $llvmExe -UseBasicParsing
        Write-Info "Running LLVM installer (unattended, add to PATH)..."
        $proc = Start-Process -FilePath $llvmExe -ArgumentList "/S /D=C:\Program Files\LLVM" -Wait -PassThru -NoNewWindow
        Remove-Item $llvmExe -Force -ErrorAction SilentlyContinue
        if ($proc.ExitCode -eq 0) {
            Write-Ok "LLVM installed. Restart terminal for PATH to take effect."
            $script:installedCount++
            $installed = $true
        }
    }

    if ($installed) {
        Update-SessionPath
        $script:installedCount++
    } else {
        Write-Fail "Could not install LLVM/clang."
        Write-Info "  Manual install: https://github.com/llvm/llvm-project/releases"
        Write-Info "  NOTE: Without clang, the compiler emits .ll IR files but cannot link native binaries."
        Write-Info "  The compiler itself (axiomc) does not require clang to run."
        $script:failedCount++
    }
}

# ============================================================================
# 3. C++ Build Tools / Windows SDK (linker — needed by clang on Windows)
# ============================================================================
Write-Header "3. C++ Build Tools / Windows SDK (link.exe)"

if (Test-Command "link") {
    Write-Ok "link.exe already available"
    $script:skippedCount++
} else {
    Write-Info "link.exe not found — may need Visual Studio Build Tools"
    Write-Info "  If clang complains about missing 'link.exe' after install:"
    Write-Info "  Run: winget install Microsoft.VisualStudio.2022.BuildTools --override '--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools'"

    # Try winget auto-install
    if ($hasWinget) {
        $vsInstalled = Invoke-WingetInstall "Microsoft.VisualStudio.2022.BuildTools" "VS Build Tools"
        if ($vsInstalled) {
            Write-Info "VS Build Tools queued. You may need to run the installer GUI once."
            Write-Info "Or use lld-link which ships with LLVM: clang -fuse-ld=lld"
        }
    }

    # Not a hard failure — clang can use lld-link which ships with LLVM
    Write-Info "  Alternatively, clang can use lld-link (ships with LLVM)."
    Write-Info "  Pass -fuse-ld=lld to clang, or set it as default."
}

# ============================================================================
# 4. Git
# ============================================================================
Write-Header "4. Git (optional — for package manager)"

if (Test-Command "git") {
    $ver = (git --version 2>$null)
    Write-Ok "already installed — $ver"
    $script:skippedCount++
} else {
    Write-Info "Git not found — optional, only needed for 'axiom pkg install'"
    if ($hasWinget) {
        Invoke-WingetInstall "Git.Git" "Git" | Out-Null
    }
}

# ============================================================================
# Summary
# ============================================================================
Write-Host ""
Write-Host "  ========================================" -ForegroundColor Magenta
Write-Host "  Dependency installation complete" -ForegroundColor Magenta
Write-Host "  ========================================" -ForegroundColor Magenta
Write-Host ""
Write-Host "  Installed : $script:installedCount" -ForegroundColor Green
Write-Host "  Skipped   : $script:skippedCount  (already present)" -ForegroundColor DarkGray
Write-Host "  Failed    : $script:failedCount" -ForegroundColor $(if ($script:failedCount -gt 0) { "Red" } else { "DarkGray" })
Write-Host ""

if ($script:installedCount -gt 0) {
    Write-Host "  IMPORTANT:" -ForegroundColor Yellow
    Write-Host "  Close and reopen PowerShell for PATH changes to take effect." -ForegroundColor Yellow
    Write-Host "  Then run: .\install.ps1" -ForegroundColor White
} elseif ($script:failedCount -eq 0) {
    Write-Host "  All dependencies present. Ready to install AXIOM:" -ForegroundColor Green
    Write-Host "  Run: .\install.ps1" -ForegroundColor White
} else {
    Write-Host "  Some dependencies could not be installed automatically." -ForegroundColor Red
    Write-Host "  Install them manually, then run: .\install.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "  Manual links:" -ForegroundColor DarkGray
    Write-Host "    Rust:   https://rustup.rs" -ForegroundColor DarkGray
    Write-Host "    LLVM:   https://github.com/llvm/llvm-project/releases" -ForegroundColor DarkGray
    Write-Host "    VS BT:  https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor DarkGray
}

Write-Host ""
exit 0
