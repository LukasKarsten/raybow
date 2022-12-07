#!/bin/sh

PGO_DIR=$PWD/.pgo

echo Removing previous profiling data
rm -rf /tmp/pgo-data

echo Building with instrumentation
RUSTFLAGS="-Cprofile-generate=$PGO_DIR -Cllvm-args=-pgo-warn-missing-function" \
  cargo build --release --target=x86_64-unknown-linux-gnu

echo Running instrumented binary
./target/x86_64-unknown-linux-gnu/release/raybow 240 135 --spheres-per-axis=4 -o /dev/null

echo Processing profiling data
llvm-profdata merge -o $PGO_DIR/merged.profdata $PGO_DIR

echo Building with profiling data
RUSTFLAGS="-Cprofile-use=$PGO_DIR/merged.profdata" \
  cargo build --release --target=x86_64-unknown-linux-gnu
