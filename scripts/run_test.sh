#!/bin/bash

# common features (always included in the tests)
COMMON_FEATURES=("schema_utils")  

# schema versions features (tested one at a time)
SCHEMA_VERSION_FEATURES=("2024_11_05" "draft")

# space-separated string 
COMMON_FEATURES_STR="${COMMON_FEATURES[*]}"

for FEATURE in "${SCHEMA_VERSION_FEATURES[@]}"; do
    echo "🚀 Running tests with: --features \"$COMMON_FEATURES_STR $FEATURE\""
    cargo nextest run --no-default-features --features "$COMMON_FEATURES_STR $FEATURE"

    # stop on failure
    if [ $? -ne 0 ]; then
        echo "❌ Tests failed for: --features \"$COMMON_FEATURES_STR $FEATURE\""
        exit 1
    fi
done

echo "✅ All tests passed!"