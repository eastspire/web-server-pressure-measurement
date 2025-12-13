#!/bin/bash
cd ./flamegraph;
rm -rf ./flamegraph.svg;
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release &
SERVER_PID=$!
cd ../node;
node index.js &
CLIENT_PID=$!
wait $SERVER_PID
kill $CLIENT_PID 2>/dev/null
