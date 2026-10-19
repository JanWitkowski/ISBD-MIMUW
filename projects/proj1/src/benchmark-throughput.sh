#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE="$SCRIPT_DIR/../testdata/test_2gb.bin"
READ_BIN="$SCRIPT_DIR/target/release/throughput"
MMAP_BIN="$SCRIPT_DIR/target/release/throughput2"

BLOCK_SIZES=(512 4096 16384 65536 1048576 4194304 16777216)
REPEATS=3

cd "$SCRIPT_DIR"
cargo build --release --bin throughput --bin throughput2 --quiet

for block in "${BLOCK_SIZES[@]}"; do
    read_minimum=""
    mmap_minimum=""

    for ((repeat = 1; repeat <= REPEATS; repeat++)); do
        read_output=$("$READ_BIN" "$FILE" "$block")
        read_time=$(awk -F '|' '/Throughput/ {gsub(/[^0-9]/, "", $2); print $2}' <<< "$read_output")

        mmap_output=$("$MMAP_BIN" "$FILE" "$block")
        mmap_time=$(awk -F '|' '/Throughput mmap/ {gsub(/[^0-9]/, "", $2); print $2}' <<< "$mmap_output")

        if [[ -z "$read_minimum" || "$read_time" -lt "$read_minimum" ]]; then
            read_minimum=$read_time
        fi

        if [[ -z "$mmap_minimum" || "$mmap_time" -lt "$mmap_minimum" ]]; then
            mmap_minimum=$mmap_time
        fi
    done

    echo "| M=$block B | read min=${read_minimum} µs | mmap min=${mmap_minimum} µs |"
done
