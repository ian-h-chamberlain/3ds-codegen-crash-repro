#!/bin/bash

set -eu -o pipefail

MIN_PASS=0
MAX_PASS=2000

PASS=$(( (MAX_PASS - MIN_PASS) / 2 ))

while [[ $PASS -gt $MIN_PASS && $PASS -lt $MAX_PASS ]]; do
    echo "Running bisect-limit = $PASS"

    cargo clean

    cargo 3ds rustc \
        -- \
        -C opt-level=1 \
        -C codegen-units=1 \
        -Z verify-llvm-ir=yes \
        -Z no-parallel-llvm \
        -C llvm-args=-opt-bisect-limit=$PASS

    # wait for 3ds to come back after presumably crashing
    while ! 3dslink ../target/armv6k-nintendo-3ds/debug/examples/regex.elf; do
        sleep 5;
    done

    LAST_PASS=$PASS

    echo -n "Did it crash? [y/n] "
    while read -n1 CRASHED; do
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
