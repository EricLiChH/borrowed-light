#!/bin/sh
set -eu

course_mdbook_bin="${MDBOOK_BIN:-mdbook}"
course_rustlings_bin="${RUSTLINGS_BIN:-rustlings}"
course_python_bin="${PYTHON_BIN:-python3}"

cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
"$course_python_bin" scripts/check_links.py
"$course_mdbook_bin" test book
"$course_mdbook_bin" build book
(
    cd exercises
    "$course_rustlings_bin" dev check
)
