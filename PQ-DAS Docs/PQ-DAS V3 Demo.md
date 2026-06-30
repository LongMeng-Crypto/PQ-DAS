# PQ-DAS V3 Demo

V3 keeps the V2 cell-digest commitment path, but replaces the public commitment components with one final digest. The row hashes are Merkle-aggregated into $root_{\rm row}$, the column roots are Merkle-aggregated into $root_{\rm col}$, and the public commitment is $root = H(root_{\rm row}, root_{\rm col})$. This note records the V3-base and V3-ext benchmarks, including the doubled-blob profiles, and compares them with V1, V2, and Tau's LeanVM reference numbers.

## Difference from V2

- **Final commitment:** V2 exposes row hashes and the column root as public commitment data, while V3 exposes only the final $root$.
- **Row aggregation:** V3 adds a row Merkle tree over the systematic row hashes, producing $root_{\rm row}$ inside the proof.
- **Final aggregation:** V3 hashes $root_{\rm row}$ and $root_{\rm col}$ into the final public commitment root.
- **Verifier statement:** V3 derives the Fiat-Shamir challenge and public $L$ vector from the public parameters and final $root$.
- **Opening path:** V3 openings include the opened column path to $root_{\rm col}$ plus $root_{\rm row}$, then verify the final aggregation to $root$.
- **2x profiles:** The doubled-blob profiles double $k$, $m$, and $c$, while keeping $\ell=m/c=1024$ and $t=k/c=512$ unchanged.

## Payload Convention

- **Effective payload:** one KoalaBear symbol contributes $31$ bits, and one quintic-extension symbol contributes $5\cdot31$ bits.
- **Canonical serialization:** one KoalaBear limb occupies four bytes, so base rows are $128$ KiB or $256$ KiB on disk, while extension rows are $160$ KiB or $320$ KiB on disk.
- **Throughput column:** all tables below use effective payload throughput, unless explicitly marked canonical.

## Input Parameters: V1

| Demo | Profile | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Sampling bound | Membership path | Public commitment |
| --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| V1 | `blob-128k-1` | 1 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 114 | $2^{-124}$ worst-case | `dot_product_be`, length $m$ | row hashes + column root |
| V1 | `blob-128k-4` | 4 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 114 | $2^{-124}$ worst-case | `dot_product_be`, length $m$ | row hashes + column root |

## Input Parameters: V2

| Demo | Profile | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Sampling bound | Membership path | Public commitment |
| --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| V2-base | `blob-128k-{1,14,16}` | 1, 14, 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | row hashes + column root |
| V2-ext | `blob-ext-{1,14,16}` | 1, 14, 16 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | row hashes + column root |

## Input Parameters: V3

| Demo | Profile | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Sampling bound | Membership path | Public commitment |
| --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| V3-base | `blob-128k-1` | 1 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | final root only |
| V3-base | `blob-128k-14` | 14 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | final root only |
| V3-base | `blob-128k-16` | 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | final root only |
| V3-base 2x | `blob-256k-1` | 1 | KoalaBear | Quintic extension | 65536 | 131072 | 128 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | final root only |
| V3-base 2x | `blob-256k-14` | 14 | KoalaBear | Quintic extension | 65536 | 131072 | 128 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | final root only |
| V3-base 2x | `blob-256k-16` | 16 | KoalaBear | Quintic extension | 65536 | 131072 | 128 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | `dot_product_be`, length $m$ | final root only |
| V3-ext | `blob-ext-1` | 1 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | final root only |
| V3-ext | `blob-ext-14` | 14 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | final root only |
| V3-ext | `blob-ext-16` | 16 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | final root only |
| V3-ext 2x | `blob-ext-2x-1` | 1 | Quintic extension | Quintic extension | 16384 | 32768 | 32 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | final root only |
| V3-ext 2x | `blob-ext-2x-14` | 14 | Quintic extension | Quintic extension | 16384 | 32768 | 32 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | final root only |
| V3-ext 2x | `blob-ext-2x-16` | 16 | Quintic extension | Quintic extension | 16384 | 32768 | 32 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | `dot_product_ee`, length $m$ | final root only |

## Input Parameters: Tau LeanVM

| Demo | Profile | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Membership path | Public commitment |
| --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Tau LeanVM | `n=1` | 1 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | n/a | `dot_product_ee`, structured even/odd path | final root |
| Tau LeanVM | `n=101` | 101 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | n/a | `dot_product_ee`, structured even/odd path | final root |

## Benchmark: V1

| Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | Result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| V1 | `blob-128k-1` | 124 KiB | 1024 | 327696 | 114 | 0.062 KB | 357.512 KB | 64.570 KB | 0.023s | 0.077s | 3.600s | 0.130s | included | 0.003s | 0.067s | n/a | n/a | n/a | 34.44 KiB/s | accepted |
| V1 | `blob-128k-4` | 496 KiB | 1024 | 327720 | 114 | 0.156 KB | 398.496 KB | 150.070 KB | 0.097s | 0.079s | 17.400s | 0.209s | included | 0.008s | 0.143s | n/a | n/a | n/a | 28.51 KiB/s | accepted |

## Benchmark: V2

| Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | Result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| V2-base | `blob-128k-1` | 124 KiB | 4096 | 327696 | 19 | 0.06 KB | 278.69 KB | 10.76 KB | 0.017s | 0.114s | 0.649s | 0.112s | 0.032s | 0.001s | 0.067s | 135816 | 8702 | 65536 | 191.06 KiB/s | accepted |
| V2-base | `blob-128k-14` | 1736 KiB | 4096 | 327800 | 19 | 0.47 KB | 333.95 KB | 72.51 KB | 0.228s | 0.111s | 4.049s | 0.127s | 0.044s | 0.004s | 0.237s | 1263288 | 123889 | 917504 | 428.75 KiB/s | accepted |
| V2-base | `blob-128k-16` | 1984 KiB | 4096 | 327816 | 19 | 0.53 KB | 328.42 KB | 82.01 KB | 0.255s | 0.109s | 6.642s | 0.126s | 0.044s | 0.005s | 0.283s | 1403661 | 139247 | 1048576 | 298.70 KiB/s | accepted |
| V2-ext | `blob-ext-1` | 155 KiB | 4096 | 81936 | 19 | 0.06 KB | 249.49 KB | 11.95 KB | 0.021s | 0.062s | 0.496s | 0.060s | 0.032s | 0.001s | 0.030s | 134794 | 10750 | 16384 | 312.50 KiB/s | accepted |
| V2-ext | `blob-ext-14` | 2170 KiB | 4096 | 82040 | 19 | 0.47 KB | 327.61 KB | 89.14 KB | 0.239s | 0.060s | 3.882s | 0.077s | 0.044s | 0.005s | 0.169s | 1248980 | 152561 | 229376 | 559.00 KiB/s | accepted |
| V2-ext | `blob-ext-16` | 2480 KiB | 4096 | 82056 | 19 | 0.53 KB | 332.70 KB | 101.01 KB | 0.275s | 0.062s | 3.939s | 0.069s | 0.041s | 0.005s | 0.191s | 1387309 | 172015 | 262144 | 629.60 KiB/s | accepted |

## Benchmark: V3 Single-Blob Size

| Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | Result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| V3-base | `blob-128k-1` | 124 KiB | 4096 | 327688 | 19 | 0.03 KB | 277.38 KB | 11.36 KB | 0.017s | 0.120s | 0.654s | 0.126s | 0.032s | 0.001s | 0.063s | 135816 | 8703 | 65536 | 189.60 KiB/s | accepted |
| V3-base | `blob-128k-14` | 1736 KiB | 4096 | 327688 | 19 | 0.03 KB | 334.26 KB | 73.11 KB | 0.226s | 0.113s | 4.011s | 0.116s | 0.040s | 0.004s | 0.232s | 1263336 | 123905 | 917504 | 432.81 KiB/s | accepted |
| V3-base | `blob-128k-16` | 1984 KiB | 4096 | 327688 | 19 | 0.03 KB | 328.42 KB | 82.61 KB | 0.262s | 0.117s | 6.395s | 0.121s | 0.043s | 0.006s | 0.283s | 1403691 | 139263 | 1048576 | 310.24 KiB/s | accepted |
| V3-ext | `blob-ext-1` | 155 KiB | 4096 | 81928 | 19 | 0.03 KB | 249.31 KB | 12.54 KB | 0.021s | 0.082s | 0.576s | 0.069s | 0.037s | 0.001s | 0.036s | 141962 | 10751 | 16384 | 269.10 KiB/s | accepted |
| V3-ext | `blob-ext-14` | 2170 KiB | 4096 | 81928 | 19 | 0.03 KB | 325.89 KB | 89.73 KB | 0.287s | 0.069s | 4.796s | 0.074s | 0.046s | 0.005s | 0.186s | 1349380 | 152577 | 229376 | 452.46 KiB/s | accepted |
| V3-ext | `blob-ext-16` | 2480 KiB | 4096 | 81928 | 19 | 0.03 KB | 333.13 KB | 101.61 KB | 0.315s | 0.069s | 4.219s | 0.072s | 0.044s | 0.006s | 0.196s | 1502027 | 172031 | 262144 | 587.82 KiB/s | accepted |

## Benchmark: V3 Double-Blob Size

| Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | Result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| V3-base 2x | `blob-256k-1` | 248 KiB | 4096 | 655368 | 19 | 0.03 KB | 276.18 KB | 16.11 KB | 0.034s | 0.193s | 0.985s | 0.205s | 0.037s | 0.001s | 0.150s | 160392 | 16895 | 131072 | 251.78 KiB/s | accepted |
| V3-base 2x | `blob-256k-14` | 3472 KiB | 4096 | 655368 | 19 | 0.03 KB | 331.42 KB | 139.61 KB | 0.472s | 0.196s | 7.274s | 0.207s | 0.045s | 0.008s | 0.524s | 1607400 | 238593 | 1835008 | 477.32 KiB/s | accepted |
| V3-base 2x | `blob-256k-16` | 3968 KiB | 4096 | 655368 | 19 | 0.03 KB | 378.32 KB | 158.61 KB | 0.566s | 0.198s | 9.342s | 0.222s | 0.047s | 0.010s | 0.602s | 1796907 | 270335 | 2097152 | 424.75 KiB/s | accepted |
| V3-ext 2x | `blob-ext-2x-1` | 310 KiB | 4096 | 163848 | 19 | 0.03 KB | 277.99 KB | 18.48 KB | 0.041s | 0.091s | 0.672s | 0.095s | 0.038s | 0.001s | 0.057s | 172682 | 20991 | 32768 | 461.31 KiB/s | accepted |
| V3-ext 2x | `blob-ext-2x-14` | 4340 KiB | 4096 | 163848 | 19 | 0.03 KB | 367.35 KB | 172.86 KB | 0.556s | 0.092s | 5.358s | 0.101s | 0.045s | 0.010s | 0.361s | 1779460 | 295937 | 458752 | 809.81 KiB/s | accepted |
| V3-ext 2x | `blob-ext-2x-16` | 4960 KiB | 4096 | 163848 | 19 | 0.03 KB | 331.98 KB | 196.61 KB | 0.632s | 0.088s | 8.146s | 0.102s | 0.048s | 0.012s | 0.413s | 1993547 | 335871 | 524288 | 608.89 KiB/s | accepted |

## Benchmark: Tau LeanVM

| Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | Result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Tau LeanVM | `n=1` | 155 KiB | 84620 | n/a | n/a | n/a | 255.80 KB | n/a | 0.555s | 86666 | 10753 | 65548 | 279.24 KiB/s | accepted |
| Tau LeanVM | `n=101` | 15655 KiB | 1309906 | n/a | n/a | n/a | 377.08 KB | n/a | 20.127s | 1571794 | 1113701 | 1703948 | 777.81 KiB/s | accepted |


## Short Takeaways

- **V3 commitment size:** V3 reduces the public commitment to $32$ bytes, shown as $0.03$ KB.
- **2x sampling:** doubled profiles keep $\ell=1024$ and $t=512$, so opened cells remain $19$ while sample size increases with cell size.
- **Read-only scaling:** V3-base doubles from $327688$ to $655368$ read-only KoalaBear elements; V3-ext doubles from $81928$ to $163848$.
- **Best one-shot PQ-DAS result:** `blob-ext-2x-14` reaches $809.81$ KiB/s effective-payload proving throughput in this run.
- **14-row sweet spot:** $n=14$ often outperforms $n=16$ in wall-clock throughput because both pad to $N_{\rm padded}=16$, while $n=14$ has lower real row work and lower memory pressure.
