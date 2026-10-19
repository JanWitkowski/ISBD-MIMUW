#!/bin/bash

set -e

FILE="../testdata/test.bin"
BIN="./target/release/latency2"
RESULTS="results-latency2-cache.csv"

THREADS=(1 2 4 8 16)
BLOCK_SIZES=(512 4096 16384 65536 1048576)
REPEATS=3

cargo build --release --bin latency2 --quiet

echo "block_size,threads,repeat,avg_read_us" > "$RESULTS"

for block in "${BLOCK_SIZES[@]}"; do
    for threads in "${THREADS[@]}"; do

        best=""

        for ((i=1; i<=REPEATS; i++)); do

            output=$("$BIN" "$FILE" "$threads" "$block")

            time_us=$(awk -F '|' '/Latency2/ {gsub(/[^0-9.]/, "", $2); print $2}' <<< "$output")

            if [[ -z "$time_us" ]]; then
                echo "Nie udało się odczytać czasu z: $output" >&2
                exit 1
            fi

            echo "$block,$threads,$i,$time_us" >> "$RESULTS"

            if [[ -z "$best" ]] || awk -v current="$time_us" -v best="$best" \
                'BEGIN { exit !(current < best) }'; then
                best=$time_us
            fi

        done

        echo "| M=$block B | threads=$threads | cache min avg-read=${best} µs |"

    done
done

echo "Gotowe! Wyniki zapisano w $RESULTS"
