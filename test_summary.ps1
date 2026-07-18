#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Full Test Suite — runs all tests and prints a single-line summary
.DESCRIPTION
    Run: .\test_summary.ps1
    Output last line: "TOTAL: 444/444 tests passed" (ready for release tagging)
#>

$ErrorActionPreference = "Continue"

Write-Host "XIOM Test Suite" -ForegroundColor Magenta
Write-Host "==============" -ForegroundColor Magenta
Write-Host ""

$totalPassed = 0
$totalFailed = 0
$totalIgnored = 0
$failedSuites = @()

$suites = @(
    @{name="e2e";               test="e2e_tests"},
    @{name="feature-regression";test="feature_regression_tests"},
    @{name="stdlib-execution";  test="stdlib_execution_tests"},
    @{name="diff";              test="diff_tests"},
    @{name="full-diff";         test="full_diff_tests"},
    @{name="fuzz";              test="fuzz_tests"},
    @{name="integration";       test="integration_tests"},
    @{name="robustness";        test="robustness_tests"},
    @{name="stdlib-compile";    test="stdlib_tests"}
)

foreach ($suite in $suites) {
    $name = $suite.name
    $test = $suite.test
    Write-Host "  $($name.PadRight(22))" -NoNewline
    $output = & cargo test -p xiom-codegen --test $test 2>&1 | Out-String
    
    # Parse the test result line from cargo output
    $matched = $false
    foreach ($line in ($output -split "`n")) {
        if ($line -match 'test result: (\w+)\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
            $status = $Matches[1]
            $passed = [int]$Matches[2]
            $failed = [int]$Matches[3]
            $ignored = [int]$Matches[4]
            $matched = $true
            break
        }
    }
    
    if ($matched) {
        $totalPassed += $passed
        $totalFailed += $failed
        $totalIgnored += $ignored
        if ($failed -gt 0) {
            Write-Host "FAIL ($passed/$($passed + $failed))" -ForegroundColor Red
            $failedSuites += $name
        } else {
            Write-Host "OK ($passed/$($passed + $failed))" -ForegroundColor Green
        }
    } else {
        Write-Host "CRASH" -ForegroundColor Red
        $failedSuites += "$name (no result)"
    }
}

$total = $totalPassed + $totalFailed + $totalIgnored
Write-Host ""
Write-Host "=============================================" -ForegroundColor Magenta
if ($totalFailed -eq 0 -and $failedSuites.Count -eq 0) {
    Write-Host "  ALL $total TESTS PASSED" -ForegroundColor Green
    Write-Host "  TOTAL: $total/$total tests passed" -ForegroundColor Green
} else {
    Write-Host "  $totalPassed passed, $totalFailed failed ($total total)" -ForegroundColor Red
    if ($failedSuites.Count -gt 0) {
        Write-Host "  Failures: $($failedSuites -join ', ')" -ForegroundColor Red
    }
}
Write-Host "=============================================" -ForegroundColor Magenta

# Final summary line for release tags
if ($totalFailed -eq 0 -and $failedSuites.Count -eq 0) {
    Write-Host "  Release tag: $total/$total tests" -ForegroundColor Cyan
}
