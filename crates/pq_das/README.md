# PQ-DAS V2 LeanVM Demo

This branch contains only the current PQ-DAS V2 demo implementations:

- `v2_base`: base-field blobs. A 128 KiB blob is represented as $32768$ KoalaBear elements. RS membership uses `dot_product_be` with a public extension-field check vector $L$.
- `v2_ext`: extension-field blobs. A blob follows the Tau-style size of $8192$ quintic-extension elements, serialized as five KoalaBear coordinates per symbol. RS membership uses the extension-field path over the shorter codeword.

The V1 implementation is intentionally not present on this branch. It is preserved separately on the `V1-Demo` branch.

## V2-base Workflow

- `v2_base::encode_and_commit()` encodes each blob, reorders codewords into physical systematic-prefix layout, hashes every cell inside the codeword matrix, hashes systematic cell digests into row hashes, and builds the two-level column Merkle commitment.
- `v2_base::prepare_statement_with_relation()` recomputes Fiat-Shamir and the public $L$ vector from the public commitment, then compiles the selected LeanVM guest with row hashes, the column root, and $L$ in read-only memory.
- `v2_base::prove_codewords()` proves the selected relation. The full relation proves cell digest computation, row digest checks, column Merkle root computation, and RS membership.
- `v2_base::query()` opens sampled cell columns and returns the cells plus outer column-root authentication paths.
- `v2_base::verify_openings()` recomputes opened cell digests, inner column roots, and outer Merkle paths against the public column root.
- `v2_base::reconstruct()` supports arbitrary verified cell erasure patterns using the shared FFT-based erasure decoder.

## V2-ext Workflow

- `v2_ext::encode_and_commit()` follows the same V2 commitment layout, but blobs and codewords are quintic-extension elements.
- `v2_ext::prepare_statement()` independently derives the public extension-field $L$ vector on prover and verifier sides.
- `v2_ext::prove_codewords()` proves the full V2 relation for extension-field codewords.
- `v2_ext::query()`, `v2_ext::verify_openings()`, and `v2_ext::reconstruct()` mirror the V2-base APIs for extension-field cells.

## Current Benchmark Commands

Run all V2-base benchmarks:

```bash
cargo run --release -p pq_das -- --version v2_base --all-v2-base-benchmarks --v2-relation full
```

The old alias still works:

```bash
cargo run --release -p pq_das -- --version v2 --all-v2-benchmarks --v2-relation full
```

Run all V2-ext benchmarks:

```bash
cargo run --release -p pq_das -- --version v2_ext --all-v2-ext-benchmarks
```

Run one V2-base relation-isolation benchmark:

```bash
cargo run --release -p pq_das -- --version v2_base --profile blob-128k-16 --v2-relation row-hash-only
cargo run --release -p pq_das -- --version v2_base --profile blob-128k-16 --v2-relation cell-commit-only
cargo run --release -p pq_das -- --version v2_base --profile blob-128k-16 --v2-relation membership-only
```

Run one V2-ext profile:

```bash
cargo run --release -p pq_das -- --version v2_ext --profile blob-ext-1
```

Run tests:

```bash
cargo test --release -p pq_das -- --nocapture
```

## Notes

- `v2_base` and `v2_ext` both keep Fiat-Shamir and $L$ computation outside the proof. The verifier rebuilds the statement and never trusts a prover-supplied $L$.
- The LeanVM proof witness is the private codeword matrix. Public row hashes, the column root, and $L$ are bound through the read-only public-data segment.
- The optimized guests use fixed cell-hash chunking and avoid the old outer-tree leaf copy by Merkle-rooting directly over column roots.
