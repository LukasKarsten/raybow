#!/bin/sh

echo Removing previous profiling data
rm -rf /tmp/pgo-data

echo Building with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
  cargo build --release --target=x86_64-unknown-linux-gnu

echo Running instrumented binary
./target/x86_64-unknown-linux-gnu/release/raybow

echo Processing profiling data
llvm-profdata merge -o /tmp/pgo-data/merged.prodata /tmp/pgo-data

echo Building with profiling data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" \
  cargo build --release --target=x86_64-unknown-linux-gnu
