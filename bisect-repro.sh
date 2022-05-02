#!/bin/bash

set -eu -o pipefail
trap 'exit 1' INT

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
    # -C debuginfo=0
    # -Z verify-llvm-ir=yes
    # -C save-temps
    # -C no-prepopulate-passes
    # --emit="llvm-ir,link"
    -C codegen-units=1
    -C lto=off
    # -Z no-parallel-llvm # but this works!
)

printf '' > cargo-output.txt

function run_cargo_with_pass() {
    local pass=$1
    local rc=0

    cargo clean >>cargo-output.txt 2>&1

    { set -x; } 2>/dev/null
    RUSTFLAGS="${FLAGS[*]} -C llvm-args=-opt-bisect-limit=${pass}"
    cargo -v rustc --target armv6k-nintendo-3ds
    rc=$?
    { set +x; } 2>/dev/null
    return $rc
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
# MAX_PASS="$(
#     run_cargo_with_pass -1 |
#     sed -nr 's/^BISECT: running pass \(([0-9]+)\).*/\1/p' |
#     sort |
#     tail -n1
# )"

# Estimated min/max to speed up the bisect by a few iterations
MAX_PASS=499999
MIN_PASS=0
MAX_PASS=$(( MAX_PASS + 1 ))
echo "Max passes: $MAX_PASS"
PASS=$(( (MAX_PASS - MIN_PASS) / 2 ))

# run_executable
# if query_if_crashed; then
#     echo "Max pass (-1) crashes, bisecting is impossible. Exiting"
#     exit 1
# fi

run_cargo_with_pass $MAX_PASS >/dev/null
run_executable
if query_if_crashed; then
    echo "Max passes (${MAX_PASS}) crashes, bisecting is impossible. Exiting"
    exit 1
fi

while [[ $PASS -gt $MIN_PASS && $PASS -le $MAX_PASS ]]; do
    echo "Attempting ${PASS} passes"

    LAST_PASS=$PASS
    run_cargo_with_pass $PASS >/dev/null
    run_executable

    # if ! run_cargo_with_pass $PASS >/dev/null; then
    if query_if_crashed; then
        # echo "Cargo failed, increasing pass number"
        PASS=$(( PASS + (MAX_PASS - PASS) / 2 ))
        MIN_PASS=$LAST_PASS
    else
        # echo "Cargo succeeded, decreasing pass number"
        PASS=$(( (MIN_PASS + PASS) / 2 ))
        MAX_PASS=$LAST_PASS
    fi
done

echo "Final pass was: $PASS"
