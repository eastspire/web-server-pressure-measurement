#!/bin/bash
cd ./open-keep-alive/hyperlane;
rm -rf Cargo.lock;
cargo fmt;
cargo check;
cd ../../close-keep-alive/hyperlane;
rm -rf Cargo.lock;
cargo fmt;
cargo check;
cd ../../test-request;
rm -rf Cargo.lock;
cargo fmt;
cargo check;
cd ../../flamegraph;
rm -rf Cargo.lock;
cargo fmt;
cargo check;
cd ../;
./sh/flamegraph.sh;
