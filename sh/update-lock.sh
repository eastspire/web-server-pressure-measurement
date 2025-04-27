#!/bin/bash
cd ./open-keep-alive/hyperlane;
cargo build --release;
cd ../../close-keep-alive/hyperlane;
cargo build --release;
cd ../../;
