#!/bin/bash

# common features
COMMON_FEATURES=("schema_utils")

# schema versions features
SCHEMA_VERSION_FEATURES=("2025_11_25" "2025_06_18" "2025_03_26" "2024_11_05" "draft")

# space-separated string
COMMON_FEATURES_STR="${COMMON_FEATURES[*]}"

run_clippy() {
    local target_flag="$1"  # "" for default, "--bins", "--tests", "--examples"
    echo "🚀 Running Clippy $target_flag with features \"$COMMON_FEATURES_STR $FEATURE\""
    cargo clippy $target_flag --no-default-features --features "$COMMON_FEATURES_STR $FEATURE" -- -A deprecated -D warnings

    if [ $? -ne 0 ]; then
        echo "❌ Clippy failed for $target_flag with features \"$COMMON_FEATURES_STR $FEATURE\""
        exit 1
    fi
}

for FEATURE in "${SCHEMA_VERSION_FEATURES[@]}"; do
    # Run Clippy (exclude examples)
    run_clippy "--lib --bins --tests"

    # Run Clippy for examples only for 2025_11_25
    if [ "$FEATURE" == "2025_11_25" ]; then
        run_clippy "--examples"
    fi
done

echo "✅ All Clippy lints have passed!"
