#!/bin/sh

PGO_DIR=$(mktemp -d)
echo "Storing profile data at $PGO_DIR"

echo Building with instrumentation
RUSTFLAGS="-Ctarget-cpu=native -Cprofile-generate=$PGO_DIR -Cllvm-args=-pgo-warn-missing-function" \
  cargo build --release --target=x86_64-unknown-linux-gnu

echo Running instrumented binary
./target/x86_64-unknown-linux-gnu/release/raybow builtin:spheres 240 135 -r 200 -o /dev/null

echo Processing profiling data
llvm-profdata merge -o $PGO_DIR/merged.profdata $PGO_DIR

echo Building with profiling data
RUSTFLAGS="-Ctarget-cpu=native -Cprofile-use=$PGO_DIR/merged.profdata" \
  cargo build --release --target=x86_64-unknown-linux-gnu
