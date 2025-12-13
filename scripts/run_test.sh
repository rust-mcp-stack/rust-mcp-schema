#!/bin/bash

# common features (always included in the tests)
COMMON_FEATURES=("schema_utils")

# schema versions features (tested one at a time)
SCHEMA_VERSION_FEATURES=("2025_11_25" "2025_06_18" "2025_03_26" "2024_11_05" "draft")

# space-separated string
COMMON_FEATURES_STR="${COMMON_FEATURES[*]}"

run_nextest() {
    local target_flag="$1" # "--lib --bins --tests" or "--examples"
    echo "🚀 Running tests $target_flag with features \"$COMMON_FEATURES_STR $FEATURE\""

    cargo nextest run --no-tests=pass $target_flag \
        --no-default-features \
        --features "$COMMON_FEATURES_STR $FEATURE"

    if [ $? -ne 0 ]; then
        echo "❌ Tests failed for $target_flag with features \"$COMMON_FEATURES_STR $FEATURE\""
        exit 1
    fi
}

for FEATURE in "${SCHEMA_VERSION_FEATURES[@]}"; do
    # Run lib + bin + integration tests (NO examples)
    run_nextest "--lib --bins --tests"

    # Run example tests only for 2025_11_25
    if [ "$FEATURE" == "2025_11_25" ]; then
        run_nextest "--examples"
    fi
done

# Documentation tests (only once, only the latest schema)
FEATURE="${SCHEMA_VERSION_FEATURES[0]}"
echo
echo "🚀 Running documentation tests with: --features \"$COMMON_FEATURES_STR $FEATURE\""
cargo test --doc --no-default-features --features "$COMMON_FEATURES_STR $FEATURE"

if [ $? -ne 0 ]; then
    echo "❌ Documentation tests failed"
    exit 1
fi

echo "✅ All tests passed!"
