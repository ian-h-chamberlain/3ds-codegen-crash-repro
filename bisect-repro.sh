#!/bin/bash

set -eu -o pipefail

function cargo() {
    { set +x; } 2>/dev/null
    # Dump all out/err to file, filter out noisy BISECT output
    command cargo "$@" 2>&1 | tee >(
        grep --line-buffered --invert-match BISECT \
        >> cargo-output.txt || true
    )
}

function query_if_crashed() {
    echo -n "Did it crash? [y/n] "
    while read -r -n1 CRASHED; do
        echo
        case $CRASHED in
            y | Y)
                return 0
                ;;
            n | N)
                return 1
                ;;
            *)
                echo -n "Did it crash? [y/n] "
                ;;
        esac
    done
}

export RUSTFLAGS
FLAGS=(
    -C opt-level=1
    -C debuginfo=0
    -C codegen-units=1
    # -Z verify-llvm-ir=yes
    # -Z no-parallel-llvm
)

printf '' > cargo-output.txt

function run_cargo_with_pass() {
    local pass
    pass=$1

    cargo clean >>cargo-output.txt 2>&1

    { set -x; } 2>/dev/null
    RUSTFLAGS="${FLAGS[*]} -C llvm-args=-opt-bisect-limit=${pass}"
    cargo -v rustc --target armv6k-nintendo-3ds
    { set +x; } 2>/dev/null
}

function run_executable() {
    EXE_NAME=target/armv6k-nintendo-3ds/debug/crash-repro
    3dsxtool "${EXE_NAME}.elf" "${EXE_NAME}.3dsx"

    # wait for 3ds to come back after possibly crashing on the last round
    while ! 3dslink "${EXE_NAME}.3dsx"; do
        sleep 3;
    done
}

echo "Finding max number of LLVM passes..."
MAX_PASS="$(
    run_cargo_with_pass -1 |
    sed -nr 's/^BISECT: running pass \(([0-9]+)\).*/\1/p' |
    sort |
    tail -n1
)"

run_executable
if query_if_crashed; then
    echo "Max pass (-1) crashes, bisecting is impossible. Exiting"
    exit 1
fi

MAX_PASS=$(( MAX_PASS * 2 ))
run_cargo_with_pass $MAX_PASS >/dev/null
run_executable
if query_if_crashed; then
    echo "Max passes (${MAX_PASS}) crashes, bisecting is impossible. Exiting"
    exit 1
fi

echo "Max passes: $MAX_PASS"
MIN_PASS=0
PASS=$(( (MAX_PASS - MIN_PASS) / 2 ))

while [[ $PASS -gt $MIN_PASS && $PASS -le $MAX_PASS ]]; do
    echo "Attempting ${PASS} passes"

    run_cargo_with_pass $PASS >/dev/null
    run_executable

    LAST_PASS=$PASS
    if query_if_crashed; then
        PASS=$(( PASS + (MAX_PASS - PASS) / 2 ))
        MIN_PASS=$LAST_PASS
    else
        PASS=$(( (MIN_PASS + PASS) / 2 ))
        MAX_PASS=$LAST_PASS
    fi
done

echo "Final pass was: $PASS"
