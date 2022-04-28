#!/bin/bash

set -eu -o pipefail

printf '' > cargo-output.txt

function cargo() {
    command cargo "$@" >>cargo-output.txt 2>&1
}

FLAGS=(
    -C opt-level=1
    -C debuginfo=0
    -Z verify-llvm-ir=yes
    -Z no-parallel-llvm
)

export RUSTFLAGS="${FLAGS[*]} -C llvm-args=-opt-bisect-limit=-1"

echo "Clean building to find max number of LLVM passes..."
cargo clean
MAX_PASS="$(
    2>&1 command cargo rustc --target armv6k-nintendo-3ds |
    tee -a cargo-output.txt |
    sed -nr 's/^BISECT: running pass \(([0-9]+)\).*/\1/p' |
    sort |
    tail -n1
)"
MAX_PASS=$((MAX_PASS + 1))

echo "Max passes: $MAX_PASS"

MIN_PASS=0

PASS=$(( (MAX_PASS - MIN_PASS) / 2 ))

while [[ $PASS -gt $MIN_PASS && $PASS -lt $MAX_PASS ]]; do
    echo "Running bisect-limit = $PASS"

    cargo clean >>cargo-output.txt 2>&1

export RUSTFLAGS="${FLAGS[*]} -C llvm-args=-opt-bisect-limit=$PASS"
    cargo -v rustc --target armv6k-nintendo-3ds \
        -- -C llvm-args=-opt-bisect-limit=$PASS \

    EXE_NAME=target/armv6k-nintendo-3ds/debug/crash-repro

    3dsxtool "${EXE_NAME}.elf" "${EXE_NAME}.3dsx"

    # wait for 3ds to come back after possibly crashing on the last round
    while ! 3dslink "${EXE_NAME}.3dsx"; do
        sleep 3;
    done

    LAST_PASS=$PASS

    echo -n "Did it crash? [y/n] "
    while read -n1 CRASHED; do
        echo
        case $CRASHED in
            y | Y)
                PASS=$(( PASS + (MAX_PASS - PASS) / 2 ))
                MIN_PASS=$LAST_PASS
                break
                ;;
            n | N)
                PASS=$(( (MIN_PASS + PASS) / 2 ))
                MAX_PASS=$LAST_PASS
                break
                ;;
            *)
                echo -n "Did it crash? [y/n] "
                ;;
        esac
    done
done

echo "Final pass was: $PASS"
