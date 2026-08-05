#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Fast Test Suite — up to 3x faster than sequential.
.DESCRIPTION
    .\test_summary.ps1               # full suite with parallel threads
    .\test_summary.ps1 -Fast          # skip E2E/full-diff/fuzz (quick)
    .\test_summary.ps1 -E2EOnly       # only core E2E gate tests
    .\test_summary.ps1 -Threads 4     # control parallelism (default 8)
#>
param(
    [switch]$Fast,
    [int]$Threads = 8,
    [switch]$E2EOnly
)

$ErrorActionPreference = "Continue"
$startTime = Get-Date
$totalPassed = 0; $totalFailed = 0; $totalIgnored = 0

function Run-Suite($pkg, $test, $label, $extraArgs) {
    $cargs = @("test", "-p", $pkg)
    if ($test) { $cargs += "--test", $test }
    if ($extraArgs) { $cargs += $extraArgs }
    $cargs += "--", "--test-threads=$Threads"
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & cargo @cargs 2>&1 | Out-String
    $sw.Stop()
    
    $passed = 0; $failed = 0; $ignored = 0
    if ($output -match 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
        $passed = [int]$Matches[1]; $failed = [int]$Matches[2]; $ignored = [int]$Matches[3]
    }
    
    $elapsed = "$($sw.ElapsedMilliseconds)ms".PadLeft(7)
    $labelPadded = $label.PadRight(30)
    if ($failed -gt 0) {
        Write-Host "  ${labelPadded} FAIL ${elapsed} ($passed/$($passed+$failed))" -ForegroundColor Red
    } elseif ($passed -gt 0) {
        Write-Host "  ${labelPadded} OK   ${elapsed} ($passed)" -ForegroundColor Green
    } else {
        Write-Host "  ${labelPadded} NONE ${elapsed}" -ForegroundColor Yellow
    }
    return @{ Passed = $passed; Failed = $failed; Ignored = $ignored }
}

# ── build all test binaries in one parallel pass ─────────────────────
Write-Host "`nXIOM Fast Test Suite" -ForegroundColor Magenta
Write-Host "======================" -ForegroundColor Magenta
Write-Host ""
Write-Host "BUILD (parallel, all crates)..." -ForegroundColor Yellow -NoNewline
$sw = [System.Diagnostics.Stopwatch]::StartNew()
cargo test --workspace --no-run 2>&1 | Out-Null
$sw.Stop()
if ($LASTEXITCODE -ne 0) {
    Write-Host " FAILED ($($sw.ElapsedMilliseconds)ms)" -ForegroundColor Red
    cargo test --workspace --no-run 2>&1 | Select-Object -Last 10
    exit 1
}
Write-Host " OK ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Green
Write-Host ""

# ── run all suites ───────────────────────────────────────────────────
Write-Host "RUN (${Threads} threads) — $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow

# Core unit tests (each <5s)
Write-Host "  [UNITS]" -ForegroundColor Cyan
$r = Run-Suite "xiom-lexer"   $null "lexer";             $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-parser"  $null "parser"   @("--test-threads=4");$totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-check"   $null "checker"  @("--test-threads=4");$totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-ctfe"    $null "ctfe";               $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-graph"   $null "graph";              $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-codegen" $null "codegen-unit";       $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-verify"  "verifier_tests" "verifier";$totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-jit"     $null "jit";                $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored

# Tooling tests
Write-Host "  [TOOLS]" -ForegroundColor Cyan
$r = Run-Suite "xiom-fmt"     $null "formatter";       $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-lsp"     $null "lsp";             $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-pkg"     $null "package-mgr";     $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-doc"     $null "doc-gen";         $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-ffigen"  $null "ffi-gen";         $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-mcp"     $null "mcp-server";      $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-dbg"     $null "debugger";        $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-display" $null "display";         $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom" "scripting_tests" "scripting";   $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom" "diff_tests" "script-diff";      $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored

# Medium test suites (run with full thread count)
Write-Host "  [SUITES]" -ForegroundColor Cyan
$r = Run-Suite "xiom-codegen" "integration_tests" "integration";        $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-codegen" "diff_tests" "diff";                      $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
if (-not $Fast) { $r = Run-Suite "xiom-codegen" "full_diff_tests" "full-diff";$totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored }
if (-not $Fast) { $r = Run-Suite "xiom-codegen" "fuzz_tests" "fuzz";         $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored }
$r = Run-Suite "xiom-codegen" "robustness_tests" "robustness";           $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-codegen" "stdlib_execution_tests" "stdlib-exec";    $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
$r = Run-Suite "xiom-codegen" "stdlib_tests" "stdlib-compile";           $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
if (-not $Fast) { $r = Run-Suite "xiom-codegen" "feature_regression_tests" "feature-reg";$totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored }

# E2E tests — biggest suite, most benefit from --test-threads
if (-not $Fast) {
    Write-Host "  [E2E]" -ForegroundColor Cyan
    if ($E2EOnly) {
        $r = Run-Suite "xiom-codegen" "e2e_tests" "e2e-gate" @("e2e_p0","e2e_p1","e2e_p2","e2e_never_type","e2e_asm","e2e_spawn_basic","e2e_i2")
        $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
    } else {
        $r = Run-Suite "xiom-codegen" "e2e_tests" "e2e (all 2231)"
        $totalPassed+=$r.Passed;$totalFailed+=$r.Failed;$totalIgnored+=$r.Ignored
    }
} else {
    Write-Host "  [E2E] SKIPPED (--fast)" -ForegroundColor Yellow
}

# ── totals ───────────────────────────────────────────────────────────
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
exit (if ($totalFailed -gt 0) { 1 } else { 0 })
