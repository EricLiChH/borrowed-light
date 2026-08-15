#!/bin/sh
set -eu

course_mdbook_bin="${MDBOOK_BIN:-mdbook}"
course_rustlings_bin="${RUSTLINGS_BIN:-rustlings}"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
"$course_mdbook_bin" test book
"$course_mdbook_bin" build book
(
    cd exercises
    "$course_rustlings_bin" dev check
)
