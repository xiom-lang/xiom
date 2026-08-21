#Requires -Version 5.1
<#
.SYNOPSIS XIOM Test Suite -- run all tests with progress, logs, and summary.
.DESCRIPTION
  .\test_summary.ps1               # Full suite
  .\test_summary.ps1 -Fast          # Skip E2E/full-diff/fuzz
  .\test_summary.ps1 -E2EOnly       # Just 13 gate tests
  .\test_summary.ps1 -Threads 16    # More parallelism
  .\test_summary.ps1 -Logs          # Write .testlogs/session_*.txt
  .\test_summary.ps1 -CleanBuild    # Delete .test_build/*  
  .\test_summary.ps1 -CleanLogs     # Delete .testlogs/*
#>
param([switch]$Fast, [int]$Threads=8, [switch]$E2EOnly, [switch]$Logs, [switch]$CleanBuild, [switch]$CleanLogs)

$ErrorActionPreference = "Continue"
if ($CleanBuild) { if (Test-Path ".test_build") { Remove-Item ".test_build\*" -Recurse -Force -EA 0 }; "Cleaned .test_build"; exit 0 }
if ($CleanLogs)  { if (Test-Path ".testlogs")  { Remove-Item ".testlogs\*"  -Recurse -Force -EA 0 }; "Cleaned .testlogs";  exit 0 }

$null = New-Item -ItemType Directory -Force ".test_build", ".testlogs" -EA 0
$startTime = Get-Date; $sid = $startTime.ToString("yyyyMMdd_HHmmss")
$timings = @(); $totalPassed=0; $totalFailed=0; $totalIgnored=0

function log($m) { if ($Logs) { Add-Content ".testlogs\session_$sid.txt" "[$(Get-Date -Format 'HH:mm:ss.fff')] $m" } }

function run-test($pkg, $testFile, $label, $extraFilter) {
    $tmpO = "$env:TEMP\xt_o.txt"; $tmpE = "$env:TEMP\xt_e.txt"; Remove-Item $tmpO,$tmpE -EA 0
    
    $cargs = @("test","-p",$pkg,"--target-dir",(Resolve-Path ".test_build"))
    if ($testFile -eq "--lib") { $cargs += "--lib" }
    elseif ($testFile) { $cargs += "--test",$testFile }
    if ($extraFilter) { $cargs += $extraFilter }
    $cargs += "--","--test-threads=$Threads"
    log "cargo $cargs"
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $proc = Start-Process cargo -Arg $cargs -NoNewWindow -PassThru -RedirectStandardOutput $tmpO -RedirectStandardError $tmpE
    $total=0; $passed=0; $failed=0; $crashed=0; $lr=0; $tc=$false; $lu=$sw.Elapsed
    
    while (-not $proc.HasExited) {
        Start-Sleep -Milliseconds 200
        if ($sw.Elapsed.TotalSeconds -gt 1200) { $proc.Kill(); $crashed=2; break }  # 20min timeout
        if (Test-Path $tmpO) {
            $lines = Get-Content $tmpO -EA 0
            for ($i=$lr; $i -lt $lines.Count; $i++) {
                $l = $lines[$i]
                if ($l -match '^running (\d+) tests?$') { $total=[int]$Matches[1]; $tc=$true }
                elseif ($l -match '^test \S+ \.\.\. ok\s*$') { $passed++ }
                elseif ($l -match '^test \S+ \.\.\. FAILED\s*$') { $failed++ }
            }
            $lr = $lines.Count
        }
        if (Test-Path $tmpE) {
            $el = Get-Content $tmpE -EA 0 -Raw
            if ($el -match 'STATUS_STACK_OVERFLOW|STATUS_ACCESS_VIOLATION|STATUS_ILLEGAL') { $crashed=1 }
        }
        $fin = $passed+$failed; $now = $sw.Elapsed
        if ($tc -and $total -gt 0 -and ($now - $lu).TotalSeconds -gt 0.4) {
            $pct = [math]::Round($fin*100/$total); $bar=""
            for ($i=0;$i -lt 20;$i++) { $bar += if ($i*5 -lt $pct){"="}else{" "} }
            Write-Host ("`r  [{0}] {1}/{2} ({3}ok {4}fail)   " -f $bar,$fin,$total,$passed,$failed) -NoNewline
            $lu=$now
        }
    }
    $proc.WaitForExit()
    $sw.Stop()
    $so = if (Test-Path $tmpO) { Get-Content $tmpO -Raw -EA 0 } else {""}
    $se = if (Test-Path $tmpE) { Get-Content $tmpE -Raw -EA 0 } else {""}
    Remove-Item $tmpO,$tmpE -EA 0
    Write-Host ("`r"+" "*80+"`r") -NoNewline
    
    $elapsed = $sw.Elapsed.TotalSeconds
    $es = if ($elapsed -lt 1){"$([math]::Round($elapsed*1000))ms"}else{"$([math]::Round($elapsed,1))s"}
    
    # Handle timeout
    if ($crashed -eq 2) {
        Write-Host "  $($label.PadRight(25)) HANG  (timeout 20min)" -ForegroundColor Magenta
        $script:timings += @{L=$label;P=0;F=0;I=0;E=$elapsed}
        return
    }
    
    # Parse result from both stdout and stderr
    $p=0; $f=0; $i=0; $crash=$false
    $all = ($so + "`n" + $se)
    if ($all -match 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
        $p=[int]$Matches[1]; $f=[int]$Matches[2]; $i=[int]$Matches[3]
    } elseif ($all -match '(\d+) passed;\s*(\d+) failed') {
        $p=[int]$Matches[1]; $f=[int]$Matches[2]
    } elseif ($se -match 'STATUS_STACK_OVERFLOW|STATUS_ACCESS_VIOLATION|STATUS_ILLEGAL') {
        $crash=$true; $crashName = if ($se -match 'STATUS_(\w+)') {$Matches[1]} else {"CRASH"}
        $f = ([regex]::Matches($so, '\.\.\. FAILED')).Count
        $p = ([regex]::Matches($so, '\.\.\. ok')).Count
    }
    
    if ($crash) { Write-Host "  $($label.PadRight(25)) CRASH $crashName ($p ok before crash)" -ForegroundColor Magenta }
    elseif ($f -gt 0) { Write-Host "  $($label.PadRight(25)) FAIL ${es} ($p/$($p+$f))" -ForegroundColor Red }
    elseif ($p -gt 0) { Write-Host "  $($label.PadRight(25)) OK   ${es} ($p)" -ForegroundColor Green }
    else { Write-Host "  $($label.PadRight(25)) NONE ${es}" -ForegroundColor Yellow }
    
    $script:totalPassed+=$p; $script:totalFailed+=$f; $script:totalIgnored+=$i
    $script:timings += @{L=$label;P=$p;F=$f;I=$i;E=$elapsed}
    log "END $label p=$p f=$f i=$i crash=$crash"
    if ($f -gt 0 -and $Logs) {
        $fp = ".testlogs\failures_$sid.txt"
        Add-Content $fp "=== $label ($p/$($p+$f)) ==="
        ($so -split "`n"|?{$_ -match "FAILED"}) | %{ Add-Content $fp $_ }
        Add-Content $fp ""
    }
}

# ---- Build ----
Write-Host "`nXIOM Test Suite" -ForegroundColor Magenta
Write-Host "===============" -ForegroundColor Magenta
Write-Host ""
Write-Host "BUILD (parallel)..." -ForegroundColor Yellow -NoNewline
$sw=[System.Diagnostics.Stopwatch]::StartNew()
$td = (Resolve-Path ".test_build").Path
cargo test --workspace --no-run --target-dir $td 2>&1 | Out-Null
$sw.Stop()
if ($LASTEXITCODE) { Write-Host " FAILED" -ForegroundColor Red; exit 1 }
Write-Host " OK ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Green
Write-Host ""

# ---- Run ----
Write-Host "RUN - $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow

Write-Host " [UNITS]" -ForegroundColor Cyan
run-test "xiom-lexer"   $null "lexer"
run-test "xiom-parser"  $null "parser"
run-test "xiom-check"   $null "checker"
run-test "xiom-ctfe"    $null "ctfe"
run-test "xiom-graph"   $null "graph"
run-test "xiom"         "--lib" "xiom-lib (19)"
run-test "xiom-codegen" "--lib" "codegen-unit"
run-test "xiom-verify"  "verifier_tests" "verifier"
run-test "xiom-jit"     $null "jit"

Write-Host " [TOOLS]" -ForegroundColor Cyan
run-test "xiom-fmt"     $null "formatter"
run-test "xiom-lsp"     $null "lsp"
run-test "xiom-pkg"     $null "package-mgr"
run-test "xiom-doc"     $null "doc-gen"
run-test "xiom-ffigen"  $null "ffi-gen"
run-test "xiom-mcp"     $null "mcp-server"
run-test "xiom-dbg"     $null "debugger"
run-test "xiom-display" $null "display"
run-test "xiom" "scripting_tests" "scripting"
run-test "xiom" "diff_tests" "script-diff"

Write-Host " [SUITES]" -ForegroundColor Cyan
run-test "xiom-codegen" "integration_tests" "integration"
run-test "xiom-codegen" "diff_tests" "diff"
if (-not $Fast) { run-test "xiom-codegen" "full_diff_tests" "full-diff" }
if (-not $Fast) { run-test "xiom-codegen" "fuzz_tests" "fuzz" }
run-test "xiom-codegen" "robustness_tests" "robustness"
run-test "xiom-codegen" "stdlib_execution_tests" "stdlib-exec"
run-test "xiom-codegen" "stdlib_tests" "stdlib-compile"
if (-not $Fast) { run-test "xiom-codegen" "feature_regression_tests" "feature-reg" }

if (-not $Fast) {
    Write-Host " [E2E]" -ForegroundColor Cyan
    if ($E2EOnly) {
        run-test "xiom-codegen" "e2e_tests" "e2e-gate" @("e2e_p0","e2e_p1","e2e_p2","e2e_never_type","e2e_asm","e2e_spawn_basic","e2e_i2")
    } else {
        run-test "xiom-codegen" "e2e_tests" "e2e (all 2231)"
    }
} else { Write-Host " [E2E] SKIPPED" -ForegroundColor Yellow }

# ---- Summary ----
$tt = [math]::Round(((Get-Date)-$startTime).TotalSeconds,1); $tot = $totalPassed+$totalFailed
Write-Host ""
Write-Host "============================================" -ForegroundColor Magenta
Write-Host "  Time:   ${tt}s  |  Passed: $totalPassed  |  Failed: $totalFailed  |  Total: $tot" -ForegroundColor White
if ($totalIgnored -gt 0) { Write-Host "  Ignored: $totalIgnored" -ForegroundColor Yellow }
Write-Host "============================================" -ForegroundColor Magenta
log "SUMMARY p=$totalPassed f=$totalFailed i=$totalIgnored t=$tt"
if ($totalFailed -gt 0) {
    Write-Host "`nFAILURES:" -ForegroundColor Red
    foreach ($r in $timings) { if ($r.F -gt 0) { Write-Host "  $($r.L): $($r.F) failed / $($r.P+$r.F) total" -ForegroundColor Red } }
    if ($Logs) { Write-Host "  Logs: .testlogs\failures_$sid.txt" -ForegroundColor Yellow }
}
if ($Logs) { Write-Host "  Session: .testlogs\session_$sid.txt" -ForegroundColor Yellow }
if ($totalFailed -gt 0) { exit 1 } else { exit 0 }
