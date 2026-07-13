# PQ-DAS Research Report

<style>
table th, table td { white-space: nowrap; }
table td:nth-child(1) { white-space: nowrap; }
</style>

## 1. Motivation

Ethereum uses data availability sampling (DAS) to let validators check large blob data by sampling a small number of authenticated positions from an erasure-coded object, rather than downloading the whole payload. As Ethereum moves toward post-quantum security, DAS constructions that rely on algebraic polynomial commitments such as KZG need hash-based, post-quantum alternatives. This post evaluates one post-quantum DAS route: a Commitments-for-Arbitrary-Codes construction instantiated with Reed-Solomon encoding, hash commitments, and LeanVM proofs, with benchmarks for concrete blob-sized parameters. The implementation branch for the experiments is [LongMeng-Crypto/PQ-DAS `V2/V3-Demo`](https://github.com/LongMeng-Crypto/PQ-DAS/tree/V2%2FV3-Demo).

## 2. Introduction to DAS

A DAS protocol consists of a data builder or prover, a set of verifiers, and a reconstruction party. The builder receives the payload data and produces the commitment and proof. Verifiers sample the commitment and verify the openings and the proof. The reconstruction party collects enough accepted transcripts and recover the data. The syntax of a DAS protocol can be described as follows:

- **$\mathsf{Setup}(1^\lambda)\rightarrow {\sf pp}$:** The setup algorithm takes input a security parameter $\lambda$, it outputs public parameters of the DAS protocol. 

- **$\mathsf{Com}({\sf pp},{\sf data})\rightarrow({\sf com}, \mathsf{aux})$:** The encoding algorithm takes input the public parameter $\mathsf{pp}$ and the data to be encoded, it outputs a commitment $\mathsf{com}$ and some auxiliary data $\mathsf{aux}$.

- **${\sf V}^{\pi,Q}_1({\sf com})\rightarrow{\sf tran}$:** The query algorithm takes input a commitment $\mathsf{com}$, it outputs  a verifier query set $Q$. 

- **${\sf V}_2({\sf com},{\sf tran})\rightarrow b$:** The verification algorithm takes input a commitment $\mathsf{com}$ and a transcript, it outputs a bit 1 if the verification is passed, otherwise outputs 0.

- **${\sf Ext}({\sf com},{\sf tran}_1,\ldots,{\sf tran}_z)\rightarrow{\sf data}/\bot$:** The reconstruction algorithm takes input a commitment $\mathsf{com}$ and a set of transcripts ${\sf tran}_1,\ldots,{\sf tran}_z$, it outputs the reconstructed data $\mathsf{data}$ or a rejection symbol $\bot$.

Informally, a DAS scheme should satisfy the following security properties: completeness, soundness, consistency, subset soundness, repairability, and local accessibility. The formal definition of these properties are defined at [DAS foundations](https://eprint.iacr.org/2023/1079.pdf).

## 3. The benchmarked class of schemes
The benchmarked class in this report is **Commitments for Arbitrary Codes** from the solution space of the [DAS foundations](https://eprint.iacr.org/2023/1079.pdf). In this class, the builder encodes the data with a chosen erasure code, commits to the encoded word, and proves that the committed object is a valid codeword. Sampling opens authenticated positions of that encoded word, while reconstruction uses accepted samples as erasure-code evaluations. 

**Other schemes.** The same solution space also includes commitments for tensor codes and commitments for interleaved codes from the [DAS foundations](https://eprint.iacr.org/2023/1079.pdf), as well as later systems such as [FRIDA](https://eprint.iacr.org/2024/248.pdf) and [ZODA](https://angeris.github.io/papers/da-construction.pdf). Tensor-code schemes arrange data in a multidimensional code and check row/column or higher-dimensional consistency; interleaved-code schemes batch many codewords so one sampled coordinate opens aligned symbols across them; FRIDA uses transparent FRI-style proximity testing for availability; and ZODA uses a modified tensor-code encoding where sampled rows and columns serve as both data and proof material. 

**Why do we choose the Commitments-for-Arbitrary-Codes construction?** This is related to a security property called repairability. It is an operational requirement that accepted DAS openings can later be used to reconstruct the original payload. This is stronger than saying that a verifier accepted a commitment: it says that the accepted material accumulated by the network is already the material needed by a decoder. In blockchain settings this distinction matters because data may need to be recovered after block acceptance by users, provers, archival nodes, or fraud-proof systems.

Commitments for Arbitrary Codes give a direct way to obtain this property. The builder first encodes the data into an erasure-codeword and then commits to that encoded object. The validity proof binds the committed object to the code, while the opening protocol reveals authenticated pieces of the same object. Therefore, once enough distinct accepted cells are collected, reconstruction can run the corresponding erasure decoder on exactly those cells.

## 5. Commitments for Arbitrary Codes: Concrete Construction
![image](https://hackmd.io/_uploads/S13ZCjfEzg.png)

The protocol workflow is described as follows:

The public parameters fix the Poseidon hash function $\mathsf{H}$, a Reed-Solomon code ${\sf RS}[\mathbb{F},{\sf U},\rho]$, the row length $k$, the encoded row length $m$, the cell size $c$, the number of cells per row $\ell=m/c$, and the reconstruction threshold $t=\lceil k/c\rceil$. They also fix the LeanVM STARK proof-system parameters. In the benchmarked instantiation, each data object is parsed into $n$ blob rows, each row is encoded as one Reed-Solomon codeword, and the first $k$ symbols are systematic payload symbols.

The commitment first arranges the encoded data as an $n\times m$ codeword matrix. Each row is split into $\ell$ consecutive cells $W_{i,j}$ of $c$ field elements. Every cell is hashed into a digest $e_{i,j}=\mathsf{H}(W_{i,j})$, so the construction works from a matrix of cell digests rather than directly from raw codeword symbols after this point.

The row side commits only to the systematic portion of each encoded row. For every row $i$, the first $t$ cell digests, which cover the systematic payload cells, are hash-chained into a row digest $r_i=\mathsf{H}(e_{i,1},\ldots,e_{i,t})$. The row digests are then Merkle-aggregated into $\mathsf{root}_{\sf row}$. This row commitment gives a compact binding to each row's systematic data.

Then the full encoded matrix is commited in column-major. For every cell column $j$, the digests $(e_{1,j},\ldots,e_{n,j})$ are Merkle-aggregated into a column root $C_j$. The column roots $(C_1,\ldots,C_{\ell})$ are then Merkle-aggregated into $\mathsf{root}_{\sf col}$. Finally, the public commitment root is $\mathsf{root}=\mathsf{H}(\mathsf{root}_{\sf row},\mathsf{root}_{\sf col})$.

The LeanVM proof binds these commitments to valid Reed-Solomon codewords. The private witness is the codeword matrix. Inside the proof, LeanVM proves the computations of: (1) the witness cells $W_{i,j}$ are hashed to all cell digests $e_{i,j}$, (2) the systematic cell digests are hash chained to row hashes and all row hashes are Merkle-aggregated into a $\mathsf{root}_{\sf row}$, (3) each column of cells are Merkle aggregated into $(C_1,\ldots,C_{\ell})$ and further aggregated into $\mathsf{root}_{\sf col}$, and the final aggregation equals the public $\mathsf{root}$. It also checks Reed-Solomon membership from the barycentric check for every row using a public vector $L$.

The vector $L$ is computed outside the proof from public data. In the benchmarked rate-$1/2$ setting, let ${\sf U}=\{1,\omega,\ldots,\omega^{m-1}\}$ and $m=2k=2h$. For $x_r=(\omega^2)^r$, each row can be viewed as two length-$h$ evaluations $A_i(x_r)=w_{i,2r}$ and $B_i(x_r)=w_{i,2r+1}$. Fiat-Shamir samples $p$ from the public parameters and $\mathsf{root}$, sets $q=p/\omega$, and defines the barycentric coefficients $L_{2r}=\ell_r(p)$ and $L_{2r+1}=-\ell_r(q)$. The proof checks $\langle L,w_i\rangle=0$ for every row, which is equivalent to $A_i(p)=B_i(p/\omega)$ for valid rate-$1/2$ codewords. The verifier independently recomputes the same $L$ from public data before verifying the LeanVM proof.

A verifier samples a set $Q$ of cell-column indices. For each sampled column $j\in Q$, the opening transcript contains the cells $W_{1,j},\ldots,W_{n,j}$ and the authentication data needed to recompute the corresponding column root $C_j$ and then authenticate $C_j$ up to $\mathsf{root}_{\sf col}$ and the final $\mathsf{root}$. Verification first checks the LeanVM proof, then hashes the opened cells back into digests, recomputes each sampled inner column root, verifies the outer Merkle authentication path, and checks the final public root.

Reconstruction uses the same accepted openings. Given enough accepted transcripts, the reconstruction algorithm verifies them, unions their sampled cell-column indices, and requires at least $t$ distinct columns. It then treats the opened cells as Reed-Solomon evaluations for each row and runs erasure decoding to recover the original systematic payload rows. Thus the sampling transcript is also repair material, not only evidence that a verifier accepted.

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

**Remarks.** The number of sampled cell columns opened by a verifier, denoted by $|Q|$, is decided by the desired subset-soundness level. The formula for deriving it is shown in the Appendix. 

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

Here $D_{\mathrm{payload}}$ is the useful blob payload size, i.e. number of blobs times blob size with the extension-field bit-size correction. The upload and download times are computed from the uploaded/downloaded byte sizes and the assumed $50$ Mbps bandwidth (the assumption is made in terms of https://eips.ethereum.org/EIPS/eip-7870). Reconstruction is reported separately because it is not on the critical path for validator acceptance.

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

- **Main outcome:** A repairable post-quantum Commitments-for-Arbitrary-Codes DAS construction can be implemented with hash-based commitments and LeanVM proofs at roughly $0.9$ MiB/s across the strongest measured repairable profiles, with single-profile runs occasionally reaching about $1$ MiB/s.

- **Design choice:** The construction keeps sampled cells useful for reconstruction. This is the practical advantage of the Commitments-for-Arbitrary-Codes route over approaches optimized only for sampling or proximity testing.

- **Parameter choice:** Cell size $c=32$ is the strongest current point for the 2x profile, $c=32$ and $c=64$ are both competitive for the 4x profile, and row counts around $n=12$ to $n=14$ avoid the large proving-time cliffs seen at exact larger powers of two.

- **Main bottleneck:** The proof relation is still dominated by Poseidon calls for cell/row/column commitments and extension-field operations for RS membership. Reducing these costs inside LeanVM is the clearest path toward higher throughput.

# Appendix
### Subset Soundness With Replacement

The benchmark profiles use the with-replacement subset-soundness bound from the DAS security-definition style of Hall-Andersen, Simkin, and Wagner. Let $N_{\sf clients}$ be the total number of client transcripts, let $\epsilon$ be the fraction of clients targeted by the adversary, and let $L_{\sf sub}=\lceil \epsilon N_{\sf clients}\rceil$ be the selected accepting subset size. Let $\Delta=t-1$ be the largest number of served cell columns that is still below the reconstruction threshold, and let $\ell=m/c$ be the total number of cell columns.

For sampling with replacement, one verifier who opens $q=|Q|$ columns lands entirely inside a fixed non-reconstructing set of size $\Delta$ with probability $(\Delta/\ell)^q$. Union-bounding over the bad served set and over the adversarially selected accepting client subset gives

$\nu_{\sf sub}=\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}\left(\frac{\Delta}{\ell}\right)^{|Q|L_{\sf sub}}\le 2^{-\lambda}.$

Equivalently, the opened-cell count used by one verifier is the smallest integer satisfying

$|Q|_{\min}=\min\left\{q\in\mathbb{Z}_{\ge1}:\log_2\binom{\ell}{\Delta}+\log_2\binom{N_{\sf clients}}{L_{\sf sub}}+qL_{\sf sub}\log_2(\Delta/\ell)\le -\lambda\right\}.$

Since $\log_2(\Delta/\ell)<0$, this can also be written as the closed-form requirement

$$
|Q|
\ge
\left\lceil
\frac{\lambda+\log_2\binom{\ell}{\Delta}+\log_2\binom{N_{\sf clients}}{L_{\sf sub}}}
{L_{\sf sub}\log_2(\ell/\Delta)}
\right\rceil.
$$

In the benchmark tables, `Opened cells` is this $|Q|_{\min}$ value for the corresponding profile. For example, with $N_{\sf clients}=10000$, $\epsilon=0.01$, $L_{\sf sub}=100$, $\lambda=40$, $\ell=1024$, and $t=512$ so that $\Delta=511$, the formula gives $|Q|_{\min}=19$ and $\log_2\nu_{\sf sub}\approx -83.398$.