#!/bin/bash

# common features
COMMON_FEATURES=("schema_utils")

# schema versions features (passed to clippy one at a time)
SCHEMA_VERSION_FEATURES=("2025_06_18", "2025_03_26", "2024_11_05", "draft")

# space-separated string
COMMON_FEATURES_STR="${COMMON_FEATURES[*]}"

for FEATURE in "${SCHEMA_VERSION_FEATURES[@]}"; do
    echo "🚀 Running Clippy with: --features \"$COMMON_FEATURES_STR $FEATURE\""
    cargo clippy --all-targets --no-default-features --features "$COMMON_FEATURES_STR $FEATURE" -- -A deprecated -D warnings

    # stop on failure
    if [ $? -ne 0 ]; then
        echo "❌ Clippy failed for: --features \"$COMMON_FEATURES_STR $FEATURE\""
        exit 1
    fi
done

echo "✅ All Clippy lints have passed!"
