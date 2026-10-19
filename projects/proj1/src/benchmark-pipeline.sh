#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE="$SCRIPT_DIR/../testdata/test_2gb.bin"
BIN="$SCRIPT_DIR/target/release/pipeline"

BLOCK_SIZES=(4096 65536 1048576)
K_VALUES=(1 5 50 100)
THREAD_COUNTS=(1 2 4 8)
REPEATS=2

cd "$SCRIPT_DIR"

if command -v cargo > /dev/null 2>&1; then
    cargo build --release --bin pipeline --quiet
elif [[ ! -x "$BIN" ]]; then
    echo "Nie znaleziono cargo ani zbudowanej binarki: $BIN" >&2
    exit 1
fi

measure_minimum() {
    local block=$1
    local k=$2
    local readers=$3
    local computers=$4
    local minimum=""

    for ((repeat = 1; repeat <= REPEATS; repeat++)); do
        output=$("$BIN" "$FILE" "$block" "$k" "$readers" "$computers")
        time=$(awk -F '|' '/Pipeline/ {gsub(/[^0-9]/, "", $2); print $2}' <<< "$output")

        if [[ -z "$minimum" || "$time" -lt "$minimum" ]]; then
            minimum=$time
        fi
    done

    printf '%s' "$minimum"
}

echo "M i K (1 czytelnik, 1 obliczacz)"
for block in "${BLOCK_SIZES[@]}"; do
    for k in "${K_VALUES[@]}"; do
        minimum=$(measure_minimum "$block" "$k" 1 1)
        echo "| M=$block B | K=$k | readers=1 | computers=1 | min=${minimum} µs |"
    done
done

echo "Liczba czytelników (M=1048576 B, K=1, 1 obliczacz)"
for readers in "${THREAD_COUNTS[@]}"; do
    minimum=$(measure_minimum 1048576 1 "$readers" 1)
    echo "| readers=$readers | computers=1 | min=${minimum} µs |"
done

echo "Liczba obliczaczy (M=1048576 B, K=1000, 1 czytelnik)"
for computers in "${THREAD_COUNTS[@]}"; do
    minimum=$(measure_minimum 1048576 1000 1 "$computers")
    echo "| readers=1 | computers=$computers | min=${minimum} µs |"
done
