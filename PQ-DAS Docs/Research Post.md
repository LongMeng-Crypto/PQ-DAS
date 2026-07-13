# PQ-DAS Research Report

<style>
table th, table td { white-space: nowrap; }
</style>

## 1. Motivation

Data availability sampling (DAS) is the mechanism that lets light clients gain confidence that block data has actually been published, without downloading the full data. A block producer encodes the data with an erasure code, commits to the encoded data, and light clients randomly sample small parts of the encoding. If enough honest clients can later combine their openings, the original data can be reconstructed; if the producer withholds too much data, random sampling should detect it with high probability.

The post-quantum motivation is that many modern DAS designs use succinct polynomial commitments such as KZG. These are efficient, but they rely on elliptic-curve pairing assumptions that are not post-quantum. A PQ-DAS design should therefore avoid discrete-log and pairing assumptions, while still preserving the practical properties that make DAS useful: small samples, small verifier work, efficient reconstruction, and compatibility with future Ethereum-style data availability pipelines.

The central engineering question in this work was: can we build a practical post-quantum DAS construction by combining erasure coding with a post-quantum proof system, and can that construction be made fast enough inside LeanVM?

## 2. Solution Space

The post-quantum DAS design space can be grouped into several families.

- **Merkle-only erasure-coded DAS:** Encode the data, commit to encoded chunks with a Merkle tree, and let clients sample chunks. This is post-quantum under collision-resistant hashing, but without an additional proof the client does not know that the committed chunks form a valid codeword. The data may be unrecoverable even if sampled chunks authenticate correctly.

- **Fraud-proof DAS:** Commit to encoded data and rely on validators to publish fraud proofs when the encoding is invalid. This can reduce proving cost in the optimistic case, but availability depends on an interactive or network-timing assumption: someone must detect and publish the fraud proof before clients accept.

- **Validity-proof DAS:** Prove that the committed data is a valid encoding. This avoids optimistic fraud-proof assumptions and fits naturally with STARK/FRI-style post-quantum proof systems. Our work belongs to this family.

- **Foundations of DAS constructions:** The 2023 foundations line formalizes DAS security definitions such as soundness, extractability, and subset-soundness. The constructions in that space clarify what it means for random sampling to imply reconstructability, and they separate sampling with replacement from sampling without replacement.

- **FRIDA/ZODA-style post-quantum DAS:** These later directions use post-quantum transparent proof or proximity machinery to avoid pairing-based commitments. They are important reference points because they show that post-quantum DAS can be built without KZG. However, they do not directly give the operational property we wanted to prioritize in this implementation: practical repairability from opened cells in an Ethereum-style workflow.

The reason we focused on **Encode + Prove** is repairability.

- **Repairability** means that after enough clients or peers have opened authenticated cells, the system can reconstruct the original blob data from those actual opened cells. It is not enough to prove that a commitment is close to some codeword; the protocol should let the network use sampled/opened material as repair material.

- Merkle-only constructions do not prove that opened cells lie on a valid RS codeword, so reconstruction may fail even after enough openings.

- Fraud-proof constructions can be practical, but repairability depends on the absence of undetected invalid encodings and on timely fraud proof propagation.

- Proximity-only constructions can prove closeness or consistency properties, but the verifier may not receive the concrete set of authenticated RS evaluations needed for straightforward repair.

- Encode + Prove gives the cleanest repair path: the proof binds the entire committed matrix to valid RS codewords, and the opening protocol returns authenticated codeword cells. Once the union of verified opened cells reaches the reconstruction threshold, the original blobs can be decoded.

## 3. Encode + Prove Methodology

The construction family implemented here follows the same high-level pattern in every version.

- **Encoding:** Each blob is encoded as one RS codeword. In the optimized demos, the systematic data is placed in a layout compatible with FFT-based encoding and arbitrary-cell erasure reconstruction.

- **Cell digest layer:** Each codeword row is split into cells. Every cell is hashed into one digest. This cell-digest matrix is reused for both row commitment and column commitment.

- **Row commitment:** Systematic cell digests are committed per row. In V3, all row hashes are Merkle-aggregated into $root_{row}$.

- **Column commitment:** For every cell column, the digests in that column are committed with an inner Merkle tree, producing a column root. The column roots are then Merkle-aggregated into $root_{col}$.

- **Final V3 commitment:** V3 publishes one root, $root=H(root_{row},root_{col})$.

- **LeanVM proof:** The private witness is the codeword matrix. The proof checks cell digest computation, row commitment, column commitment, final root binding, and RS membership for every row.

- **RS membership:** The Fiat-Shamir challenge and the public check vector $L$ are computed outside the proof from public data. Inside LeanVM, each row proves only the inner product condition $\langle L,w_i\rangle=0$.

- **Opening:** A sampled column opening gives the codeword cells in that column plus Merkle authentication data up to the final V3 root.

- **Reconstruction:** After at least $t=\lceil k/c\rceil$ distinct verified cell columns have been collected, the implementation reconstructs from arbitrary cell erasure patterns using FFT-based RS erasure decoding.

## 4. Input Parameters

| Parameter | Meaning |
| --- | --- |
| $n$ | Number of blob/codeword rows proved in one LeanVM execution. |
| $k$ | Number of message symbols in one blob row. |
| $m$ | Number of encoded RS symbols in one codeword row. |
| $\rho=k/m$ | RS code rate. All current benchmark profiles use $\rho=1/2$. |
| $c$ | Number of field symbols per cell. |
| $\ell=m/c$ | Number of cells per row. |
| $t=\lceil k/c\rceil$ | Number of distinct cell columns needed for reconstruction. |
| $|Q|$ | Number of sampled/opened cell columns per verifier transcript. |
| $N_{clients}$ | Number of client transcripts considered in subset-soundness. |
| $\epsilon$ | Fraction of accepting clients targeted by the adversary. |
| $L_{sub}=\lceil\epsilon N_{clients}\rceil$ | Size of the accepting client subset in subset-soundness. |
| $\Delta=t-1$ | Largest non-reconstructing number of served cell columns. |
| $\lambda$ | Sampling security target in bits. Current sweeps use $\lambda=40$. |
| $\nu_{rep}$ | With-replacement subset-soundness failure bound. |
| Symbol field | Field used for blob symbols and codeword symbols. |
| Challenge field | Field used for Fiat-Shamir challenges and RS inner products. |
| WHIR log inv rate | LeanVM/WHIR inverse-rate exponent. This affects proof-system parameters, not the RS code rate. |

The with-replacement subset-soundness bound used in the current V3-ext sweeps is:

$\nu_{sub}=\binom{\ell}{\Delta}\binom{N_{clients}}{L_{sub}}\left(\frac{\Delta}{\ell}\right)^{|Q|L_{sub}}\le 2^{-\lambda}$.

Current benchmark defaults are $N_{clients}=10000$, $\epsilon=0.01$, and $\lambda=40$.

## 5. Benchmark Metrics

| Metric | Meaning |
| --- | --- |
| Payload | Effective payload size, using $31$ bits per KoalaBear limb and $5\cdot31$ bits per extension symbol. |
| Read-only elements | Number of KoalaBear field elements placed in the LeanVM read-only public segment. |
| Opened cells | Number of sampled cell columns per verifier transcript. |
| Commitment size | Size of the public commitment. V3 reduces this to one digest. |
| Proof size | Serialized LeanVM proof size. |
| Sample size | Size of one sampled opening transcript. |
| Encode + commit | Host time for RS encoding and commitment construction. |
| Prover preprocess | Host time for statement preparation, Fiat-Shamir, $L$ construction, and bytecode preparation. |
| LeanVM prove | Time for LeanVM proving. Throughput is computed from this metric. |
| Verifier rebuild | Host time for verifier-side statement reconstruction. |
| LeanVM verify | Time for LeanVM proof verification. |
| Verify openings | Time to verify sampled column openings. |
| Reconstruct | Time to reconstruct blobs from enough verified cells. |
| VM cycles | LeanVM cycle count for the guest relation. |
| Poseidon16 calls | Number of Poseidon width-16 calls in the guest. |
| ExtensionOp calls | Number of extension-field operations in the guest. |
| Throughput | Effective payload divided by LeanVM prove time. |

## 6. V3 Benchmark IDs

- **Format:** <span style="white-space:nowrap">`vX-field-bY-cZ-rN-wR`</span>.
- **Version:** `v3` is the current construction version.
- **Field:** `base` means KoalaBear payload symbols; `ext` means quintic-extension payload symbols.
- **Blob size:** `b1`, `b2`, and `b4` denote 1x, 2x, and 4x row payload profiles.
- **Cell size:** `c16`, `c32`, `c64`, and `c128` record the cell size.
- **Rows:** `r14` means $n=14$ rows.
- **WHIR:** `w1` means WHIR log inverse rate $1$.

## 7. V3 Input Parameter Summary

| Benchmark IDs | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Public commitment | Membership |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| <span style="white-space:nowrap">`v3-base-b1-c64-r1-w1`</span>, <span style="white-space:nowrap">`v3-base-b1-c64-r14-w1`</span>, <span style="white-space:nowrap">`v3-base-b1-c64-r16-w1`</span> | 1, 14, 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | final root | `dot_product_be` |
| <span style="white-space:nowrap">`v3-base-b2-c128-r1-w1`</span>, <span style="white-space:nowrap">`v3-base-b2-c128-r14-w1`</span>, <span style="white-space:nowrap">`v3-base-b2-c128-r16-w1`</span> | 1, 14, 16 | KoalaBear | Quintic extension | 65536 | 131072 | 128 | 1024 | 512 | 19 | final root | `dot_product_be` |
| <span style="white-space:nowrap">`v3-ext-b1-c16-r14-w1`</span> | 14 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | final root | `dot_product_ee` |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 14 | Quintic extension | Quintic extension | 16384 | 32768 | 32 | 1024 | 512 | 19 | final root | `dot_product_ee` |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 14 | Quintic extension | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | final root | `dot_product_ee` |

## 8. V3-Base Results

| Benchmark ID | Payload | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-base-b1-c64-r1-w1`</span> | 124 KiB | 327688 | 19 | 0.03 KB | 279.06 KB | 11.36 KB | 0.633s | 0.110s | 0.033s | 0.001s | 0.064s | 135816 | 8703 | 65536 | 195.89 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-base-b1-c64-r14-w1`</span> | 1736 KiB | 327688 | 19 | 0.03 KB | 336.35 KB | 73.11 KB | 3.827s | 0.113s | 0.037s | 0.004s | 0.226s | 1263336 | 123905 | 917504 | 453.39 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-base-b1-c64-r16-w1`</span> | 1984 KiB | 327688 | 19 | 0.03 KB | 328.80 KB | 82.61 KB | 6.195s | 0.121s | 0.044s | 0.004s | 0.271s | 1403691 | 139263 | 1048576 | 320.26 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-base-b2-c128-r1-w1`</span> | 248 KiB | 655368 | 19 | 0.03 KB | 276.18 KB | 16.11 KB | 0.985s | 0.205s | 0.037s | 0.001s | 0.150s | 160392 | 16895 | 131072 | 251.78 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-base-b2-c128-r14-w1`</span> | 3472 KiB | 655368 | 19 | 0.03 KB | 331.42 KB | 139.61 KB | 7.274s | 0.207s | 0.045s | 0.008s | 0.524s | 1607400 | 238593 | 1835008 | 477.32 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-base-b2-c128-r16-w1`</span> | 3968 KiB | 655368 | 19 | 0.03 KB | 378.32 KB | 158.61 KB | 9.342s | 0.222s | 0.047s | 0.010s | 0.602s | 1796907 | 270335 | 2097152 | 424.75 KiB/s | accepted |

## 9. V3-Ext Sweep A: Blob Size

| Benchmark ID | Blob size | $k$ | $m$ | $c$ | $\ell$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b1-c16-r14-w1`</span> | 1x | 8192 | 16384 | 16 | 1024 | 2170 KiB | 327.36 KB | 89.73 KB | 0.243s | 0.063s | 3.757s | 0.062s | 0.041s | 0.005s | 0.147s | 1349380 | 152577 | 229376 | 577.59 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 2x | 16384 | 32768 | 32 | 1024 | 4340 KiB | 367.91 KB | 172.86 KB | 0.495s | 0.076s | 4.808s | 0.081s | 0.040s | 0.009s | 0.313s | 1779460 | 295937 | 458752 | 902.66 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 4x | 32768 | 65536 | 64 | 1024 | 8680 KiB | 388.76 KB | 339.11 KB | 0.932s | 0.125s | 9.566s | 0.121s | 0.045s | 0.020s | 0.636s | 2639620 | 582657 | 917504 | 907.38 KiB/s | accepted |

## 10. V3-Ext Sweep B: Cell Size at 2x Blob Size

| Benchmark ID | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{rep}$ | Payload | Proof size | Sample size | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b2-c16-r14-w1`</span> | 16 | 2048 | 1024 | 29 | -58.625 | 4340 KiB | 343.39 KB | 137.86 KB | 9.346s | 2697988 | 305153 | 458752 | 464.37 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 32 | 1024 | 512 | 19 | -83.398 | 4340 KiB | 367.91 KB | 172.86 KB | 5.128s | 1779460 | 295937 | 458752 | 846.33 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c64-r14-w1`</span> | 64 | 512 | 256 | 14 | -97.448 | 4340 KiB | 369.10 KB | 249.43 KB | 5.350s | 1320196 | 291329 | 458752 | 811.21 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c128-r14-w1`</span> | 128 | 256 | 128 | 11 | -57.495 | 4340 KiB | 368.95 KB | 388.14 KB | 5.441s | 1090564 | 289025 | 458752 | 797.65 KiB/s | accepted |

## 11. V3-Ext Sweep C: Row Count at 2x Blob Size

| Benchmark ID | $n$ | Payload | Proof size | Sample size | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r1-w1`</span> | 1 | 310 KiB | 277.36 KB | 18.48 KB | 0.616s | 172682 | 20991 | 32768 | 503.25 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r2-w1`</span> | 2 | 620 KiB | 295.35 KB | 30.36 KB | 1.114s | 294073 | 41983 | 65536 | 556.55 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r4-w1`</span> | 4 | 1240 KiB | 316.00 KB | 54.11 KB | 2.107s | 536855 | 83967 | 131072 | 588.51 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r6-w1`</span> | 6 | 1860 KiB | 325.46 KB | 77.86 KB | 2.336s | 807308 | 128001 | 196608 | 796.23 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r8-w1`</span> | 8 | 2480 KiB | 312.74 KB | 101.61 KB | 3.781s | 1021395 | 167935 | 262144 | 655.91 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r10-w1`</span> | 10 | 3100 KiB | 342.79 KB | 125.36 KB | 4.743s | 1347188 | 216069 | 327680 | 653.59 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r12-w1`</span> | 12 | 3720 KiB | 343.16 KB | 149.11 KB | 5.094s | 1563324 | 256003 | 393216 | 730.27 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 14 | 4340 KiB | 367.91 KB | 172.86 KB | 5.709s | 1779460 | 295937 | 458752 | 760.20 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r16-w1`</span> | 16 | 4960 KiB | 331.52 KB | 196.61 KB | 8.047s | 1993547 | 335871 | 524288 | 616.38 KiB/s | accepted |

## 12. V3-Ext Sweep D: Cell Size at 4x Blob Size

| Benchmark ID | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{rep}$ | Payload | Proof size | Sample size | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b4-c16-r14-w1`</span> | 16 | 4096 | 2048 | 50 | -110.560 | 8680 KiB | 366.48 KB | 239.26 KB | 18.111s | 5395204 | 610305 | 917504 | 479.26 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r14-w1`</span> | 32 | 2048 | 1024 | 29 | -58.625 | 8680 KiB | 390.29 KB | 264.74 KB | 9.838s | 3558148 | 591873 | 917504 | 882.29 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 64 | 1024 | 512 | 19 | -83.398 | 8680 KiB | 388.76 KB | 339.11 KB | 10.026s | 2639620 | 582657 | 917504 | 865.75 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c128-r14-w1`</span> | 128 | 512 | 256 | 14 | -97.448 | 8680 KiB | 388.73 KB | 494.43 KB | 10.616s | 2180356 | 578049 | 917504 | 817.63 KiB/s | accepted |

## 13. V3-Ext Sweep E: Row Count at 4x Blob Size

| Benchmark ID | $n$ | Payload | Proof size | Sample size | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r1-w1`</span> | 1 | 620 KiB | 293.60 KB | 29.11 KB | 1.085s | 345226 | 41983 | 65536 | 571.43 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r2-w1`</span> | 2 | 1240 KiB | 324.15 KB | 47.24 KB | 2.064s | 587961 | 83967 | 131072 | 600.78 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r4-w1`</span> | 4 | 2480 KiB | 333.82 KB | 83.49 KB | 4.209s | 1073431 | 167935 | 262144 | 589.21 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r6-w1`</span> | 6 | 3720 KiB | 342.45 KB | 119.74 KB | 4.674s | 1614220 | 256001 | 393216 | 795.89 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r8-w1`</span> | 8 | 4960 KiB | 331.17 KB | 155.99 KB | 7.119s | 2042323 | 335871 | 524288 | 696.73 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r10-w1`</span> | 10 | 6200 KiB | 360.48 KB | 192.24 KB | 9.641s | 2693748 | 432133 | 655360 | 643.09 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r12-w1`</span> | 12 | 7440 KiB | 361.54 KB | 228.49 KB | 11.070s | 3125948 | 512003 | 786432 | 672.09 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r14-w1`</span> | 14 | 8680 KiB | 390.29 KB | 264.74 KB | 11.937s | 3558148 | 591873 | 917504 | 727.15 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r16-w1`</span> | 16 | 9920 KiB | 353.09 KB | 300.99 KB | 21.605s | 3986251 | 671743 | 1048576 | 459.15 KiB/s | accepted |

## 14. V3-Ext Sweep F: WHIR Rate

| Benchmark ID | WHIR log inv rate | Payload | Proof size | Sample size | LeanVM prove | VM cycles | Poseidon16 calls | ExtensionOp calls | Throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 1 | 4340 KiB | 367.91 KB | 172.86 KB | 5.498s | 1779460 | 295937 | 458752 | 789.38 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w2`</span> | 2 | 4340 KiB | 243.73 KB | 172.86 KB | 7.138s | 1779460 | 295937 | 458752 | 607.73 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 1 | 8680 KiB | 388.76 KB | 339.11 KB | 11.287s | 2639620 | 582657 | 917504 | 768.14 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w2`</span> | 2 | 8680 KiB | 253.98 KB | 339.11 KB | 15.620s | 2639620 | 582657 | 917504 | 555.70 KiB/s | accepted |

## 15. Summary

The main outcome is that Encode + Prove is a viable post-quantum DAS direction when repairability is required. V3-ext is the most promising implementation path because extension-field payloads reduce the number of RS symbols and allow `dot_product_ee` to check membership with fewer rows than the base-field construction.

The strongest V3-ext runs reach roughly $0.9$ MiB/s effective-payload proving throughput inside LeanVM. The best clean blob-size sweep result is <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> at about $902.66$ KiB/s, while the 4x blob sweep reaches about $882.29$ KiB/s at <span style="white-space:nowrap">`v3-ext-b4-c32-r14-w1`</span>. Increasing blob size beyond 2x does not automatically improve throughput, because larger traces begin to expose proof-system and memory effects.

The most important parameter lessons are:

- $c=16$ creates too many cells and is consistently slow.
- $c=32$ and $c=64$ are the strongest cell-size candidates in the current implementation.
- $n=16$ often has a proving-time cliff, while $n=6$ and $n=14$ are usually better batch points.
- WHIR log inverse rate $2$ reduces proof size but slows proving; rate $1$ is better for throughput.

The remaining route to $1$ MiB/s is therefore not simply larger blobs. The next work should focus on LeanVM-level profiling, proof backend tuning, and tighter row-count/cell-size search around the observed sweet spots.
