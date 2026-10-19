#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE="$SCRIPT_DIR/../testdata/test.bin"
BIN="$SCRIPT_DIR/target/release/latency2"
RESULTS="$SCRIPT_DIR/results-latency2-cold-cache.csv"

THREADS=(1 2 4 8 16)
BLOCK_SIZES=(512 4096 16384 65536 1048576)
REPEATS=3

if [[ ! -f "$FILE" ]]; then
    echo "Nie znaleziono pliku testowego: $FILE" >&2
    exit 1
fi

cd "$SCRIPT_DIR"
cargo build --release --bin latency2 --quiet

sudo -v

echo "block_size,threads,repeat,avg_read_us" > "$RESULTS"

for block in "${BLOCK_SIZES[@]}"; do
    for threads in "${THREADS[@]}"; do
        best=""

        for ((repeat = 1; repeat <= REPEATS; repeat++)); do
            sync
            sudo sysctl -q -w vm.drop_caches=3

            output=$("$BIN" "$FILE" "$threads" "$block")
            time_us=$(awk -F '|' '/Latency2/ {gsub(/[^0-9.]/, "", $2); print $2}' <<< "$output")

            if [[ -z "$time_us" ]]; then
                echo "Nie udało się odczytać czasu z: $output" >&2
                exit 1
            fi

            echo "$block,$threads,$repeat,$time_us" >> "$RESULTS"

            if [[ -z "$best" ]] || awk -v current="$time_us" -v best="$best" \
                'BEGIN { exit !(current < best) }'; then
                best=$time_us
            fi
        done

        echo "| M=$block B | threads=$threads | cold-cache min avg-read=${best} µs |"
    done
done

echo "Gotowe. Wyniki zapisano w: $RESULTS"
