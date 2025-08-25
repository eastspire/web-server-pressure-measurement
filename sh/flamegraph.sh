#!/bin/bash
cd ./flamegraph;
rm -rf ./flamegraph.svg;
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release;
