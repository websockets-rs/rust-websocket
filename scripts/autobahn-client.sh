#!/usr/bin/env bash
# Author michael <themichaeleden@gmail.com>
set -euo pipefail
set -x
SOURCE_DIR=$(readlink -f "${BASH_SOURCE[0]}")
SOURCE_DIR=$(dirname "$SOURCE_DIR")
cd "${SOURCE_DIR}/.."

function cleanup() {
    kill -9 ${FUZZINGSERVER_PID}
}
trap cleanup TERM EXIT

wstest -m fuzzingserver -s 'autobahn/fuzzingserver.json' & \
    FUZZINGSERVER_PID=$!
sleep 10

function test_diff() {
    SAVED_RESULTS=$(sed 's/NON-STRICT/OK/g' autobahn/client-results.json)
    DIFF=$(diff \
        <(jq -S 'del(."rust-websocket" | .. | .duration?)' "$SAVED_RESULTS") \
        <(jq -S 'del(."rust-websocket" | .. | .duration?)' 'autobahn/client/index.json') )

    if [[ $DIFF ]]; then
        echo 'Difference in results, either this is a regression or' \
             'one should update autobahn/client-results.json with the new results.' \
             'The results are:'
        echo $DIFF
        exit 64
    fi
}

cargo build --example autobahn-client
cargo run --example autobahn-client
test_diff

cargo build --example async-autobahn-client
cargo run --example async-autobahn-client
test_diff

