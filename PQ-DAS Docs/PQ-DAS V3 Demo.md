# PQ-DAS V3 Demo

V3 keeps the V2 cell-digest commitment path, but replaces the public commitment components with one final digest. The row hashes are Merkle-aggregated into $root_{\rm row}$, the column roots are Merkle-aggregated into $root_{\rm col}$, and the public commitment is $root = H(root_{\rm row}, root_{\rm col})$. The benchmark below compares V1, V2-base, V2-ext, V3-base, V3-ext, and Tau's LeanVM reference numbers on the same LeanVM-style metrics when available.

## Difference from V2

- **Final commitment:** V2 exposes row hashes and the column root as public commitment data, while V3 exposes only the final $root$.
- **Row aggregation:** V3 adds a row Merkle tree over the systematic row hashes, producing $root_{\rm row}$ inside the proof.
- **Final aggregation:** V3 hashes $root_{\rm row}$ and $root_{\rm col}$ into the final public commitment root.
- **Verifier statement:** V3 derives the Fiat-Shamir challenge and the public $L$ vector from the public parameters and final $root$, so verifier-side statement rebuild needs only the final root.
- **Opening path:** V3 openings include the opened column path to $root_{\rm col}$ plus $root_{\rm row}$, then verify the final aggregation to $root$.
- **Proof cost:** V3 adds only a small number of Poseidon calls for row aggregation and final aggregation; the main costs remain cell hashing, column Merkle hashing, and RS membership.

## Input Parameters

Payload is measured as field-native effective payload: one KoalaBear symbol contributes $31$ bits, and one quintic-extension symbol contributes $5\cdot31$ bits. Canonical serialization would use four bytes per KoalaBear limb, so base rows are $128$ KiB on disk and extension rows are $160$ KiB on disk.

| Demo | Profile | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell=m/c$ | Threshold $t=k/c$ | Opened cells | Sampling bound | Code rate | Membership path | Public commitment |
| --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| V1 | `blob-128k-1` | 1 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 114 | $2^{-124}$ worst-case | $1/2$ | `dot_product_be`, length $m$ | row hashes + column root |
| V1 | `blob-128k-4` | 4 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 114 | $2^{-124}$ worst-case | $1/2$ | `dot_product_be`, length $m$ | row hashes + column root |
| V2-base | `blob-128k-{1,14,16}` | 1, 14, 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | $1/2$ | `dot_product_be`, length $m$ | row hashes + column root |
| V2-ext | `blob-ext-{1,14,16}` | 1, 14, 16 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | $1/2$ | `dot_product_ee`, length $m$ | row hashes + column root |
| V3-base | `blob-128k-{1,14,16}` | 1, 14, 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | $\log_2\nu_{\rm wor}=-108.031$ | $1/2$ | `dot_product_be`, length $m$ | final root only |
| V3-ext | `blob-ext-{1,14,16}` | 1, 14, 16 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | $\log_2\nu_{\rm rep}=-83.398$ | $1/2$ | `dot_product_ee`, length $m$ | final root only |
| Tau LeanVM | `n=1` | 1 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | n/a | n/a | $1/2$ | `dot_product_ee`, structured even/odd path | final root |
| Tau LeanVM | `n=101` | 101 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | n/a | n/a | $1/2$ | `dot_product_ee`, structured even/odd path | final root |

## Benchmark Results

The table uses effective payload for throughput. Unknown Tau fields are marked `n/a` because Tau's public benchmark log reports a different set of metrics.

| Group | Demo | Profile | Payload | Bytecode instructions | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Prove throughput | Result |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Single blob | V1 | `blob-128k-1` | 124 KiB | 1024 | 327696 | 114 | 0.062 KB | 357.512 KB | 64.570 KB | 0.023s | 0.077s | 3.600s | n/a | 0.130s | included | 0.003s | 0.067s | n/a | n/a | n/a | 34.44 KiB/s | accepted |
| Single blob | V2-base | `blob-128k-1` | 124 KiB | 4096 | 327696 | 19 | 0.06 KB | 278.69 KB | 10.76 KB | 0.017s | 0.114s | 0.649s | 0.000s | 0.112s | 0.032s | 0.001s | 0.067s | 135816 | 8702 | 65536 | 191.06 KiB/s | accepted |
| Single blob | V2-ext | `blob-ext-1` | 155 KiB | 4096 | 81936 | 19 | 0.06 KB | 249.49 KB | 11.95 KB | 0.021s | 0.062s | 0.496s | 0.000s | 0.060s | 0.032s | 0.001s | 0.030s | 134794 | 10750 | 16384 | 312.50 KiB/s | accepted |
| Single blob | V3-base | `blob-128k-1` | 124 KiB | 4096 | 327688 | 19 | 0.03 KB | 279.06 KB | 11.36 KB | 0.017s | 0.111s | 0.633s | 0.000s | 0.110s | 0.033s | 0.001s | 0.064s | 135816 | 8703 | 65536 | 195.89 KiB/s | accepted |
| Single blob | V3-ext | `blob-ext-1` | 155 KiB | 4096 | 81928 | 19 | 0.03 KB | 248.18 KB | 12.54 KB | 0.019s | 0.062s | 0.510s | 0.000s | 0.063s | 0.033s | 0.001s | 0.026s | 134794 | 10751 | 16384 | 303.92 KiB/s | accepted |
| Single blob | Tau LeanVM | `n=1` | 155 KiB | 84620 | n/a | n/a | n/a | 255.80 KB | n/a | n/a | n/a | 0.555s | n/a | n/a | n/a | n/a | n/a | 86666 | 10753 | 65548 | 279.24 KiB/s | accepted |
| Multi blob | V1 | `blob-128k-4` | 496 KiB | 1024 | 327720 | 114 | 0.156 KB | 398.496 KB | 150.070 KB | 0.097s | 0.079s | 17.400s | n/a | 0.209s | included | 0.008s | 0.143s | n/a | n/a | n/a | 28.51 KiB/s | accepted |
| Multi blob | V2-base | `blob-128k-14` | 1736 KiB | 4096 | 327800 | 19 | 0.47 KB | 333.95 KB | 72.51 KB | 0.228s | 0.111s | 4.049s | 0.000s | 0.127s | 0.044s | 0.004s | 0.237s | 1263288 | 123889 | 917504 | 428.75 KiB/s | accepted |
| Multi blob | V2-base | `blob-128k-16` | 1984 KiB | 4096 | 327816 | 19 | 0.53 KB | 328.42 KB | 82.01 KB | 0.255s | 0.109s | 6.642s | 0.000s | 0.126s | 0.044s | 0.005s | 0.283s | 1403661 | 139247 | 1048576 | 298.70 KiB/s | accepted |
| Multi blob | V2-ext | `blob-ext-14` | 2170 KiB | 4096 | 82040 | 19 | 0.47 KB | 327.61 KB | 89.14 KB | 0.239s | 0.060s | 3.882s | 0.000s | 0.077s | 0.044s | 0.005s | 0.169s | 1248980 | 152561 | 229376 | 559.00 KiB/s | accepted |
| Multi blob | V2-ext | `blob-ext-16` | 2480 KiB | 4096 | 82056 | 19 | 0.53 KB | 332.70 KB | 101.01 KB | 0.275s | 0.062s | 3.939s | 0.000s | 0.069s | 0.041s | 0.005s | 0.191s | 1387309 | 172015 | 262144 | 629.60 KiB/s | accepted |
| Multi blob | V3-base | `blob-128k-14` | 1736 KiB | 4096 | 327688 | 19 | 0.03 KB | 336.35 KB | 73.11 KB | 0.228s | 0.113s | 3.827s | 0.000s | 0.113s | 0.037s | 0.004s | 0.226s | 1263336 | 123905 | 917504 | 453.35 KiB/s | accepted |
| Multi blob | V3-base | `blob-128k-16` | 1984 KiB | 4096 | 327688 | 19 | 0.03 KB | 328.80 KB | 82.61 KB | 0.258s | 0.111s | 6.195s | 0.000s | 0.121s | 0.044s | 0.004s | 0.271s | 1403691 | 139263 | 1048576 | 320.26 KiB/s | accepted |
| Multi blob | V3-ext | `blob-ext-14` | 2170 KiB | 4096 | 81928 | 19 | 0.03 KB | 325.58 KB | 89.73 KB | 0.243s | 0.061s | 3.959s | 0.000s | 0.068s | 0.041s | 0.005s | 0.166s | 1249028 | 152577 | 229376 | 548.12 KiB/s | accepted |
| Multi blob | V3-ext | `blob-ext-16` | 2480 KiB | 4096 | 81928 | 19 | 0.03 KB | 333.29 KB | 101.61 KB | 0.290s | 0.064s | 3.964s | 0.000s | 0.071s | 0.043s | 0.006s | 0.187s | 1387339 | 172031 | 262144 | 625.63 KiB/s | accepted |
| Multi blob | Tau LeanVM | `n=101` | 15655 KiB | 1309906 | n/a | n/a | n/a | 377.08 KB | n/a | n/a | n/a | 20.127s | n/a | n/a | n/a | n/a | n/a | 1571794 | 1113701 | 1703948 | 777.81 KiB/s | accepted |

## Short Takeaways

- **V3 commitment size:** V3 reduces the public commitment to $32$ bytes, shown as $0.03$ KB.
- **V3 read-only size:** V3-base read-only memory is $327688=327680+8$ KoalaBear elements, and V3-ext is $81928=81920+8$.
- **V3 overhead:** V3 adds only a tiny Poseidon overhead relative to V2 because row aggregation and final aggregation are small compared with cell hashing and membership.
- **Best PQ-DAS result:** V3-ext with $16$ rows reaches $625.63$ KiB/s effective-payload proving throughput in this run.
- **Tau comparison:** Tau LeanVM remains faster on the large batch, but V3-ext is now in the same order of magnitude while preserving the PQ-DAS V3 statement structure.
