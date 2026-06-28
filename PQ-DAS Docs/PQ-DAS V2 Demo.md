# PQ-DAS V2 Demo

This demo line first replaces the V1 commitment layout with the V2 cell-digest layout, so the proof binds each codeword through cell hashes that are reused by both row and column commitments. After that, we applied a sequence of LeanVM engineering optimizations to reduce copying, memory traffic, bytecode expansion, and generic hash-loop overhead. The final branch contains two instantiations: `v2_base`, which preserves the original 128 KiB KoalaBear-blob format, and `v2_ext`, which uses quintic-extension blobs to shorten the codeword and membership checks.

## Optimizations from V1
- **Cell-first layout:** V2 replaces the V1 raw-column commitment with a cell-first layout: every codeword cell is hashed into a digest, then the same cell digests feed row hashes and column Merkle commitments
- **Even-first codewords:** Each codeword is stored physically as $[w_0,w_2,\ldots,w_{m-2},w_1,w_3,\ldots,w_{m-1}]$, so the systematic payload occupies the first $k$ physical symbols. Row hashing reads the systematic prefix, cell hashing reads direct cell slices, and RS membership reads each row contiguously, avoiding strided gathers and temporary `systematic` or `cell_data` arrays.
- **Column-major digests:** The witness codewords remain row-major, but the derived cell-digest matrix is column-major with padded row count, making each inner column Merkle tree read a contiguous digest block.
- **Fixed cell hashing:** The guest uses fixed-length Poseidon16 compression chains specialized to the benchmark cell sizes, avoiding generic runtime sponge loops where the length is known.

- **Merkle fusion:** The optimized guest removes the outer-tree leaf copy and computes the outer Merkle root directly from the column-root array.

- **Batch rows:** Multi-blob profiles prove many rows in one LeanVM execution, amortizing fixed bytecode, statement rebuild, verifier, and proof-system overhead.

- **Hash kernels:** Replace the generic full-guest cell hash path with specialized `hash_cell_8_chunks()`, removing dynamic chunk logic and unnecessary scratch arrays from the hot loop.

- **Difference between `v2_base` and `v2_ext`:** `v2_base` keeps the original field-native 128 KiB blob format, while `v2_ext` uses extension-field blobs.
  - `v2_base` uses $32768$ KoalaBear symbols per blob, $m=65536$, and `dot_product_be` for base-field codewords against extension-field $L$.
  - `v2_ext` uses $8192$ quintic-extension symbols per blob, $m=16384$, and an extension-field membership path over a shorter codeword.
  - `v2_ext` reduces the public $L$ size and membership length by $4\times$, but each extension symbol serializes to five KoalaBear limbs.

## Parameter Comparison

| Demo | Profile | Blob payload | Codeword symbol field | Challenge field | $n$ | $k$ | $m$ | $c$ | $\ell=m/c$ | $t=k/c$ | Opened cells | Sampling bound | Code rate | Membership path | Commitment layout |
| --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| V1 | `blob-128k-1` | 124 KiB | KoalaBear | Quintic extension | 1 | 32768 | 65536 | 64 | 1024 | 512 | 114 | $2^{-124}$ worst-case | $1/2$ | `dot_product_be`, length $m$ | No cell digests |
| V2-base | `blob-128k-1` | 124 KiB | KoalaBear | Quintic extension | 1 | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\mathrm{wor}}=-108.031$ | $1/2$ | `dot_product_be`, length $m$ | cell digest -> row/column commitments |
| V2-ext | `blob-ext-1` | 155 KiB | Quintic extension | Quintic extension | 1 | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\mathrm{rep}}=-83.398$ | $1/2$ | `dot_product_ee`, length $m$ | cell digest -> row/column commitments |

- **Base Payload convention:** V1 and V2-base count one KoalaBear element as four bytes, so $32768$ symbols give $128$ KiB per blob.
- **Extension payload convention:** V2-ext uses logical payload as $8192\cdot5\cdot31$ bits, about $155$ KiB; canonical serialization is $8192\cdot5\cdot4=160$ KiB.
- **Sampling convention:** V1 used the $124$-bit worst-case sampling target; V2 uses the updated V2 subset-soundness parameters and opens $19$ cells.
- **Read-only element convention:** this column counts KoalaBear base-field elements in LeanVM read-only memory. V1 and V2-base use $327696$ elements, i.e. $327696\cdot4=1310784$ bytes, while V2-ext uses $81936$ elements, i.e. $81936\cdot4=327744$ bytes.

## Benchmark Comparison: Single-Blob 

| Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | 
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| V1 | `blob-128k-1` | 124 KiB | 1024 | 327696 | 114 | 0.062 KB | 357.512 KB | 64.570 KB | 0.023s | 0.077s | 3.600s | 0.130s | included | 0.003s | 0.067s | n/a | n/a | n/a | 34.44 KiB/s | 
| V2-base | `blob-128k-1` | 124 KiB | 4096 | 327696 | 19 | 0.06 KB | 278.69 KB | 10.76 KB | 0.017s | 0.114s | 0.649s | 0.112s | 0.032s | 0.001s | 0.067s | 135816 | 8702 | 65536 | 191.06 KiB/s | 
| V2-ext | `blob-ext-1` | 155 KiB | 4096 | 81936 | 19 | 0.06 KB | 249.49 KB | 11.95 KB | 0.021s | 0.062s | 0.496s | 0.060s | 0.032s | 0.001s | 0.030s | 134794 | 10750 | 16384 | 312.50 KiB/s | 
| Tau LeanVM | `n=1` | 155 KiB | 84620 | n/a | n/a | n/a | 255.80 KB | n/a | n/a | n/a | 0.555s | n/a | n/a | n/a | n/a | 86666 | 10753 | 65548 | 279.24 KiB/s | 

## Benchmark Comparison: Multi-Blob

| Demo | Profile | Payload | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| V2-base | `blob-128k-14` | 1736 KiB | 4.049s | 1263288 | 123889 | 917504 | 442.58 KiB/s |
| V2-base | `blob-128k-16` | 1984 KiB | 6.642s | 1403661 | 139247 | 1048576 | 308.34 KiB/s |
| V2-ext | `blob-ext-14` | 2170 KiB | 3.882s | 1248980 | 152561 | 229376 | 559.00 KiB/s |
| V2-ext | `blob-ext-16` | 2480 KiB | 3.939s | 1387309 | 172015 | 262144 | 629.60 KiB/s |
| Tau LeanVM | `n=101` | 15655 KiB | 20.127s | 1571794 | 1113701 | 1703948 | 777.81 KiB/s |

- **Batching effect:** both V2 variants improve throughput when multiple rows are proved in one LeanVM execution because fixed statement and proof overheads are amortized.
- **Best current result:** `blob-ext-16` is the fastest current configuration, reaching $629.60$ KiB/s throughput.
