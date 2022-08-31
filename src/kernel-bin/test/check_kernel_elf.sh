#!/usr/bin/env bash

set -e

EXPECTED_FILE=test/expected_readelf.txt
EXPECTED=$(cat "$EXPECTED_FILE")
ACTUAL=$(readelf -Wl "$KERNEL_BINARY")

if [[ "$EXPECTED" == "$ACTUAL" ]]; then
    echo "✅ kernel ELF looks good"
else
    echo "⚠ ELF doesn't look as expected:"
    echo "$ACTUAL" | diff "$EXPECTED_FILE" -
fi
