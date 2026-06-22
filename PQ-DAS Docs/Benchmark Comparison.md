# Benchmark comparisons

This note compares the our PQ-DAS with Tau's LeanAIR demo and Tau's LeanVM demo.

## Parameter comparison

Common parameters for the current demo:

- Base field: KoalaBear.
- Challenge field: quintic extension over KoalaBear.
- Hash: Poseidon-16/8.
- RS rate: `1/2`, `m = 2k`.
- RS membership check: special barycentric parity identity.
- WHIR inverse-rate exponent: `1`.
- Blob symbols are field-native KoalaBear elements. The 128 KiB profiles use `k = 32768`, counted as `32768 * 4` bytes.
- Current DAS sampling uses the formal worst-case bound with `T = 128` accepting transcripts, so the opened-cell counts below are the reduced post-correction values.
- Tau's benchmark uses quintic-extension symbols for `k`, `m`, and `c`; one Tau extension symbol contains 5 KoalaBear base-field limbs.

| Dataset | n | m | k | c | Cells `m/c` | Reconstruction threshold `k/c` | Opened cells | Input size | Encoded size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 2 | 16 | 8 | 8 | 2 | 1 | 1 | 64 B | 128 B |
| `medium` | 8 | 256 | 128 | 8 | 32 | 16 | 2 | 4 KiB | 8 KiB |
| `large` | 16 | 1,024 | 512 | 8 | 128 | 64 | 2 | 32 KiB | 64 KiB |
| `stress` | 32 | 4,096 | 2,048 | 8 | 512 | 256 | 5 | 256 KiB | 512 KiB |
| `blob-128k-1` | 1 | 65,536 | 32,768 | 64 | 1,024 | 512 | 9 | 128 KiB | 256 KiB |
| `blob-128k-4` | 4 | 65,536 | 32,768 | 64 | 1,024 | 512 | 9 | 512 KiB | 1024 KiB |
| Tau LeanVM | 101 | 16,384 Ext | 8,192 Ext | 16 Ext | 1024 | 512 | n/a | 15,655 KiB | 31,310 KiB |
| Tau LeanAIR | 101 | 16,384 Ext | 8,192 Ext | 512 Ext | 32 | 16 | n/a | 15,655 KiB | 31,310 KiB |

## PQ-DAS V1: Desktop PC vs Server

The following benchmark shows a benchamrk comparion between a desktop PC and a server.

### Desktop PC: i9-14900

The PC timings are the original run. The opened-cell column is shown with the current reduced sampling count; LeanVM prove throughput is unaffected by the sampling correction.

| Profile | Opened cells | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings | Reconstruct | LeanVM prove throughput |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 1 | 0.000s | 0.005s | 0.100s | 0.028s | 0.000s | 0.000s | 0.63 KiB/s |
| `medium` | 2 | 0.001s | 0.006s | 0.200s | 0.035s | 0.000s | 0.000s | 20.00 KiB/s |
| `large` | 2 | 0.006s | 0.015s | 0.915s | 0.045s | 0.002s | 0.003s | 34.97 KiB/s |
| `stress` | 5 | 0.043s | 0.016s | 6.812s | 0.055s | 0.006s | 0.025s | 37.58 KiB/s |
| `blob-128k-1` | 9 | 0.023s | 0.077s | 3.600s | 0.130s | 0.003s | 0.067s | 35.56 KiB/s |
| `blob-128k-4` | 9 | 0.097s | 0.079s | 17.400s | 0.209s | 0.008s | 0.143s | 29.43 KiB/s |

### Server: ax42u

| Profile | Opened cells | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings | Reconstruct | LeanVM prove throughput |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 1 | 0.000s | 0.006s | 0.030s | 0.027s | 0.000s | 0.000s | 2.08 KiB/s |
| `medium` | 2 | 0.001s | 0.005s | 0.169s | 0.034s | 0.000s | 0.001s | 23.67 KiB/s |
| `large` | 2 | 0.006s | 0.010s | 1.112s | 0.043s | 0.000s | 0.004s | 28.78 KiB/s |
| `stress` | 5 | 0.047s | 0.014s | 9.327s | 0.056s | 0.000s | 0.029s | 27.45 KiB/s |
| `blob-128k-1` | 9 | 0.026s | 0.083s | 4.589s | 0.125s | 0.000s | 0.070s | 27.89 KiB/s |
| `blob-128k-4` | 9 | 0.095s | 0.083s | 18.948s | 0.124s | 0.001s | 0.111s | 27.02 KiB/s |

## Tau LeanAIR: Tau's Server vs Our Server

Tau's demo measures field-native systematic payload throughput. In our notation, Tau's `n_rows` is `n`, `message_len_ext = 2^log_m` is `k`, `codeword_len_ext = 2k` is `m`, and `cell_len_ext` is `c`. These are extension-field symbols; each contains 5 KoalaBear base-field limbs. Tau's code counts each limb as 31 bits.
- Tau reports `k_EF * 5 * 31 = 8192 * 5 * 31 = 1,269,760` bits per row, or about `155 KiB`. 
- We report `k * 32 = 32768 * 32 = 1,048,576 = 128 KiB` bits per row. 
- Thus one row from Tau's demo is about `1.21x` larger than one row from our demo.

| Environment | Mode | Threads | Payload | Total throughput |
| --- | --- | ---: | ---: | ---: |
| 192-vCPU Graviton4 | single proof | 1 | 15,655 KiB | about 7.5 MiB/s |
| 192-vCPU Graviton4 | parallel batch | 192 | 3,005,760 KiB | about 25 MiB/s |
| Our ax42u server | single proof | 1 | 15,655 KiB | 3.08 MiB/s |
| Our ax42u server | parallel batch | 16 | 250,480 KiB | 3.41 MiB/s |


## Tau LeanVM on Our Server

This benchmark runs Tau's generic LeanVM `column-commit` construction. Each blob contains `k = 8192` extension-field message coefficients and is encoded to `m = 16384` extension-field evaluations. The construction uses `c = 16` extension elements per cell, so each codeword contains 1,024 cells. It proves the row/column commitment and the RS barycentric check.

| Blobs `n` | Payload | Bytecode size | Cycles | Poseidon16 calls | ExtensionOp calls | LeanVM prove | Proof size | Prove throughput | Peak RSS | Total wall time |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 155 KiB | 84,620 | 86,666 | 10,753 | 65,548 | 0.555s | 255.80 KiB | 279.24 KiB/s | 1.05 GiB | 2.10s |
| 101 | 15,655 KiB | 1,309,906 | 1,571,794 | 1,113,701 | 1,703,948 | 20.127s | 377.08 KiB | 777.81 KiB/s | 29.87 GiB | 55.28s |



## Tau LeanVM, Tau LeanAIR, and PQ-DAS V1 

This table compares three demos together: Tau's LeanVM, Tau's LeanAIR, and our PQ-DAS V1.

| Demo | Profile / shape | Payload | Prove time | Prove throughput |
| --- | --- | ---: | ---: | ---: |
| PQ-DAS V1 LeanVM | `large` | 32 KiB | 1.112s | 28.78 KiB/s |
| PQ-DAS V1 LeanVM | `stress` | 256 KiB | 9.327s | 27.45 KiB/s |
| PQ-DAS V1 LeanVM | `blob-128k-1` | 128 KiB | 4.589s | 27.89 KiB/s |
| PQ-DAS V1 LeanVM | `blob-128k-4` | 512 KiB | 18.948s | 27.02 KiB/s |
| Tau LeanVM | `n=1`, `m=16384 EF`, `k=8192 EF`, `c=16 EF` | 155 KiB | 0.555s | 279.24 KiB/s |
| Tau LeanVM | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=16 EF` | 15,655 KiB | 20.127s | 777.81 KiB/s |
| Tau LeanAIR | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=512 EF` | 15,655 KiB | 4.96s | 3,156.8 KiB/s |

- Tau LeanVM with one blob is `279.24 / 27.89 = 10.01x` the throughput of PQ-DAS V1 with one blob.
- Tau LeanVM with 101 blobs is `777.81 / 279.24 = 2.79x` the throughput of Tau LeanVM with one blob.
- Tau LeanAIR with 101 blobs is `3156.8 / 777.81 = 4.06x` the throughput of Tau LeanVM with 101 blobs.
- Tau LeanAIR with 101 blobs is `3156.8 / 27.89 = 113.19x` the throughput of PQ-DAS V1 with one blob.

## Result analysis
### Tau LeanAIR hardware differences 

- Tau's reported high-end benchmark uses a 192-vCPU Graviton4 machine, while our ax42u server is a much smaller 8-core / 16-thread class machine.
- The single-proof gap is moderate: about `7.5 MiB/s` on Tau's server versus about `3.08 MiB/s` on ours, or about `2.4x`.
- The parallel aggregate gap is larger because Tau's machine can keep many independent workers alive at once. His reported aggregate throughput is about `25 MiB/s`, while our 16-thread server saturates around `3.4 MiB/s`.

### Tau LeanAIR vs PQ-DAS V1

- LeanAIR proves a dedicated AIR trace rather than a generic LeanVM execution. 
    - This removes instruction decoding, bytecode constraints, memory-bus constraints, stack constraints, public-memory machinery, and generic precompile routing. 
    - All poseidon hashes are represented as deterministic hash tables with local AIR constraints. Our demo executes Poseidon through the VM and then proves that VM execution.
- LeanAIR proves a more efficient commitment layout
    - Hash each cell into cell digest
    - Hash chain the systmatic digests on each row into a row digest 
    - Hash chain all row digests into an aggregated row digest
    - Merkle commits all digests on each column into a column root
    - Merkle commits all column roots into an aggregated root
    - Hash the aggregated row digest and column root into a final digest
- LeanAIR batches many rows/blobs in one proof
    - A single proof uses `n_rows=101` and about `15.3 MiB` of systematic payload, so fixed proof overhead is heavily amortized.
- LeanAIR binds RS membership to the same committed trace columns that feed the hash computation. The row parity claim reads the exact trace positions where codeword limbs are absorbed, avoiding a second independent codeword copy.

### Tau LeanVM vs PQ-DAS V1

#### Why Tau LeanVM is faster
- Tau LeanVM also batches 101-row proof for amortizing shared VM and proof-system costs. Its throughput is `2.79x` the one-row run, and its measured cycle count falls from 86,666 per blob to about 15,562 per blob.
- Tau LeanVM proves the same commitment layout as in LeanAIR.
- Tau LeanVM uses fixed-length cell compression without a length-IV sponge call.

#### Tau LeanVM different building blocks
- **RS encoding:** Tau interprets each blob `D = (d_0, ..., d_8191)` as the coefficients of an extension-field polynomial `f(X)`, zero-pads the coefficient vector to length 16,384, and applies one forward NTT to obtain `W_j = f(omega^j)`. The witness is reordered as all even evaluations followed by all odd evaluations; the first half is an information set, not the original coefficient vector.
- **Barycentric-vector construction:** Tau derives the extension-field challenge from the final public commitment root and constructs both barycentric vectors inside the LeanVM guest. This costs 49,164 ExtensionOp rows once, after which the same vectors are reused across all codeword rows. PQ-DAS V1 instead computes its check vector during host-side statement preparation and binds it as read-only memory.
- **Extension-field inner products:** For each codeword, Tau executes two length-8,192 `dot_product_ee` calls, one for the even evaluations and one for the odd evaluations, and constrains both results to be equal. Each input element is five KoalaBear limbs; one ExtensionOp trace row enforces the complete multiplication in `E = F[X]/(X^5 + X^2 - 1)` plus the running extension-field accumulation. The RS check therefore uses 16,384 ExtensionOp rows per codeword.


