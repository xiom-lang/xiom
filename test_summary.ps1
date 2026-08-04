#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Full Test Suite with realtime progress counters.
.DESCRIPTION
    Run: .\test_summary.ps1
    Shows live test counts as they pass/fail instead of waiting silently.
#>

$ErrorActionPreference = "Continue"

function Run-Suite($pkg, $test, $label, $extraArgs) {
    $labelPadded = $label.PadRight(22)
    Write-Host "  ${labelPadded}" -NoNewline
    $args = @("test", "-p", $pkg)
    if ($test) { $args += "--test", $test }
    if ($extraArgs) { $args += $extraArgs }

    $passed = 0; $failed = 0; $ignored = 0; $finished = 0
    $total = 0
    $lastLine = ""
    $lineCount = 0

    # Run cargo test with line-by-line output streaming
    $stdoutFile = "$env:TEMP\xiom_test_stdout.txt"
    $stderrFile = "$env:TEMP\xiom_test_stderr.txt"
    Remove-Item $stdoutFile, $stderrFile -ErrorAction SilentlyContinue
    $process = Start-Process -FilePath "cargo" -ArgumentList $args -NoNewWindow -PassThru -RedirectStandardOutput $stdoutFile -RedirectStandardError $stderrFile
    
    $lastUpdate = 0
    $testCountKnown = $false
    
    while (-not $process.HasExited) {
        Start-Sleep -Milliseconds 200
        if (Test-Path $stdoutFile) {
            $lines = Get-Content $stdoutFile -ErrorAction SilentlyContinue
            $totalLines = $lines.Count
            # Only count NEW lines since last poll
            for ($i = $lineCount; $i -lt $totalLines; $i++) {
                $line = $lines[$i]
                if ($line -match '^test .*\.\.\. ok$') { $passed++ }
                if ($line -match '^test .*\.\.\. FAILED$') { $failed++ }
                if (-not $testCountKnown -and $line -match 'running (\d+) tests?') {
                    $total = [int]$Matches[1]
                    $testCountKnown = $true
                }
                $lineCount++
            }
            $finished = $passed + $failed
            # Update progress line every 500ms
            $now = (Get-Date).Ticks / 10000000
            if ($now - $lastUpdate -gt 0.5) {
                $pct = if ($total -gt 0) { [math]::Round($finished * 100 / $total) } else { 0 }
                $bar = ""
                for ($i = 0; $i -lt 20; $i++) { $bar += if ($i * 5 -lt $pct) { "=" } else { " " } }
                Write-Host "`r  ${labelPadded}[${bar}] ${finished}/$total ($passed pass, $failed fail)   " -NoNewline
                $lastUpdate = $now
            }
        }
    }
    $process.WaitForExit()

    # Parse final result from the file
    if (Test-Path $stdoutFile) {
        $final = Get-Content $stdoutFile -Raw -ErrorAction SilentlyContinue
        if ($final -match 'test result: (\w+)\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
            $passed = [int]$Matches[2]
            $failed = [int]$Matches[3]
            $ignored = [int]$Matches[4]
        }
    }

    # Clear progress line and show final
    Write-Host "`r  ${labelPadded}" -NoNewline
    if ($failed -gt 0) {
        Write-Host "FAIL ($passed/$($passed + $failed) passed)" -ForegroundColor Red
        $global:totalFailed += $failed
    } elseif ($passed -gt 0) {
        Write-Host " OK  ($passed passed)" -ForegroundColor Green
    } else {
        Write-Host "CRASH" -ForegroundColor Red
        $global:failedSuites += "$label (no result)"
    }
    $global:totalPassed += $passed
    $global:totalIgnored += $ignored
    
    Remove-Item $stdoutFile, $stderrFile -ErrorAction SilentlyContinue
}

$global:totalPassed = 0
$global:totalFailed = 0
$global:totalIgnored = 0
$global:failedSuites = @()

# ============================================================================
# COMPILER SUITES
# ============================================================================
$compilerPassed = 0; $compilerFailed = 0; $compilerIgnored = 0

Write-Host ""
Write-Host "XIOM Test Suite (realtime)" -ForegroundColor Magenta
Write-Host "===========================" -ForegroundColor Magenta
Write-Host ""
Write-Host "COMPILER ($(Get-Date -Format 'HH:mm:ss'))" -ForegroundColor Yellow

$compiler = @(
    @{pkg="xiom-codegen"; test="e2e_tests";                 label="e2e (2231 tests)";            extra=$null},
    @{pkg="xiom-codegen"; test="feature_regression_tests";   label="feature-regression (491)";     extra=$null},
    @{pkg="xiom-codegen"; test="stdlib_execution_tests";     label="stdlib-execution";             extra=$null},
    @{pkg="xiom-codegen"; test="stdlib_tests";               label="stdlib-compile";               extra=$null},
    @{pkg="xiom-codegen"; test="integration_tests";          label="integration (128)";            extra=$null},
    @{pkg="xiom-codegen"; test="diff_tests";                 label="diff";                         extra=$null},
    @{pkg="xiom-codegen"; test="full_diff_tests";            label="full-diff";                    extra=$null},
    @{pkg="xiom-codegen"; test="fuzz_tests";                 label="fuzz";                         extra=$null},
    @{pkg="xiom-codegen"; test="robustness_tests";           label="robustness";                   extra=$null},
    @{pkg="xiom-codegen"; test=$null;                        label="codegen-unit";                 extra=$null},
    @{pkg="xiom-lexer";   test=$null;                        label="lexer";                        extra=$null},
    @{pkg="xiom-parser";  test=$null;                        label="parser";                       extra=@("--","--test-threads=2")},
    @{pkg="xiom-check";   test=$null;                        label="checker";                      extra=@("--","--test-threads=2")},
    @{pkg="xiom-ctfe";    test=$null;                        label="ctfe";                         extra=$null},
    @{pkg="xiom-graph";   test=$null;                        label="graph";                        extra=$null},
    @{pkg="xiom-verify";  test="verifier_tests";             label="verifier";                     extra=$null},
    @{pkg="xiom-jit";     test=$null;                        label="jit";                          extra=$null},
    @{pkg="xiom";         test="scripting_tests";            label="scripting (34)";               extra=$null},
    @{pkg="xiom";         test="diff_tests";                 label="script-diff (15)";             extra=$null}
)

$global:totalPassed = 0; $global:totalFailed = 0; $global:totalIgnored = 0
foreach ($s in $compiler) {
    Run-Suite $s.pkg $s.test $s.label $s.extra
}
$compilerPassed = $global:totalPassed
$compilerFailed = $global:totalFailed
$compilerIgnored = $global:totalIgnored
$compilerTotal = $compilerPassed + $compilerFailed

# ============================================================================
# TOOLING SUITES
# ============================================================================
$toolingPassed = 0; $toolingFailed = 0; $toolingIgnored = 0

Write-Host ""
Write-Host "TOOLING ($(Get-Date -Format 'HH:mm:ss'))" -ForegroundColor Yellow

$tooling = @(
    @{pkg="xiom-fmt";      test=$null; label="formatter"},
    @{pkg="xiom-lsp";      test=$null; label="lsp"},
    @{pkg="xiom-pkg";      test=$null; label="package-mgr"},
    @{pkg="xiom-doc";      test=$null; label="doc-gen"},
    @{pkg="xiom-ffigen";   test=$null; label="ffi-gen"},
    @{pkg="xiom-mcp";      test=$null; label="mcp-server"},
    @{pkg="xiom-dbg";      test=$null; label="debugger"},
    @{pkg="xiom-display";  test=$null; label="display"}
)

$global:totalPassed = 0; $global:totalFailed = 0; $global:totalIgnored = 0
foreach ($s in $tooling) {
    Run-Suite $s.pkg $s.test $s.label $null
}
$toolingPassed = $global:totalPassed
$toolingFailed = $global:totalFailed
$toolingIgnored = $global:totalIgnored
$toolingTotal = $toolingPassed + $toolingFailed

# ============================================================================
# TOTALS
# ============================================================================
$grandPassed  = $compilerPassed  + $toolingPassed
$grandFailed  = $compilerFailed  + $toolingFailed
$grandIgnored = $compilerIgnored + $toolingIgnored
$grandTotal   = $compilerTotal   + $toolingTotal

Write-Host ""
Write-Host "=============================================" -ForegroundColor Magenta
Write-Host "  COMPILER  $compilerPassed/$compilerTotal" -ForegroundColor $(if ($compilerFailed -eq 0){"Green"}else{"Red"})
Write-Host "  TOOLING   $toolingPassed/$toolingTotal"  -ForegroundColor $(if ($toolingFailed -eq 0){"Green"}else{"Red"})
Write-Host "  ----------------------------------------"
if ($grandFailed -eq 0 -and $global:failedSuites.Count -eq 0) {
    Write-Host "  ALL $grandTotal TESTS PASSED" -ForegroundColor Green
    Write-Host "  TOTAL: $grandPassed/$grandTotal tests passed" -ForegroundColor Green
    if ($grandIgnored -gt 0) {
        Write-Host "  ($grandIgnored ignored)" -ForegroundColor Yellow
    }
} else {
    Write-Host "  $grandPassed passed, $grandFailed failed ($grandTotal total)" -ForegroundColor Red
    if ($global:failedSuites.Count -gt 0) {
        Write-Host "  Failures: $($global:failedSuites -join ', ')" -ForegroundColor Red
    }
}
Write-Host "=============================================" -ForegroundColor Magenta
Write-Host "  Finished at $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor DarkGray

if ($grandFailed -eq 0 -and $global:failedSuites.Count -eq 0) {
    Write-Host "  Release tag: $grandPassed/$grandTotal tests" -ForegroundColor Cyan
}
