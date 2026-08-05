#Requires -Version 5.1
param([switch]$Fast, [int]$Threads = 8, [switch]$E2EOnly)

$ErrorActionPreference = "Continue"
$startTime = Get-Date
$totalPassed = 0; $totalFailed = 0; $totalIgnored = 0

function Test-Crate($pkg, $testFile, $label, $extraFilter) {
    $args = @("test", "-p", $pkg)
    if ($testFile) { $args += "--test", $testFile }
    if ($extraFilter) { $args += $extraFilter }
    $args += "--", "--test-threads=$Threads"
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & cargo @args 2>&1 | Out-String
    $sw.Stop()
    
    $p = 0; $f = 0; $i = 0
    if ($output -match 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
        $p = [int]$Matches[1]; $f = [int]$Matches[2]; $i = [int]$Matches[3]
    }
    $elapsed = "$($sw.ElapsedMilliseconds)ms".PadLeft(7)
    $lbl = $label.PadRight(25)
    if ($f -gt 0) { Write-Host "  $lbl FAIL $elapsed ($p/$($p+$f))" -ForegroundColor Red }
    elseif ($p -gt 0) { Write-Host "  $lbl OK   $elapsed ($p)" -ForegroundColor Green }
    else { Write-Host "  $lbl NONE $elapsed" -ForegroundColor Yellow }
    
    $script:totalPassed += $p
    $script:totalFailed += $f
    $script:totalIgnored += $i
}

# Build phase
Write-Host "`nXIOM Fast Test Suite" -ForegroundColor Magenta
Write-Host "======================" -ForegroundColor Magenta
Write-Host ""
Write-Host "BUILD (parallel)..." -ForegroundColor Yellow -NoNewline
$sw = [System.Diagnostics.Stopwatch]::StartNew()
cargo test --workspace --no-run 2>&1 | Out-Null
$sw.Stop()
if ($LASTEXITCODE -ne 0) {
    Write-Host " FAILED" -ForegroundColor Red
    cargo test --workspace --no-run 2>&1 | Select-Object -Last 10
    exit 1
}
Write-Host " OK ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Green
Write-Host ""

# Run phase
Write-Host "RUN ($Threads threads) " -ForegroundColor Yellow -NoNewline
Write-Host (Get-Date -Format 'HH:mm:ss')

# Units
Write-Host " [UNITS]" -ForegroundColor Cyan
Test-Crate "xiom-lexer"   $null "lexer"
Test-Crate "xiom-parser"  $null "parser"
Test-Crate "xiom-check"   $null "checker"
Test-Crate "xiom-ctfe"    $null "ctfe"
Test-Crate "xiom-graph"   $null "graph"
Test-Crate "xiom-codegen" $null "codegen-unit"
Test-Crate "xiom-verify"  "verifier_tests" "verifier"
Test-Crate "xiom-jit"     $null "jit"

# Tooling
Write-Host " [TOOLS]" -ForegroundColor Cyan
Test-Crate "xiom-fmt"     $null "formatter"
Test-Crate "xiom-lsp"     $null "lsp"
Test-Crate "xiom-pkg"     $null "package-mgr"
Test-Crate "xiom-doc"     $null "doc-gen"
Test-Crate "xiom-ffigen"  $null "ffi-gen"
Test-Crate "xiom-mcp"     $null "mcp-server"
Test-Crate "xiom-dbg"     $null "debugger"
Test-Crate "xiom-display" $null "display"
Test-Crate "xiom" "scripting_tests" "scripting"
Test-Crate "xiom" "diff_tests" "script-diff"

# Suites
Write-Host " [SUITES]" -ForegroundColor Cyan
Test-Crate "xiom-codegen" "integration_tests" "integration"
Test-Crate "xiom-codegen" "diff_tests" "diff"
if (-not $Fast) { Test-Crate "xiom-codegen" "full_diff_tests" "full-diff" }
if (-not $Fast) { Test-Crate "xiom-codegen" "fuzz_tests" "fuzz" }
Test-Crate "xiom-codegen" "robustness_tests" "robustness"
Test-Crate "xiom-codegen" "stdlib_execution_tests" "stdlib-exec"
Test-Crate "xiom-codegen" "stdlib_tests" "stdlib-compile"
if (-not $Fast) { Test-Crate "xiom-codegen" "feature_regression_tests" "feature-reg" }

# E2E
if (-not $Fast) {
    Write-Host " [E2E]" -ForegroundColor Cyan
    if ($E2EOnly) {
        Test-Crate "xiom-codegen" "e2e_tests" "e2e-gate" @("e2e_p0","e2e_p1","e2e_p2","e2e_never_type","e2e_asm","e2e_spawn_basic","e2e_i2")
    } else {
        Test-Crate "xiom-codegen" "e2e_tests" "e2e (all 2231)"
    }
} else {
    Write-Host " [E2E] SKIPPED (--fast)" -ForegroundColor Yellow
}

# Totals
$elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)
Write-Host ""
Write-Host "=================================" -ForegroundColor Magenta
Write-Host "  TOTAL: $($totalPassed + $totalFailed) tests in ${elapsed}s" -ForegroundColor Cyan
if ($totalFailed -eq 0) {
    Write-Host "  $totalPassed PASSED" -ForegroundColor Green
} else {
    Write-Host "  $totalPassed passed, $totalFailed FAILED" -ForegroundColor Red
}
if ($totalIgnored -gt 0) { Write-Host "  ($totalIgnored ignored)" -ForegroundColor Yellow }
Write-Host "=================================" -ForegroundColor Magenta
if ($totalFailed -gt 0) { exit 1 } else { exit 0 }
