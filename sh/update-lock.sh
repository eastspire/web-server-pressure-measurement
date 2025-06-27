#!/bin/bash
cd ./open-keep-alive/hyperlane;
rm -rf Cargo.lock;
cargo check;
cd ../../close-keep-alive/hyperlane;
rm -rf Cargo.lock;
cargo check;
cd ../../test-request;
rm -rf Cargo.lock;
cargo check;
