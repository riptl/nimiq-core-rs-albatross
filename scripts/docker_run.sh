#!/usr/bin/env bash

set -euo pipefail

mkfifo "$HOME/nimiq.log.pipe" || true
cat "$HOME/nimiq.log.pipe" &
mkdir -p "$HOME/.nimiq"

function hex2bin () {
    sed 's/\([0-9A-F]\{2\}\)/\\\\\\x\1/gI' | xargs printf
}

if [[ ! -z "$NIMIQ_PEER_KEY" ]]; then
    export NIMIQ_PEER_KEY_FILE="$HOME/.nimiq/peer_key.dat"
    echo "$NIMIQ_PEER_KEY" | hex2bin > "$NIMIQ_PEER_KEY_FILE"
fi

if [[ ! -z "$VALIDATOR_KEY" ]]; then
    export VALIDATOR_KEY_FILE="$HOME/.nimiq/validator_key.dat"
    echo "$VALIDATOR_KEY" | hex2bin > "$VALIDATOR_KEY_FILE"
fi

if [[ -z "$NIMIQ_HOST" ]]; then
    NIMIQ_HOST="$(hostname -i)"
    export NIMIQ_HOST
fi

./docker_config.sh > "$HOME/.nimiq/client.toml"

nimiq-client "$@"
