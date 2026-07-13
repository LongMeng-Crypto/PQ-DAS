# PQ-DAS Research Report

<style>
table th, table td { white-space: nowrap; }
table td:nth-child(1) { white-space: nowrap; }
</style>

## 1. Motivation

Data availability sampling is the mechanism that allows a distributed system to accept large data objects without requiring every validator to download them in full. Instead of checking the whole payload, validators check a small number of authenticated samples from an erasure-coded representation. If the sampling protocol is sound, then a producer who withholds too much data is detected with high probability, while honest data can still be accepted with small bandwidth and verification cost.

This problem is especially important for blockchain data availability layers. Modern rollup and sharding designs rely on the assumption that transaction or blob data remains available after a block is accepted. If the data later disappears, users may be unable to reconstruct state transitions, generate fraud proofs, or independently validate the system history. DAS is therefore not merely a compression technique for validators; it is a way to turn local randomized checks into a global availability guarantee.

The post-quantum setting changes the design constraints. Highly efficient DAS proposals use polynomial commitments such as KZG, whose security relies on algebraic assumptions that are not post-quantum. A post-quantum DAS construction should instead rely on Post-quantum primitives, while still preserving the operational properties that make DAS useful in practice: small samples, efficient verification, and the ability to recover data once enough openings have been collected.

The purpose of this project is to evaluate whether a repairable post-quantum DAS construction can be made practical. The main engineering question is not only whether the construction is asymptotically possible, but whether concrete parameters, proof sizes, sampling sizes, and prover throughput are compatible with realistic blob workloads. The implementation branch for the experiments in this report is [LongMeng-Crypto/PQ-DAS `V2/V3-Demo`](https://github.com/LongMeng-Crypto/PQ-DAS/tree/V2%2FV3-Demo).

## 2. Introduction to DAS

A DAS protocol consists of a data builder or prover, a set of sampling verifiers, and a reconstruction procedure. The data producer commits to an encoded representation of the data and provides proof material; verifiers sample a small subset of positions; and any party that obtains enough accepted openings can reconstruct the original data.

- **Parties:** The builder or prover receives the payload data and produces the commitment, auxiliary opening state, and proof. Validators or light clients sample the commitment and verify openings. A retriever or reconstruction party collects enough accepted transcripts and runs decoding to recover the data.

- **Setup algorithm $\mathsf{Setup}(1^\lambda)\rightarrow {\sf pp}$:** On input a security parameter, setup outputs public parameters. These include the erasure code, evaluation domain, cell size, sampling rules, hash function, proof-system parameters, and reconstruction threshold.

- **Commitment algorithm $\mathsf{Com}({\sf pp},{\sf data})\rightarrow({\sf com},{\sf aux})$:** On input public parameters and data, the builder encodes the data, computes a commitment, and produces auxiliary state for answering future openings. The output commitment ${\sf com}$ is public, while ${\sf aux}$ contains the encoded data, Merkle paths, or other opening state.

- **Query algorithm $\mathsf{Query}({\sf pp},{\sf com};r)\rightarrow Q$:** On input the public parameters, the commitment, and verifier randomness, the verifier samples a query set $Q$. In a cell-based DAS protocol, $Q$ is a set of cell-column indices.

- **Opening algorithm $\mathsf{Open}({\sf pp},{\sf aux},Q)\rightarrow {\sf tran}$:** On input the auxiliary opening state and query set, the builder returns a transcript containing the requested symbols or cells and their authentication data. The transcript should be small compared with the full encoded data.

- **Verification algorithm $\mathsf{Verify}({\sf pp},{\sf com},Q,{\sf tran})\rightarrow\{0 &/ 1\}$:** On input the commitment, query set, and transcript, the verifier checks the proof and the sampled openings. The output is $1$ if the transcript is accepted and $0$ otherwise.

- **Reconstruction algorithm $\mathsf{Ext}({\sf pp},{\sf com},{\sf tran}_1,\ldots,{\sf tran}_z)\rightarrow {\sf data}/\bot$:** On input multiple accepted transcripts, the reconstruction algorithm verifies the openings, extracts their encoded symbols, and attempts to decode the original data. It outputs the recovered data or $\bot$ if the transcripts do not contain enough valid information.

The security requirements can be stated at a high level as follows.

- **Correctness:** If the builder is honest and the data is encoded correctly, then honestly generated openings verify and reconstruction recovers the original data once enough openings are available.

- **Soundness:** If a verifier accepts with sufficiently high probability, then the committed object should contain enough information for the data to be recovered. Informally, an adversary should not be able to make many verifiers accept while withholding too many encoded positions.

- **Extractability:** An accepting commitment should correspond to extractable data. This rules out commitments that pass sampling checks but do not determine a recoverable payload.

- **Subset soundness:** Even if an adversary only needs to fool a selected subset of clients, the probability that all of those clients accept while the served samples remain below the reconstruction threshold should be small.

- **Binding or consistency:** The commitment must bind all accepted openings to a single encoded object. Two different transcripts accepted under the same commitment should not correspond to incompatible data.

- **Repairability:** Accepted openings should be useful as reconstruction material. This property is operational rather than only game-based: the objects sampled by verifiers should be the same objects consumed by the decoder.

## 3. Solution Space

- **Commitments for Arbitrary Codes:** The builder encodes the data using a chosen erasure code, commits to the encoded word, and proves or otherwise enforces that the committed word belongs to the code. Sampling opens authenticated positions of this encoded word. The construction is code-agnostic and can be instantiated with Reed-Solomon codes or other erasure codes.

- **Commitments for Tensor Codes:** The data is arranged in a multidimensional tensor-code structure. Commitments and checks are organized along the tensor dimensions, so local consistency checks across rows, columns, or higher-dimensional slices imply global structure. This approach uses the algebraic structure of product codes to organize sampling and commitment verification.

- **Commitments for Interleaved Codes:** Several codewords are batched by interleaving their symbols, so that one sampled position can open aligned symbols from many codewords. This amortizes authentication and sampling over multiple blobs or rows. The high-level workflow is to encode many objects, interleave their coordinates, commit to the interleaved representation, and sample aligned positions.

- **FRIDA:** FRIDA follows a transparent, FRI-style approach to data availability. The builder commits to encoded data using hash-based structures and provides proximity-proof material showing that the committed object is close to a valid low-degree or codeword representation. Verifiers combine random samples with transparent proof checks rather than relying on pairing-based polynomial commitments.

- **ZODA:** ZODA is a post-quantum DAS construction based on hash-based commitments and transparent proof techniques. At a high level, it encodes data, commits to the encoded representation, and provides proof material that sampled positions are consistent with an available codeword. Its workflow is designed to avoid trusted setup and pairing assumptions.

## 4. Repairability and Encode + Prove

Repairability is the property that accepted DAS openings can later be used to reconstruct the original payload. This matters because a DA system is not only trying to convince validators to vote on a block; it is also trying to ensure that the data needed by users, provers, and archival participants can be recovered after acceptance. A protocol that detects withholding but does not produce useful repair material leaves a practical gap between availability voting and actual data recovery.

The Commitments-for-Arbitrary-Codes approach gives the cleanest repairability interface. The proof binds the committed object to a valid erasure-codeword, and the opening protocol reveals authenticated pieces of that same codeword. Once enough distinct accepted cells are collected, reconstruction can run the ordinary erasure decoder on those cells.

This is why the project focuses on an Encode + Prove methodology. The builder first encodes the data, then proves that the committed encoded object satisfies the required code membership and commitment relations. If the commitment layer is hash-based and the proof system is transparent and hash/symmetric-based, the resulting DAS construction is post-quantum while retaining the practical repair path from samples to decoding.

## 5. Concrete Construction

- **Payload representation:** The implementation treats each blob row as extension-field symbols. This reduces the number of codeword positions compared with a base-field-only payload while remaining compatible with LeanVM extension-field arithmetic.

- **RS encoding:** Each row is Reed-Solomon encoded from $k$ payload symbols to $m=2k$ codeword symbols. The encoded matrix has $n$ rows, one per blob, and $m$ symbols per row.

- **Cell decomposition:** Every encoded row is partitioned into $\ell=m/c$ cells, each containing $c$ consecutive extension-field symbols. This produces an $n\times\ell$ cell matrix.

- **Cell digest layer:** The proof hashes every cell into a digest. This digest matrix is the shared commitment interface used by both the systematic row commitment and the column-sampling commitment.

- **Systematic row commitment:** For each row, the systematic cell digests are hashed into a row digest. The row digests are Merkle-aggregated into a row root, which binds the payload-facing systematic view.

- **Column commitment:** For each cell column, an inner Merkle tree commits to the digests in that column and outputs one column root. The column roots are then Merkle-aggregated into a column root used for sampling.

- **Final commitment:** The public commitment is a hash of the row root and the column root. This binds the systematic view and sampling view to the same cell digest matrix.

- **Proof relation:** The LeanVM proof uses the encoded matrix as witness and proves cell digest computation, systematic row commitment, row-root aggregation, inner column Merkle trees, outer column Merkle aggregation, final-root binding, and Reed-Solomon membership for every row.

- **Fiat-Shamir and membership:** The public check vector $L$ is derived outside the proof from public parameters and the public root. The verifier recomputes $L$ independently, while the proof checks only the inner products $\langle w_i,L\rangle=0$ for all rows.

- **Opening and reconstruction:** A sampled opening reveals codeword cells and authentication data for the selected columns. After enough accepted columns are collected, reconstruction uses FFT-based arbitrary-erasure Reed-Solomon decoding to recover the original payload.

## 6. Input Parameters

| Parameter | Meaning |
| --- | --- |
| Base field $\mathbb{F}$ | KoalaBear base field used by LeanVM memory, Poseidon inputs, digest coordinates, and proof-system arithmetic. |
| Extension field $\mathbb{E}$ | Quintic extension field used for payload symbols, RS evaluations, Fiat-Shamir challenge points, and RS membership inner products. |
| $n$ | Number of blob rows in one proved matrix. |
| $k$ | Number of payload symbols per row before rate-$1/2$ encoding. |
| $m$ | Number of encoded symbols per row. In the measured profile, $m=2k$. |
| $\rho$ | RS code rate $k/m$. |
| $c$ | Number of extension-field symbols per cell. |
| $\ell=m/c$ | Number of cells per row and number of cell columns. |
| $t=k/c$ | Number of systematic cells per row. |
| $\lvert Q\rvert$ | Number of sampled cell columns opened by a verifier. |
| $\log_2\nu_{\mathrm{rep}}$ | Logarithmic sampling failure bound for sampling with replacement. |
| WHIR log inverse rate | LeanVM/WHIR proof-system rate parameter used by the execution proof. |
| Upload/download bandwidth | Network bandwidth used in the full DAS throughput model; this report uses $50$ Mbps in each direction. |

## 7. Benchmark Metrics

| Metric | Meaning |
| --- | --- |
| Commitment size | Public commitment size. |
| Proof size | Serialized LeanVM proof size reported by the benchmark harness. |
| Sample size | Total sampled opening size for the chosen $\lvert Q\rvert$ columns. |
| Encode + commit | Host time to encode rows, compute cell digests, and build commitments. |
| Prover preprocess | Time to rebuild verifier-visible public statement data, including Fiat-Shamir and $L$. |
| LeanVM prove | Time to generate the LeanVM proof. |
| Opening generation | Time to produce sampled openings. |
| Verifier rebuild | Time for the verifier to reconstruct the public statement and $L$. |
| LeanVM verify | Time to verify the LeanVM proof. |
| Verify openings | Time to verify sampled column openings. |
| Reconstruct | Time to reconstruct the payload from accepted cells. |
| VM cycles | LeanVM guest cycles. |
| Poseidon16 calls | Number of Poseidon width-16 calls used by the proof relation. |
| ExtensionOp calls | Number of extension-field operation calls used by RS membership. |
| LeanVM proving throughput | Effective payload divided by LeanVM proving time. |
| Full DAS throughput | Effective payload divided by the full builder-to-validator workflow time. |

The full DAS throughput is computed as

$$
\frac{D_{\mathrm{payload}}}{T_{\mathrm{encode+commit}}+T_{\mathrm{preprocess}}+T_{\mathrm{prove}}+T_{\mathrm{open}}+T_{\mathrm{verifier\ rebuild}}+T_{\mathrm{verify\ proof}}+T_{\mathrm{verify\ openings}}+T_{\mathrm{upload}}+T_{\mathrm{download}}}.
$$

Here $D_{\mathrm{payload}}$ is the useful blob payload size, i.e. number of blobs times blob size with the extension-field bit-size correction. The upload and download times are computed from the uploaded/downloaded byte sizes and the assumed $50$ Mbps bandwidth. Reconstruction is reported separately because it is not on the critical path for validator acceptance.

## 8. Benchmark Profile Names

- **Format:** `ext-bY-cZ-rN-wR`.
- **Field:** `ext` means quintic-extension payload symbols and quintic-extension RS membership checks.
- **Blob size:** `b1`, `b2`, and `b4` denote the 1x, 2x, and 4x row payload profiles.
- **Cell size:** `c16`, `c32`, `c64`, and `c128` record the number of extension-field symbols per cell.
- **Rows and WHIR:** `r14` means $n=14$ rows, and `w1` means WHIR log inverse rate $1$.

## 9. Extension-Field Parameter Summary

| Profile family | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Public commitment | Membership |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| `ext-b1-c16-r14-w1` | 14 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | final root | `dot_product_ee` |
| `ext-b2-c32-r14-w1` | 14 | Quintic extension | Quintic extension | 16384 | 32768 | 32 | 1024 | 512 | 19 | final root | `dot_product_ee` |
| `ext-b4-c64-r14-w1` | 14 | Quintic extension | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | final root | `dot_product_ee` |
| `ext-b2-c16-r14-w1` | 14 | Quintic extension | Quintic extension | 16384 | 32768 | 16 | 2048 | 1024 | 29 | final root | `dot_product_ee` |
| `ext-b2-c64-r14-w1` | 14 | Quintic extension | Quintic extension | 16384 | 32768 | 64 | 512 | 256 | 14 | final root | `dot_product_ee` |
| `ext-b2-c128-r14-w1` | 14 | Quintic extension | Quintic extension | 16384 | 32768 | 128 | 256 | 128 | 11 | final root | `dot_product_ee` |
| `ext-b4-c16-r14-w1` | 14 | Quintic extension | Quintic extension | 32768 | 65536 | 16 | 4096 | 2048 | 50 | final root | `dot_product_ee` |
| `ext-b4-c32-r14-w1` | 14 | Quintic extension | Quintic extension | 32768 | 65536 | 32 | 2048 | 1024 | 29 | final root | `dot_product_ee` |
| `ext-b4-c128-r14-w1` | 14 | Quintic extension | Quintic extension | 32768 | 65536 | 128 | 512 | 256 | 14 | final root | `dot_product_ee` |

## 10. Sweep A: Blob Size

- Fixed parameters: $\ell=1024$, $n=14$, $t=512$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: blob size, with $c$ scaled so that $\ell=m/c$ stays fixed.

| Profile | Blob size | $k$ | $m$ | $c$ | $\ell$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b1-c16-r14-w1` | 1x | 8192 | 16384 | 16 | 1024 | 2170 KiB | 327.36 KB | 89.73 KB | 0.243s | 0.063s | 3.757s | 0.062s | 0.041s | 0.005s | 0.147s | 1349380 | 152577 | 229376 | 577.59 KiB/s | 431.67 KiB/s | accepted |
| `ext-b2-c32-r14-w1` | 2x | 16384 | 32768 | 32 | 1024 | 4340 KiB | 367.91 KB | 172.86 KB | 0.495s | 0.076s | 4.808s | 0.081s | 0.040s | 0.009s | 0.313s | 1779460 | 295937 | 458752 | 902.66 KiB/s | 609.05 KiB/s | accepted |
| `ext-b4-c64-r14-w1` | 4x | 32768 | 65536 | 64 | 1024 | 8680 KiB | 388.76 KB | 339.11 KB | 0.932s | 0.125s | 9.566s | 0.121s | 0.045s | 0.020s | 0.636s | 2639620 | 582657 | 917504 | 907.38 KiB/s | 623.21 KiB/s | accepted |

## 11. Sweep B: Cell Size at 2x Blob Size

- Fixed parameters: blob size $=2x$, $n=14$, $k=16384$, $m=32768$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Profile | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b2-c16-r14-w1` | 16 | 2048 | 1024 | 29 | -58.625 | 4340 KiB | 343.39 KB | 137.86 KB | 0.514s | 0.131s | 9.346s | 0.139s | 0.046s | 0.008s | 0.356s | 2697988 | 305153 | 458752 | 464.37 KiB/s | 368.20 KiB/s | accepted |
| `ext-b2-c32-r14-w1` | 32 | 1024 | 512 | 19 | -83.398 | 4340 KiB | 367.91 KB | 172.86 KB | 0.521s | 0.088s | 5.128s | 0.092s | 0.045s | 0.010s | 0.344s | 1779460 | 295937 | 458752 | 846.33 KiB/s | 578.60 KiB/s | accepted |
| `ext-b2-c64-r14-w1` | 64 | 512 | 256 | 14 | -97.448 | 4340 KiB | 369.10 KB | 249.43 KB | 0.516s | 0.066s | 5.350s | 0.071s | 0.046s | 0.017s | 0.351s | 1320196 | 291329 | 458752 | 811.21 KiB/s | 563.94 KiB/s | accepted |
| `ext-b2-c128-r14-w1` | 128 | 256 | 128 | 11 | -57.495 | 4340 KiB | 368.95 KB | 388.14 KB | 0.531s | 0.063s | 5.441s | 0.072s | 0.047s | 0.024s | 0.356s | 1090564 | 289025 | 458752 | 797.65 KiB/s | 554.24 KiB/s | accepted |

## 12. Sweep C: Row Count at 2x Blob Size

- Fixed parameters: blob size $=2x$, $c=32$, $k=16384$, $m=32768$, $\ell=1024$, $t=512$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: row count $n$.

| Profile | $n$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b2-c32-r1-w1` | 1 | 310 KiB | 277.36 KB | 18.48 KB | 0.038s | 0.088s | 0.667s | 0.075s | 0.029s | 0.001s | 0.045s | 172682 | 20991 | 32768 | 464.77 KiB/s | 282.65 KiB/s | accepted |
| `ext-b2-c32-r2-w1` | 2 | 620 KiB | 295.76 KB | 30.36 KB | 0.064s | 0.073s | 1.168s | 0.075s | 0.033s | 0.001s | 0.064s | 294073 | 41983 | 65536 | 530.82 KiB/s | 359.29 KiB/s | accepted |
| `ext-b2-c32-r4-w1` | 4 | 1240 KiB | 316.00 KB | 54.11 KB | 0.125s | 0.072s | 1.878s | 0.077s | 0.036s | 0.003s | 0.104s | 536855 | 83967 | 131072 | 660.28 KiB/s | 455.40 KiB/s | accepted |
| `ext-b2-c32-r6-w1` | 6 | 1860 KiB | 325.06 KB | 77.86 KB | 0.199s | 0.083s | 2.286s | 0.078s | 0.037s | 0.004s | 0.144s | 807308 | 128001 | 196608 | 813.65 KiB/s | 541.42 KiB/s | accepted |
| `ext-b2-c32-r8-w1` | 8 | 2480 KiB | 312.74 KB | 101.61 KB | 0.268s | 0.080s | 3.374s | 0.079s | 0.038s | 0.005s | 0.182s | 1021395 | 167935 | 262144 | 735.03 KiB/s | 516.45 KiB/s | accepted |
| `ext-b2-c32-r10-w1` | 10 | 3100 KiB | 342.79 KB | 125.36 KB | 0.335s | 0.079s | 4.014s | 0.078s | 0.036s | 0.006s | 0.215s | 1347188 | 216069 | 327680 | 772.30 KiB/s | 541.06 KiB/s | accepted |
| `ext-b2-c32-r12-w1` | 12 | 3720 KiB | 343.16 KB | 149.11 KB | 0.378s | 0.074s | 4.261s | 0.084s | 0.039s | 0.008s | 0.270s | 1563324 | 256003 | 393216 | 873.03 KiB/s | 596.23 KiB/s | accepted |
| `ext-b2-c32-r14-w1` | 14 | 4340 KiB | 367.91 KB | 172.86 KB | 0.483s | 0.081s | 4.892s | 0.085s | 0.039s | 0.009s | 0.311s | 1779460 | 295937 | 458752 | 887.16 KiB/s | 602.28 KiB/s | accepted |
| `ext-b2-c32-r16-w1` | 16 | 4960 KiB | 331.52 KB | 196.61 KB | 0.547s | 0.083s | 7.310s | 0.090s | 0.050s | 0.010s | 0.360s | 1993547 | 335871 | 524288 | 678.52 KiB/s | 500.58 KiB/s | accepted |
| `ext-b2-c32-r18-w1` | 18 | 5580 KiB | 352.98 KB | 220.36 KB | 0.640s | 0.084s | 8.578s | 0.091s | 0.044s | 0.013s | 0.419s | 2424900 | 392205 | 589824 | 650.50 KiB/s | 485.67 KiB/s | accepted |
| `ext-b2-c32-r20-w1` | 20 | 6200 KiB | 361.98 KB | 244.11 KB | 0.820s | 0.090s | 9.607s | 0.101s | 0.044s | 0.014s | 0.462s | 2641036 | 432139 | 655360 | 645.36 KiB/s | 479.44 KiB/s | accepted |
| `ext-b2-c32-r22-w1` | 22 | 6820 KiB | 362.04 KB | 267.86 KB | 0.822s | 0.090s | 10.542s | 0.096s | 0.045s | 0.015s | 0.514s | 2857172 | 472073 | 720896 | 646.94 KiB/s | 484.40 KiB/s | accepted |
| `ext-b2-c32-r24-w1` | 24 | 7440 KiB | 361.67 KB | 291.61 KB | 0.894s | 0.090s | 10.705s | 0.099s | 0.045s | 0.015s | 0.545s | 3073308 | 512007 | 786432 | 695.00 KiB/s | 512.01 KiB/s | accepted |
| `ext-b2-c32-r26-w1` | 26 | 8060 KiB | 388.51 KB | 315.36 KB | 0.955s | 0.093s | 11.664s | 0.101s | 0.045s | 0.018s | 0.610s | 3289444 | 551941 | 851968 | 691.02 KiB/s | 510.73 KiB/s | accepted |
| `ext-b2-c32-r28-w1` | 28 | 8680 KiB | 390.23 KB | 339.11 KB | 1.065s | 0.090s | 12.433s | 0.101s | 0.045s | 0.019s | 0.655s | 3505580 | 591875 | 917504 | 698.14 KiB/s | 514.45 KiB/s | accepted |
| `ext-b2-c32-r30-w1` | 30 | 9300 KiB | 388.67 KB | 362.86 KB | 1.137s | 0.093s | 12.130s | 0.102s | 0.046s | 0.020s | 0.710s | 3721716 | 631809 | 983040 | 766.69 KiB/s | 551.58 KiB/s | accepted |
| `ext-b2-c32-r32-w1` | 32 | 9920 KiB | 350.37 KB | 386.61 KB | 1.211s | 0.095s | 21.419s | 0.322s | 0.058s | 0.023s | 1.080s | 3935803 | 671743 | 1048576 | 463.14 KiB/s | 372.07 KiB/s | accepted |

## 13. Sweep D: Cell Size at 4x Blob Size

- Fixed parameters: blob size $=4x$, $n=14$, $k=32768$, $m=65536$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Profile | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b4-c16-r14-w1` | 16 | 4096 | 2048 | 50 | -110.560 | 8680 KiB | 366.48 KB | 239.26 KB | 1.050s | 0.276s | 18.111s | 0.371s | 0.045s | 0.012s | 0.841s | 5395204 | 610305 | 917504 | 479.27 KiB/s | 378.04 KiB/s | accepted |
| `ext-b4-c32-r14-w1` | 32 | 2048 | 1024 | 29 | -58.625 | 8680 KiB | 390.29 KB | 264.74 KB | 0.925s | 0.154s | 9.838s | 0.160s | 0.039s | 0.013s | 0.622s | 3558148 | 591873 | 917504 | 882.29 KiB/s | 609.71 KiB/s | accepted |
| `ext-b4-c64-r14-w1` | 64 | 1024 | 512 | 19 | -83.398 | 8680 KiB | 388.76 KB | 339.11 KB | 0.960s | 0.117s | 10.026s | 0.131s | 0.045s | 0.019s | 0.661s | 2639620 | 582657 | 917504 | 865.75 KiB/s | 602.07 KiB/s | accepted |
| `ext-b4-c128-r14-w1` | 128 | 512 | 256 | 14 | -97.448 | 8680 KiB | 388.73 KB | 494.43 KB | 0.996s | 0.100s | 10.616s | 0.109s | 0.044s | 0.027s | 0.674s | 2180356 | 578049 | 917504 | 817.63 KiB/s | 577.27 KiB/s | accepted |

## 14. Sweep E: Row Count at 4x Blob Size

- Fixed parameters: blob size $=4x$, $c=32$, $k=32768$, $m=65536$, $\ell=2048$, $t=1024$, opened cells $=29$, WHIR log inverse rate $=1$.
- Variable parameter: row count $n$.

| Profile | $n$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b4-c32-r1-w1` | 1 | 620 KiB | 293.60 KB | 29.11 KB | 0.079s | 0.159s | 1.085s | 0.164s | 0.034s | 0.002s | 0.108s | 345226 | 41983 | 65536 | 571.43 KiB/s | 338.11 KiB/s | accepted |
| `ext-b4-c32-r2-w1` | 2 | 1240 KiB | 324.15 KB | 47.24 KB | 0.144s | 0.157s | 2.064s | 0.164s | 0.039s | 0.003s | 0.148s | 587961 | 83967 | 131072 | 600.78 KiB/s | 399.43 KiB/s | accepted |
| `ext-b4-c32-r4-w1` | 4 | 2480 KiB | 333.82 KB | 83.49 KB | 0.272s | 0.157s | 4.209s | 0.176s | 0.042s | 0.005s | 0.243s | 1073431 | 167935 | 262144 | 589.21 KiB/s | 425.90 KiB/s | accepted |
| `ext-b4-c32-r6-w1` | 6 | 3720 KiB | 342.45 KB | 119.74 KB | 0.444s | 0.168s | 4.674s | 0.179s | 0.044s | 0.007s | 0.340s | 1614220 | 256001 | 393216 | 795.89 KiB/s | 538.65 KiB/s | accepted |
| `ext-b4-c32-r8-w1` | 8 | 4960 KiB | 331.17 KB | 155.99 KB | 0.596s | 0.171s | 7.119s | 0.180s | 0.047s | 0.009s | 0.431s | 2042323 | 335871 | 524288 | 696.73 KiB/s | 499.31 KiB/s | accepted |
| `ext-b4-c32-r10-w1` | 10 | 6200 KiB | 360.48 KB | 192.24 KB | 0.782s | 0.173s | 9.641s | 0.212s | 0.048s | 0.012s | 0.564s | 2693748 | 432133 | 655360 | 643.09 KiB/s | 472.75 KiB/s | accepted |
| `ext-b4-c32-r12-w1` | 12 | 7440 KiB | 361.54 KB | 228.49 KB | 0.948s | 0.177s | 11.070s | 0.188s | 0.046s | 0.013s | 0.629s | 3125948 | 512003 | 786432 | 672.09 KiB/s | 492.24 KiB/s | accepted |
| `ext-b4-c32-r14-w1` | 14 | 8680 KiB | 390.29 KB | 264.74 KB | 1.038s | 0.168s | 11.937s | 0.192s | 0.045s | 0.015s | 0.703s | 3558148 | 591873 | 917504 | 727.15 KiB/s | 525.99 KiB/s | accepted |
| `ext-b4-c32-r16-w1` | 16 | 9920 KiB | 353.09 KB | 300.99 KB | 1.199s | 0.180s | 21.605s | 0.404s | 0.051s | 0.017s | 1.062s | 3986251 | 671743 | 1048576 | 459.15 KiB/s | 367.73 KiB/s | accepted |

## 15. Sweep F: WHIR Rate

- Fixed candidates: `ext-b2-c32-r14-w1` and `ext-b4-c64-r14-w1`.
- Variable parameter: WHIR log inverse rate $r\in\{1,2\}$ under the default LeanVM folding factors.

| Profile | WHIR log inv rate | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b2-c32-r14-w1` | 1 | 4340 KiB | 367.91 KB | 172.86 KB | 0.517s | 0.091s | 5.498s | 0.093s | 0.047s | 0.011s | 0.359s | 1779460 | 295937 | 458752 | 789.38 KiB/s | 551.19 KiB/s | accepted |
| `ext-b2-c32-r14-w2` | 2 | 4340 KiB | 243.73 KB | 172.86 KB | 0.544s | 0.090s | 7.138s | 0.097s | 0.029s | 0.009s | 0.350s | 1779460 | 295937 | 458752 | 608.01 KiB/s | 457.65 KiB/s | accepted |
| `ext-b4-c64-r14-w1` | 1 | 8680 KiB | 388.76 KB | 339.11 KB | 1.076s | 0.130s | 11.287s | 0.138s | 0.050s | 0.020s | 0.780s | 2639620 | 582657 | 917504 | 769.03 KiB/s | 548.67 KiB/s | accepted |
| `ext-b4-c64-r14-w2` | 2 | 8680 KiB | 253.98 KB | 339.11 KB | 1.076s | 0.136s | 15.620s | 0.142s | 0.032s | 0.021s | 0.733s | 2639620 | 582657 | 917504 | 555.70 KiB/s | 431.80 KiB/s | accepted |

- WHIR log inverse rate $r$ means the WHIR proof-system RS rate is $2^{-r}$, so $r=1$ is rate $1/2$ and $r=2$ is rate $1/4$.
- Rates $r=3,4$ correspond to WHIR rates $1/8$ and $1/16$, but the target extension-field profiles panic in WHIR config construction with `Increase folding_factor_0` under LeanVM's default `WHIR_INITIAL_FOLDING_FACTOR=7`. Supporting them would require changing the global WHIR initial folding factor and synchronizing verifier/recursion configuration, so they are not included as a one-variable benchmark sweep.

## 16. Summary

- **Main outcome:** A repairable post-quantum Encode + Prove DAS construction can be implemented with hash-based commitments and LeanVM proofs at roughly $0.9$ MiB/s across the strongest measured repairable profiles, with single-profile runs occasionally reaching about $1$ MiB/s.

- **Most important design choice:** The construction keeps sampled cells useful for reconstruction. This is the practical advantage of the arbitrary-code Encode + Prove route over approaches optimized only for sampling or proximity testing.

- **Parameter lessons:** Cell size $c=32$ is the strongest current point for the 2x profile, $c=32$ and $c=64$ are both competitive for the 4x profile, and row counts around $n=12$ to $n=14$ avoid the large proving-time cliffs seen at exact larger powers of two.

- **Main bottleneck:** The proof relation is still dominated by Poseidon calls for cell/row/column commitments and extension-field operations for RS membership. Reducing these costs inside LeanVM is the clearest path toward higher throughput.
