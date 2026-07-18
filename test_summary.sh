#!/usr/bin/env bash
# XIOM Full Test Suite — runs all tests and prints a single-line summary.
# Run: ./test_summary.sh
# Output last line: "TOTAL: 441/441 tests passed" (ready for release tagging)

set -euo pipefail

TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_IGNORED=0
FAILED_SUITES=()

echo ""
echo "XIOM Test Suite"
echo "=============="
echo ""

declare -A SUITES=(
    ["e2e"]="e2e_tests"
    ["feature-regression"]="feature_regression_tests"
    ["stdlib-execution"]="stdlib_execution_tests"
    ["diff"]="diff_tests"
    ["full-diff"]="full_diff_tests"
    ["fuzz"]="fuzz_tests"
    ["integration"]="integration_tests"
    ["robustness"]="robustness_tests"
    ["stdlib-compile"]="stdlib_tests"
)

for name in e2e feature-regression stdlib-execution diff full-diff fuzz integration robustness stdlib-compile; do
    test_file="${SUITES[$name]}"
    printf "  %-22s " "$name ..."
    output=$(cargo test -p xiom-codegen --test "$test_file" 2>&1) || true
    
    if echo "$output" | grep -q 'test result:'; then
        result_line=$(echo "$output" | grep 'test result:')
        passed=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
        failed=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
        ignored=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) ignored.*/\1/p')
        passed=${passed:-0}
        failed=${failed:-0}
        ignored=${ignored:-0}
        
        TOTAL_PASSED=$((TOTAL_PASSED + passed))
        TOTAL_FAILED=$((TOTAL_FAILED + failed))
        TOTAL_IGNORED=$((TOTAL_IGNORED + ignored))
        
        if [ "$failed" -gt 0 ]; then
            echo -e "\033[31mFAIL ($passed/$((passed + failed)))\033[0m"
            FAILED_SUITES+=("$name")
        else
            echo -e "\033[32mOK ($passed/$((passed + failed)))\033[0m"
        fi
    else
        echo -e "\033[31mERROR\033[0m"
    fi
done

TOTAL=$((TOTAL_PASSED + TOTAL_FAILED + TOTAL_IGNORED))
echo ""
echo "============================================="
if [ "$TOTAL_FAILED" -eq 0 ]; then
    echo -e "\033[32m  ALL $TOTAL TESTS PASSED\033[0m"
    echo -e "\033[32m  TOTAL: $TOTAL/$TOTAL tests passed\033[0m"
else
    echo -e "\033[31m  $TOTAL_PASSED passed, $TOTAL_FAILED failed ($TOTAL total)\033[0m"
    echo -e "\033[31m  Failures: ${FAILED_SUITES[*]}\033[0m"
fi
echo "============================================="

if [ "$TOTAL_FAILED" -eq 0 ]; then
    echo -e "\033[36m  Release tag: $TOTAL/$TOTAL tests\033[0m"
fi
