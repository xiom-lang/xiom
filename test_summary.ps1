#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Comprehensive Test Suite v3 — progress bars, logs, cleanup.
.DESCRIPTION
    .\test_summary.ps1                 # full suite with progress + logs
    .\test_summary.ps1 -Fast            # skip E2E/full-diff/fuzz
    .\test_summary.ps1 -E2EOnly         # just 13 core gate tests
    .\test_summary.ps1 -Threads 16      # more parallelism
    .\test_summary.ps1 -Logs            # write .testlogs/session_*.log
    .\test_summary.ps1 -CleanBuild      # delete .test_build/ contents
    .\test_summary.ps1 -CleanLogs       # delete .testlogs/ contents
#>
param(
    [switch]$Fast,
    [int]$Threads = 8,
    [switch]$E2EOnly,
    [switch]$Logs,
    [switch]$CleanBuild,
    [switch]$CleanLogs
)

$ErrorActionPreference = "Continue"

# ── Cleanup modes ───────────────────────────────────────────────────
if ($CleanBuild) {
    Write-Host "Cleaning .test_build/..." -ForegroundColor Yellow
    if (Test-Path ".test_build") {
        Remove-Item ".test_build\*" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  Done." -ForegroundColor Green
    }
    exit 0
}
if ($CleanLogs) {
    Write-Host "Cleaning .testlogs/..." -ForegroundColor Yellow
    if (Test-Path ".testlogs") {
        Remove-Item ".testlogs\*" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  Done." -ForegroundColor Green
    }
    exit 0
}

# ── Setup ───────────────────────────────────────────────────────────
$BuildDir = ".test_build"
$LogDir = ".testlogs"
if (-not (Test-Path $BuildDir)) { New-Item -ItemType Directory -Force $BuildDir | Out-Null }
if (-not (Test-Path $LogDir))   { New-Item -ItemType Directory -Force $LogDir   | Out-Null }

$startTime = Get-Date
$sessionId = $startTime.ToString("yyyyMMdd_HHmmss")
$logPath = "$LogDir\session_$sessionId.txt"
$totalPassed = 0; $totalFailed = 0; $totalIgnored = 0
$allResults = @()
$global:suiteNum = 0; $global:suiteTotal = 0

# ── Logging ─────────────────────────────────────────────────────────
function Write-Log($msg) {
    $ts = (Get-Date -Format 'HH:mm:ss.fff')
    $line = "[$ts] $msg"
    if ($Logs) { Add-Content -Path $logPath -Value $line }
}

# ── Test crate function ─────────────────────────────────────────────
function Test-Crate($pkg, $testFile, $label, $extraFilter) {
    $global:suiteNum++
    $suiteLabel = $label.PadRight(25)
    
    $cargs = @("test", "-p", $pkg, "--target-dir", (Resolve-Path $BuildDir))
    if ($testFile) { $cargs += "--test", $testFile }
    if ($extraFilter) { $cargs += $extraFilter }
    $cargs += "--", "--test-threads=$Threads"
    
    Write-Log "START $label (pkg=$pkg test=$testFile)"
    
    # Run and capture output line-by-line for progress
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "cargo"
    $psi.Arguments = [string]::Join(" ", $cargs)
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    $psi.WorkingDirectory = (Get-Location).Path
    
    $process = [System.Diagnostics.Process]::Start($psi)
    $stdout = $process.StandardOutput
    $stderr = $process.StandardError
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $passed = 0; $failed = 0; $total = 0
    $testCountKnown = $false
    $lastUpdate = 0
    $outputLines = @()
    
    while (-not $process.HasExited -or -not $stdout.EndOfStream) {
        while ($stdout.Peek() -ge 0) {
            $line = $stdout.ReadLine()
            $outputLines += $line
            
            if ($line -match '^running (\d+) tests?') {
                $total = [int]$Matches[1]
                $testCountKnown = $true
            }
            if ($line -match '^test (?!result:)(\S+) \.\.\. (ok|FAILED)$') {
                if ($Matches[1] -ne 'result:') {
                    if ($Matches[2] -eq 'ok') { $passed++ } else { $failed++ }
                }
            }
        }
        
        $finished = $passed + $failed
        $now = $sw.Elapsed.TotalSeconds
        if ($now - $lastUpdate -ge 0.3) {
            $pct = if ($testCountKnown -and $total -gt 0) { [math]::Round($finished * 100 / $total) } else { 0 }
            $bar = ""; for ($i = 0; $i -lt 20; $i++) { $bar += if ($i * 5 -lt $pct) { "=" } else { " " } }
            $rate = if ($now -gt 0) { [math]::Round($finished / $now, 1) } else { 0 }
            Write-Host ("`r  ${suiteLabel}[${bar}] {0}/{1} ({2} pass, {3} fail) {4}s {5}t/s  " -f $finished, $(if($testCountKnown){$total}else{'?'}), $passed, $failed, [math]::Round($now,1), $rate) -NoNewline
            $lastUpdate = $now
        }
        Start-Sleep -Milliseconds 50
    }
    $process.WaitForExit()
    $sw.Stop()
    
    # Parse final result from output
    $outputText = $outputLines -join "`n"
    if ($outputText -match 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
        $passed = [int]$Matches[1]; $failed = [int]$Matches[2]; $ignored = [int]$Matches[3]
    }
    
    $elapsed = $sw.Elapsed.TotalSeconds
    $elapsedStr = if ($elapsed -lt 1) { "$([math]::Round($elapsed*1000))ms" } else { "$([math]::Round($elapsed,1))s" }
    
    # Clear progress line
    Write-Host ("`r" + " " * 100 + "`r") -NoNewline
    
    # Show result
    if ($failed -gt 0) {
        Write-Host "  ${suiteLabel} FAIL ${elapsedStr} ($passed/$($passed+$failed))" -ForegroundColor Red
    } elseif ($passed -gt 0) {
        Write-Host "  ${suiteLabel} OK   ${elapsedStr} ($passed)" -ForegroundColor Green
    } else {
        Write-Host "  ${suiteLabel} NONE  ${elapsedStr}" -ForegroundColor Yellow
    }
    
    $result = @{
        Label = $label; Passed = $passed; Failed = $failed; 
        Ignored = $ignored; Elapsed = $elapsed
    }
    $script:allResults += $result
    $script:totalPassed += $passed
    $script:totalFailed += $failed
    $script:totalIgnored += $ignored
    
    Write-Log "END $label — $passed passed, $failed failed, $ignored ignored in ${elapsedStr}"
    
    # Log failures
    if ($failed -gt 0 -and $Logs) {
        $failPath = "$LogDir\failures_$sessionId.txt"
        $failLines = $outputLines | Select-String "FAILED" | ForEach-Object { $_.Line }
        Add-Content -Path $failPath -Value "=== $label ==="
        Add-Content -Path $failPath -Value $failLines
        Add-Content -Path $failPath -Value ""
    }
    
    return $result
}

# ── Build phase ─────────────────────────────────────────────────────
Write-Host "`nXIOM Test Suite v3" -ForegroundColor Magenta
Write-Host "===================" -ForegroundColor Magenta
Write-Host ""
Write-Host "BUILD (parallel, all crates)..." -ForegroundColor Yellow -NoNewline
Write-Log "BUILD START"
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$buildOut = cargo test --workspace --no-run --target-dir "$(Resolve-Path $BuildDir)" 2>&1 | Out-String
$sw.Stop()
if ($LASTEXITCODE -ne 0) {
    Write-Host " FAILED ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Red
    Write-Host $buildOut | Select-Object -Last 10
    Write-Log "BUILD FAILED"
    exit 1
}
Write-Host " OK ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Green
Write-Log "BUILD OK $([math]::Round($sw.Elapsed.TotalSeconds,1))s"
Write-Host ""

# ── Run phase ───────────────────────────────────────────────────────
$runStart = Get-Date
Write-Host "RUN ($($Threads) threads) — $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow
Write-Log "RUN START (threads=$Threads)"

$global:suiteTotal = if ($Fast) { 21 } else { 26 }

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
    Write-Host " [E2E] SKIPPED (-fast)" -ForegroundColor Yellow
}

$runEnd = Get-Date
$runElapsed = ($runEnd - $runStart).TotalSeconds

# ── Totals ──────────────────────────────────────────────────────────
$totalElapsed = [math]::Round(($runEnd - $startTime).TotalSeconds, 1)
Write-Host ""
Write-Host "============================================" -ForegroundColor Magenta
Write-Host "  TEST SUMMARY" -ForegroundColor Cyan
Write-Host "  Time:    ${totalElapsed}s total" -ForegroundColor White
Write-Host "  Passed:  $totalPassed" -ForegroundColor Green
if ($totalFailed -gt 0) {
    Write-Host "  Failed:  $totalFailed" -ForegroundColor Red
} else {
    Write-Host "  Failed:  0" -ForegroundColor Green
}
if ($totalIgnored -gt 0) {
    Write-Host "  Ignored: $totalIgnored" -ForegroundColor Yellow
}
Write-Host "  Total:   $($totalPassed + $totalFailed)" -ForegroundColor White
Write-Host "============================================" -ForegroundColor Magenta

Write-Log "SUMMARY: $totalPassed passed, $totalFailed failed, $totalIgnored ignored in ${totalElapsed}s"
Write-Log "RUN END"

# Show failures detail
if ($totalFailed -gt 0) {
    Write-Host ""
    Write-Host "FAILURES:" -ForegroundColor Red
    foreach ($r in $allResults) {
        if ($r.Failed -gt 0) {
            Write-Host "  $($r.Label): $($r.Failed) failed / $($r.Passed + $r.Failed) total" -ForegroundColor Red
        }
    }
    if ($Logs) {
        Write-Host "  Full failure logs: $LogDir\failures_$sessionId.txt" -ForegroundColor Yellow
    }
}

if ($Logs) {
    Write-Host "  Session log: $logPath" -ForegroundColor Yellow
}

if ($totalFailed -gt 0) { exit 1 } else { exit 0 }
