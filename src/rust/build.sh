#!/usr/bin/env bash

# This file invokes all the Rust builds, as long as we can't use a Cargo workspace.
# See "cargo-workspace-info.md"

set -e
set -x

# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

# libs are regular no_std custom libs, that are not specific to a target
# We develop them as we would use them on the host platform. This makes test
# execution easier (as long as https://github.com/rust-lang/cargo/issues/9710 exists).
LIBS=(
    "kernel-lib"
)

for LIB in "${LIBS[@]}"
do
   # the parentheses will start a subshell => we have no need to cd back
   (
     cd "$LIB" || exit
     cargo build
     cargo test
     cargo fmt -- --check
   )
done




BINS=(
    "kernel-bin"
)

for BIN in "${BINS[@]}"
do
   # the parentheses will start a subshell => we have no need to cd back
   (
     cd "$BIN" || exit
     cargo build
     cargo fmt -- --check
     # tests don't work so far
     # cargo test --target x86_64-unknown-linux-gnu
   )
done
