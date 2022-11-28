#!/usr/bin/env bash

# This script helps to check if the structure of the ELF changes. As early boot code heavily relies
# on certain properties (such as addresses and offsets), this is something that we want.

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
