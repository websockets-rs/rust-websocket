#!/usr/bin/env bash
set -euo pipefail
SOURCE_DIR=$(readlink -f "${BASH_SOURCE[0]}")
SOURCE_DIR=$(dirname "$SOURCE_DIR")
cd "${SOURCE_DIR}/.."

if [[ ${1:-} = "-i" ]]; then
    INTERACTIVE=true
fi

ALL_FEATURES="
async
sync
sync-ssl
async-ssl
async sync
async sync-ssl
sync async-ssl
sync-ssl async-ssl"

while read FEATS; do
    if [[ ${INTERACTIVE:-} ]]; then
        cargo build --no-default-features --features "$FEATS" \
              --color always 2>&1 | less -r
    else
        set -x
        cargo build --no-default-features --features "$FEATS"
        set +x
    fi
done < <(echo "$ALL_FEATURES")

## all combs of features (lol)
# async
# sync
# sync-ssl
# async-ssl
# async sync
# async sync-ssl
# async async-ssl
# sync sync-ssl
# sync async-ssl
# sync-ssl async-ssl
# async sync sync-ssl
# async sync async-ssl
# async sync-ssl async-ssl
# sync sync-ssl async-ssl
# async sync sync-ssl async-ssl
