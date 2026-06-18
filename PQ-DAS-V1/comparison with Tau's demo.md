# Comparison with Tau's Demo

This note compares the current PQ-DAS V1 LeanVM demo with Tau Lepton's LeanAIR demo from
`frisitano/leanMultisig`, commit `232308f2711ee742cada68cca91aa93dfe379655`.

## Current PQ-DAS V1 Parameters

Common parameters for the current demo:

- Base field: KoalaBear.
- Challenge field: quintic extension over KoalaBear.
- Hash: Poseidon-16/8.
- RS rate: `1/2`, so `m = 2k`.
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
| `blob-128k-4` | 4 | 65,536 | 32,768 | 64 | 1,024 | 512 | 9 | 512 KiB | 1 MiB |
| Tau LeanAIR single-proof shape | 101 | 16,384 EF | 8,192 EF | 512 EF | 32 | 16 | n/a | 15,655 KiB, 31-bit limbs; 16,160 KiB, 4-byte limbs | 31,310 KiB, 31-bit limbs; 32,320 KiB, 4-byte limbs |

## PQ-DAS V1: Desktop PC vs Server

The desktop PC results are the original i9-14900 measurements. The server results are the clean post-revert benchmark state on the ax42u server, with the sampling correction retained. Throughput is computed as

$$
{\rm throughput} = \frac{\text{input size}}{\text{LeanVM prove time}}.
$$

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

## Tau LeanAIR Demo: Tau's Server vs Our Server

Tau's demo measures field-native systematic payload throughput. In our notation, Tau's
`n_rows` is `n`, `message_len_ext = 2^log_m` is `k`, `codeword_len_ext = 2k` is `m`, and
`cell_len_ext` is `c`. These are extension-field symbols; each contains 5 KoalaBear base-field limbs.
Tau's code counts each limb as 31 bits.

| Environment | Parameter shape in our notation | Payload | Prove time / throughput | Parallel throughput |
| --- | --- | ---: | ---: | ---: |
| Tau slides, 192-vCPU Graviton4 | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=512 EF`, `m/c=32`, `k/c=16` | about 15.3 MiB | about 7.5 MiB/s | about 25 MiB/s |
| Our ax42u server, single proof | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=512 EF`, `m/c=32`, `k/c=16` | 15,655 KiB | avg 4.96s, 3.08 MiB/s | n/a |
| Our ax42u server, parallel `c4_t4` | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=128 EF`, `m/c=128`, `k/c=64` | batched runs | n/a | 3.40 MiB/s wall, 3.77 MiB/s prove |
| Our ax42u server, parallel `c8_t2` | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=128 EF`, `m/c=128`, `k/c=64` | batched runs | n/a | 3.41 MiB/s wall, 3.71 MiB/s prove |

## Tau LeanAIR vs PQ-DAS V1 on Our Server

This table compares prover throughput on the same ax42u server using the same byte convention:
one KoalaBear limb is counted as 4 bytes. Tau's own benchmark output reports 31-bit limb throughput,
so the normalized Tau payload below is larger by exactly `32/31`.

| Demo | Profile / shape | Payload | Prove time | Prove throughput |
| --- | --- | ---: | ---: | ---: |
| PQ-DAS V1 LeanVM | `large` | 32 KiB | 1.112s | 28.78 KiB/s |
| PQ-DAS V1 LeanVM | `stress` | 256 KiB | 9.327s | 27.45 KiB/s |
| PQ-DAS V1 LeanVM | `blob-128k-1` | 128 KiB | 4.589s | 27.89 KiB/s |
| PQ-DAS V1 LeanVM | `blob-128k-4` | 512 KiB | 18.948s | 27.02 KiB/s |
| Tau LeanAIR | `n=101`, `m=16384 EF`, `k=8192 EF`, `c=512 EF` | 16,160 KiB | avg 4.96s | 3,258.0 KiB/s |

Tau's single-proof prover throughput on our server is roughly

$$
\frac{3258.0}{27.0}\approx 121\times
$$

the throughput of the current PQ-DAS V1 LeanVM path on the large field-native profiles.

## Why Tau's Demo Differs Across Machines

- Tau's reported high-end benchmark uses a 192-vCPU Graviton4 machine, while our ax42u server is a much smaller 8-core / 16-thread class machine.
- The single-proof gap is moderate: about `7.5 MiB/s` on Tau's server versus about `3.08 MiB/s` on ours, or about `2.4x`.
- The parallel aggregate gap is larger because Tau's machine can keep many independent workers alive at once. His reported aggregate throughput is about `25 MiB/s`, while our 16-thread server saturates around `3.4 MiB/s`.
- The `c4_t4` and `c8_t2` runs on our server have nearly identical aggregate wall throughput, which suggests the server is already saturated by memory bandwidth, cache pressure, CPU scheduling, or WHIR/prover parallelism overhead.

## Why Tau's Demo Is Faster Than PQ-DAS V1 on Our Server

- Tau proves a dedicated LeanAIR trace rather than a generic LeanVM execution. This removes instruction decoding, bytecode constraints, memory-bus constraints, stack constraints, public-memory machinery, and generic precompile routing.
- Tau's Poseidon work is represented directly as a deterministic hash table with local AIR constraints. Our demo executes Poseidon through the VM and then proves that VM execution.
- Tau batches many rows/blobs in one proof. His single proof uses `n_rows=101` and about `15.3 MiB` of systematic payload, so fixed proof overhead is heavily amortized.
- Tau binds RS membership to the same committed trace columns that feed the hash computation. The row parity claim reads the exact trace positions where codeword limbs are absorbed, avoiding a second independent codeword copy.
- The current PQ-DAS V1 prover spends most time in LeanVM proof-system stages, especially stack/commit, logup, AIR sumcheck, and WHIR. Host-side encoding, commitment, opening verification, and reconstruction are not the dominant bottlenecks.
- The current PQ-DAS V1 profiles have stable LeanVM throughput around `27-29 KiB/s` on the server, which indicates a VM-layer unit-cost bottleneck rather than an isolated inefficient host function.
