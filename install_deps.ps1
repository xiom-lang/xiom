#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Dependency Auto-Installer -- Windows
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

function Invoke-WingetInstall($id, $name, $extraArgs = "") {
    Write-Info "Installing $name via winget..."
    $wingetArgs = "install --id $id --silent --accept-package-agreements --accept-source-agreements"
    if ($extraArgs) {
        $wingetArgs += " $extraArgs"
    }
    $result = cmd /c "winget $wingetArgs 2>&1"
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
Write-Host "  XIOM Dependency Installer (Windows)" -ForegroundColor Magenta
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
    Write-Ok "already installed -- $ver"
    $script:skippedCount++
} else {
    Write-Info "Rust not found -- installing..."
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
    Write-Ok "already installed -- $ver"
    $script:skippedCount++
} else {
    # clang not on PATH, but LLVM might be installed elsewhere.
    # Search common installation directories first.
    $llvmPaths = @(
        "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\LLVM.LLVM_*\bin\clang.exe",
        "C:\Program Files\LLVM\bin\clang.exe",
        "$env:LOCALAPPDATA\Programs\LLVM\bin\clang.exe",
        "$env:ProgramData\Microsoft\WinGet\Packages\LLVM.LLVM_*\bin\clang.exe"
    )
    $foundClang = $null
    foreach ($pattern in $llvmPaths) {
        $found = Get-ChildItem -Path $pattern -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) {
            $foundClang = $found.FullName
            break
        }
    }

    if ($foundClang) {
        $llvmBin = Split-Path -Parent $foundClang
        Write-Ok "LLVM found at $llvmBin (not on PATH)"
        Write-Info "Adding $llvmBin to user PATH..."
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$llvmBin*") {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$llvmBin", "User")
        }
        $env:Path = "$env:Path;$llvmBin"
        $script:installedCount++
    } else {
        Write-Info "clang not found -- installing..."
        $installed = $false

        # Try winget first (even if it says "already installed", try anyway)
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
            try {
                Invoke-WebRequest -Uri $llvmUrl -OutFile $llvmExe -UseBasicParsing -TimeoutSec 120
                Write-Info "Running LLVM installer (unattended, add to PATH)..."
                $proc = Start-Process -FilePath $llvmExe -ArgumentList "/S /D=C:\Program Files\LLVM" -Wait -PassThru -NoNewWindow
                Remove-Item $llvmExe -Force -ErrorAction SilentlyContinue
                if ($proc.ExitCode -eq 0) {
                    Write-Ok "LLVM installed. Restart terminal for PATH to take effect."
                    $script:installedCount++
                    $installed = $true
                }
            } catch {
                Write-Warn "LLVM download failed (timeout/network): $_"
                Write-Info "  Install manually: https://github.com/llvm/llvm-project/releases"
            }
        }

        if ($installed) {
            Update-SessionPath
            $script:installedCount++
        } else {
            Write-Fail "Could not install LLVM/clang."
            Write-Info "  Manual install: https://github.com/llvm/llvm-project/releases"
            Write-Info "  NOTE: Without clang, the compiler emits .ll IR files but cannot link native binaries."
            Write-Info "  The compiler itself (xiom) does not require clang to run."
            $script:failedCount++
        }
    }
}

# After LLVM install, verify it's on PATH
if (Test-Command "clang") {
    $clangPath = (Get-Command clang -ErrorAction SilentlyContinue).Source
    Write-Info "clang location: $clangPath"
    # Ensure LLVM bin is in user PATH for persistence
    if ($clangPath) {
        $llvmBin = Split-Path -Parent $clangPath
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$llvmBin*") {
            Write-Info "Adding $llvmBin to user PATH..."
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$llvmBin", "User")
        }
    }
}

# ============================================================================
# 3. C Headers & Windows SDK (stdio.h -- needed by clang to compile xiom_runtime.c)
# ============================================================================
Write-Header "3. C/C++ Headers + Windows SDK (stdio.h, stdlib.h)"

# Test if clang can actually compile a trivial C program (verifies headers exist)
$clangCanCompile = $false
if (Test-Command "clang") {
    $testDir = "$env:TEMP\xiom_clang_test"
    New-Item -ItemType Directory -Force -Path $testDir | Out-Null
    "#include <stdio.h>`nint main() { return 0; }" | Out-File -FilePath "$testDir\test.c" -Encoding ASCII
    $savedErrorAction = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $clangOut = clang -o "$testDir\test.exe" "$testDir\test.c" 2>&1
    $ErrorActionPreference = $savedErrorAction
    if ($LASTEXITCODE -eq 0) {
        Write-Ok "clang can compile C -- C headers found"
        $clangCanCompile = $true
    } else {
        Write-Warn "clang cannot compile C (missing C standard library headers)"
        if ($clangOut) { Write-Info "  $clangOut" }
    }
    Remove-Item -Recurse -Force $testDir -ErrorAction SilentlyContinue
}

if ($clangCanCompile) {
    $script:skippedCount++
} else {
    Write-Info "C headers not found -- clang needs Windows SDK or VS Build Tools"
    Write-Info "  On Windows, stdio.h comes from the Windows SDK, which is installed"
    Write-Info "  with Visual Studio Build Tools (C++ workload). Installing now..."

    $vsInstalled = $false

    # Try winget with full C++ workload
    if ($hasWinget -and -not $vsInstalled) {
        $overrideArgs = '--override "--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --includeRecommended"'
        $vsInstalled = Invoke-WingetInstall "Microsoft.VisualStudio.2022.BuildTools" "VS 2022 Build Tools (C++)" $overrideArgs
    }

    # Try choco
    if ($hasChoco -and -not $vsInstalled) {
        $vsInstalled = Invoke-ChocoInstall "visualstudio2022buildtools --params '--add Microsoft.VisualStudio.Workload.VCTools'" "VS Build Tools"
    }

    if (-not $vsInstalled) {
        Write-Fail "Could not auto-install C/C++ headers."
        Write-Info "  Manual fix -- run ONE of the following:"
        Write-Info ""
        Write-Info "  Option A (recommended): Install VS 2022 Build Tools with C++"
        Write-Info "    winget install Microsoft.VisualStudio.2022.BuildTools"
        Write-Info "    Then run the Visual Studio Installer, select 'Desktop development with C++'"
        Write-Info ""
        Write-Info "  Option B: Install just the Windows SDK"
        Write-Info "    winget install Microsoft.WindowsSDK.10.0.22621"
        Write-Info ""
        Write-Info "  Option C: Install MinGW-w64 (alternative CRT)"
        Write-Info "    winget install MSYS2.MSYS2"
        Write-Info "    Then: pacman -S mingw-w64-ucrt-x86_64-gcc"
        Write-Info ""
        Write-Info "  After installing, RESTART your terminal and re-run this script."
        Write-Info "  NOTE: This is needed because xiom_runtime.c uses stdio."
        Write-Info "  The compiler emits valid LLVM IR without C headers."
        Write-Info "  Only native binary linking requires them."
        $script:failedCount++
    } else {
        Write-Info "VS Build Tools installation queued."
        Write-Info "  The installer may open a GUI -- select 'Desktop development with C++'"
        Write-Info "  and click Install. After it completes, RESTART your terminal."
        Write-Info "  Then re-run: .\install_deps.ps1"
        $script:installedCount++
    }
}

# ============================================================================
# 4. Git
# ============================================================================
Write-Header "4. Git (optional - for package manager)"

if (Test-Command "git") {
    $ver = (git --version 2>$null)
    Write-Ok "already installed -- $ver"
    $script:skippedCount++
} else {
    Write-Info "Git not found -- optional, only needed for 'xiom pkg install'"
    if ($hasWinget) {
        Invoke-WingetInstall "Git.Git" "Git" | Out-Null
    }
}

# ============================================================================
# 5. NASM (optional — hardware-accelerated stdlib functions)
# ============================================================================
Write-Header "5. NASM (optional - crypto/memcpy/simd assembly acceleration)"

if (Test-Command "nasm") {
    $ver = (nasm --version 2>$null | Select-Object -First 1)
    Write-Ok "already installed -- $ver"
    $script:skippedCount++
} else {
    Write-Info "NASM not found - optional, enables AES-NI, fast memcpy, context switching"
    Write-Info "  Install: winget install NASM.NASM"
    Write-Info "  Without NASM: stdlib falls back to C software implementations."
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
    Write-Host "  All dependencies present. Ready to install XIOM:" -ForegroundColor Green
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
