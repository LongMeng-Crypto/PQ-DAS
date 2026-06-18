# PQ-DAS Optimization Checklist

## A. Profiling and Cost Breakdown

- [x] **A1. Add detailed LeanVM prover profiling:** report actual and padded rows for every LeanVM table.
- [x] **A2. Add stage-level prover timing:** separately measure bytecode execution, trace generation, memory access counting, logup, AIR sumcheck, WHIR, and grinding.
- [x] **A3. Enable LeanVM VM profiling and tracing:** expose instruction, memory, Poseidon, and extension-operation statistics.
- [x] **A4. Add relation-isolation benchmarks:** benchmark row hashing, column hashing plus Merkle construction, and RS membership independently.

## LeanVM Prover-Time Roadmap

Current profiling conclusion: for large profiles, LeanVM proving is 95-99% of end-to-end runtime. Inside LeanVM proving, WHIR, polynomial stacking/commitment, and logup account for roughly 82-83% of prover time. Therefore, optimizations that only make bytecode execution or trace generation slightly faster are not enough unless they also reduce committed trace size, memory-bus traffic, logup work, or cross a power-of-two padding boundary.

| Route | Requires LeanVM source changes? | Expected impact | Status |
| --- | --- | --- | --- |
| Reduce guest VM cycles and memory traffic in zkDSL only | No | Medium if it crosses a table padding boundary; otherwise low-to-medium | Still feasible |
| Use existing LeanVM precompiles more aggressively | No, if using current Poseidon16 and extension-op interfaces | Medium; limited by existing precompile granularity | Still feasible |
| Reduce execution-table padding by changing guest row count/profile shape | No protocol change, no LeanVM source change | Potentially high at threshold crossings | Still feasible |
| Reduce generic memory-bus/logup pressure without new AIR | Usually no for guest-side copies; yes for structural memory-bus changes | Medium-to-high if many memory lookups disappear | Partly feasible |
| Add PQ-DAS-specific precompile/AIR or dedicated trace columns | Yes | High | B-class work |
| Tune WHIR or change proof-system backend | Outside current scope | High but changes proof-system configuration/risk profile | Deferred |
| GPU acceleration | Outside current scope | High for polynomial/FFT-heavy stages | Deferred |

- **Decision rule:** a candidate should be benchmarked against LeanVM prove time, table row counts, padded row counts, and prover sub-stage timing. A change that does not reduce padded rows or logup/WHIR input size is unlikely to produce a large prover-time improvement.

## G. LeanVM-Prover Optimizations Without Deep LeanVM Source Changes

- [ ] **G1. Remove row-hash temporary `systematic` arrays in zkDSL:** investigated. Current Poseidon precompile requires contiguous memory, so true strided row hashing without a temporary array becomes B3 unless a new input mode is added.
- [ ] **G2. Remove column-hash temporary `column_data` arrays in zkDSL:** investigated. Current Poseidon precompile requires contiguous memory, so true gathered column hashing without a temporary array becomes B4 unless a new input mode is added.
- [ ] **G3. Reduce Merkle-tree VM memory footprint:** investigated. Current zkDSL memory is write-once, so a rolling two-level Merkle buffer cannot safely overwrite previous levels without LeanVM/DSL support.
- [ ] **G4. In-place Merkle level computation:** investigated. Blocked by the same write-once memory semantics as G3; the safe version becomes B5/B9.
- [ ] **G5. Specialize `hash_chunks` for fixed small chunk counts:** candidate implemented for the two-block case; keep only if server benchmark shows a net improvement.
- [ ] **G6. Reduce loop/index arithmetic in hot zkDSL loops:** candidate implemented by hoisting base offsets in row hashing, column hashing, and Merkle loops; keep only if server benchmark shows a net improvement.
- [ ] **G7. Reorder guest memory layout for sequential access:** investigated. A row-major to column-major witness layout would help column hashing but hurt row hashing and RS membership; a useful version likely needs either duplicated witness layout or B8-style dedicated columns.
- [ ] **G8. Split profiles to avoid bad power-of-two padding cliffs:** analysis-only candidate. Useful for benchmarking sensitivity, but it changes the demo parameter shape rather than optimizing a fixed profile.
- [ ] **G9. Add a padding-aware benchmark gate:** already supported by A1/A2 output; use it as the accept/reject rule for G5/G6 and future guest rewrites.
- [ ] **G10. Use existing relation toggles to isolate next guest rewrite:** run row-hash-only, column-Merkle-only, and RS-membership-only profiles after each zkDSL rewrite to prove which relation moved.
- [ ] **G11. Evaluate proof splitting only as an engineering benchmark:** produce separate LeanVM proofs for row hashes, column Merkle, and RS membership to see whether smaller independent padded tables beat one large proof. This changes proof packaging and verification workflow, so it is not the default protocol path.
- [ ] **G12. Investigate compiler-level constant folding and CSE for zkDSL:** reduce repeated arithmetic and address expressions before bytecode generation. This may be possible in `lean_compiler` without changing LeanVM AIR.

- **Expected ceiling without LeanVM/AIR changes:** these G-class changes can plausibly give a meaningful constant-factor improvement only if they remove enough VM rows or memory traffic to lower padded table sizes. They are unlikely to produce an order-of-magnitude prover-time reduction because WHIR, stacking/commitment, and logup dominate once the trace shape is fixed.

## B. LeanVM Proof Optimization

- [ ] **B1. Implement a PQ-DAS-specific LeanVM precompile/AIR:** directly constrain row hashes, the column Merkle root, and RS membership.
- [ ] **B2. Add a batched Poseidon sponge precompile:** process all row hashes or all column hashes in batches.
- [ ] **B3. Add a strided Poseidon input mode:** hash systematic row symbols directly without copying them into a temporary array.
- [ ] **B4. Add a gathered Poseidon input mode:** hash column cells directly across rows without constructing `column_data`.
- [ ] **B5. Fuse column hashing with Merkle construction:** stream leaf digests into a Merkle accumulator without storing the full tree in VM memory.
- [ ] **B6. Add a batched matrix-vector membership precompile:** compute all $\langle w_i,L\rangle$ checks while reusing the same public $L$.
- [ ] **B7. Add a structured fixed-column segment for $L$:** remove the $5m$ coordinates of $L$ from generic VM memory while keeping verifier-side derivation mandatory.
- [ ] **B8. Store codewords in dedicated AIR trace columns:** avoid placing the complete $n\times m$ codeword matrix in generic VM memory.
- [ ] **B9. Reduce generic VM memory traffic:** eliminate unnecessary intermediate arrays, reads, writes, and memory-bus lookups in the guest.

## C. Parallelism and Low-Level Prover Optimization

- [ ] **C1. Parallelize row-hash guest loops:** attempted and reverted; reduced a small execution-side stage but did not improve total LeanVM proving time.
- [ ] **C2. Parallelize column-hash guest loops:** attempted and reverted; same conclusion as C1.
- [ ] **C3. Parallelize each independent Merkle-tree level.**
- [ ] **C4. Parallelize RS membership checks across rows:** attempted and reverted; RS membership is not the dominant VM-cycle source in current profiles.
- [ ] **C5. Parallelize prover memory access-count construction:** attempted with atomics and reverted; future attempt should use thread-local accumulators followed by reduction.
- [ ] **C6. Parallelize bytecode access-count construction:** attempted with atomics and reverted; future attempt should use thread-local accumulators followed by reduction.
- [ ] **C7. Parallelize independent trace-table processing:** attempted and reverted; measured stage was too small to justify extra complexity.
- [x] **C8. Enable native CPU optimization:** benchmark builds using `RUSTFLAGS="-C target-cpu=native"`.
- [x] **C9. Verify SIMD utilization:** confirm AVX2 or AVX-512 Poseidon and packed KoalaBear arithmetic are active.
- [x] **C10. Tune LeanVM worker-thread count for the benchmark machine:** support `PQ_DAS_NUM_THREADS` and `RAYON_NUM_THREADS` for the internal worker pool.
- [ ] **C11. Add NUMA-aware trace allocation and worker placement.**
- [ ] **C12. Reuse prover scratch buffers and avoid repeated zero-initialization.**
- [ ] **C13. Evaluate GPU acceleration for WHIR FFTs, Poseidon, Merkle commitments, and sumcheck.**

## D. WHIR Configuration Optimization

- [ ] **D1. Benchmark `whir_log_inv_rate` values 1 through 4:** compare prover time, verifier time, proof size, and memory.
- [ ] **D2. Tune the initial WHIR folding factor while preserving at least 124-bit security.**
- [ ] **D3. Tune the subsequent WHIR folding factor while preserving at least 124-bit security.**
- [ ] **D4. Tune the RS-domain initial reduction factor.**
- [ ] **D5. Tune the grinding-bit and query-count allocation while preserving total soundness.**
- [ ] **D6. Tune `MAX_NUM_VARIABLES_TO_SEND_COEFFS`.**

## E. Host and Engineering Optimization

- [ ] **E1. Cache profile-specific compiled bytecode templates.**
- [ ] **E2. Bind commitment-specific row hashes, root, and $L$ without recompiling the guest.**
- [ ] **E3. Cache FFT roots, twiddle factors, domains, and reusable FFT plans.**
- [ ] **E4. Cache reconstruction locator data when multiple rows use the same erasure pattern.**
- [ ] **E5. Implement a zero-copy contiguous codeword representation.**
- [ ] **E6. Remove the witness flattening copy before LeanVM execution.**
- [ ] **E7. Parallelize native RS encoding across blobs.**
- [ ] **E8. Parallelize native row and column commitment hashing.**
- [ ] **E9. Parallelize reconstruction across rows.**
- [ ] **E10. Add reusable benchmark automation and machine-configuration reporting.**

## F. Protocol-Level Optimization Candidates

- [ ] **F1. Evaluate row-sharded parallel proofs:** measure latency, total proof size, and verifier cost.
- [ ] **F2. Evaluate recursive aggregation of row-sharded proofs.**
- [ ] **F3. Benchmark alternative cell sizes $c$:** measure Merkle cost, sample size, sampling count, and network granularity.
- [ ] **F4. Benchmark alternative RS code rates $\rho=k/m$:** recompute reconstruction and availability soundness for every rate.
- [ ] **F5. Design a V2 commitment that avoids hashing systematic symbols independently in both row and column commitments.**

## Security Constraints

- [ ] **S1. Keep LeanVM and DAS sampling soundness at or above the configured security target.**
- [ ] **S2. Keep verifier-side Fiat-Shamir challenge and $L$ reconstruction mandatory.**
- [ ] **S3. Keep row hashes, column commitment root, and RS membership inside the proved relation.**
- [ ] **S4. Do not reduce the extension degree, digest length, or proof-system security solely for performance.**
