#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

RUNS=${RUNS:-1}
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
COMMIT=$(git rev-parse --short=12 HEAD)
OUTPUT_DIR=${1:-"benchmark-results/server/${TIMESTAMP}_${COMMIT}"}
mkdir -p "$OUTPUT_DIR/logs"

{
    echo "timestamp_utc: $TIMESTAMP"
    echo "commit: $(git rev-parse HEAD)"
    echo "branch: $(git branch --show-current)"
    echo "rustc: $(rustc --version)"
    echo "cargo: $(cargo --version)"
    echo "rustflags: ${RUSTFLAGS:-}"
    echo "runs: $RUNS"
    echo
    uname -a
    echo
    lscpu
} >"$OUTPUT_DIR/machine.txt"

echo "Building pq_das with RUSTFLAGS=${RUSTFLAGS:-<unset>}"
cargo build --release -p pq_das 2>&1 | tee "$OUTPUT_DIR/build.log"

profiles=(tiny medium large stress blob-128k-1 blob-128k-4)
reduced_relations=(row-hashes column-merkle rs-membership)

for run in $(seq 1 "$RUNS"); do
    for profile in "${profiles[@]}"; do
        log="$OUTPUT_DIR/logs/${profile}_all_run${run}.log"
        echo "Running profile=$profile relation=all run=$run/$RUNS"
        target/release/pq_das \
            --profile "$profile" \
            --relation all \
            --detailed-profiling \
            2>&1 | tee "$log"
    done

    for profile in "${profiles[@]}"; do
        for relation in "${reduced_relations[@]}"; do
            log="$OUTPUT_DIR/logs/${profile}_${relation}_run${run}.log"
            echo "Running profile=$profile relation=$relation run=$run/$RUNS"
            target/release/pq_das \
                --profile "$profile" \
                --relation "$relation" \
                --detailed-profiling \
                --skip-reconstruction \
                2>&1 | tee "$log"
        done
    done
done

python3 scripts/summarize_pq_das_profiling.py "$OUTPUT_DIR"
echo "Benchmark results written to $OUTPUT_DIR"
