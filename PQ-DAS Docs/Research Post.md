# PQ-DAS Research Report

<style>
table th, table td { white-space: nowrap; }
table td:nth-child(1) { white-space: nowrap; }
</style>

## 1. Motivation

Ethereum already uses data availability sampling (DAS) to let validators check large blob data by sampling a small number of authenticated positions from an erasure-coded object, rather than downloading the whole payload. As Ethereum moves toward post-quantum security, DAS constructions that rely on algebraic polynomial commitments such as KZG need hash-based, post-quantum alternatives. This post evaluates one repairable post-quantum DAS route: a Commitments-for-Arbitrary-Codes construction instantiated with Reed-Solomon encoding, hash commitments, and LeanVM proofs, with benchmarks for concrete blob-sized parameters. The implementation branch for the experiments is [LongMeng-Crypto/PQ-DAS `V2/V3-Demo`](https://github.com/LongMeng-Crypto/PQ-DAS/tree/V2%2FV3-Demo).

## 2. Introduction to DAS

A DAS protocol consists of a data builder or prover, a set of verifiers, and a reconstruction party. The builder receives the payload data and produces the commitment and proof. Verifiers sample the commitment and verify the openings and the proof. The reconstruction party collects enough accepted transcripts and recovers the data. The syntax of a DAS protocol can be described as follows:

- **$\mathsf{Setup}(1^\lambda)\rightarrow {\sf pp}$:** The setup algorithm takes input a security parameter $\lambda$, it outputs public parameters of the DAS protocol. 

- **$\mathsf{Com}({\sf pp},{\sf data})\rightarrow({\sf com}, \mathsf{aux})$:** The encoding algorithm takes input the public parameter $\mathsf{pp}$ and the data to be encoded, it outputs a commitment $\mathsf{com}$ and some auxiliary data $\mathsf{aux}$.

- **${\sf V}^{\pi,Q}_1({\sf com})\rightarrow{\sf tran}$:** The query algorithm takes input a commitment $\mathsf{com}$, it outputs  a verifier query set $Q$. 

- **${\sf V}_2({\sf com},{\sf tran})\rightarrow b$:** The verification algorithm takes input a commitment $\mathsf{com}$ and a transcript, it outputs a bit 1 if the verification is passed, otherwise outputs 0.

- **${\sf Ext}({\sf com},{\sf tran}_1,\ldots,{\sf tran}_z)\rightarrow{\sf data}/\bot$:** The reconstruction algorithm takes input a commitment $\mathsf{com}$ and a set of transcripts ${\sf tran}_1,\ldots,{\sf tran}_z$, it outputs the reconstructed data $\mathsf{data}$ or a rejection symbol $\bot$.

Informally, a DAS scheme should satisfy completeness, soundness, consistency, subset soundness, repairability, and local accessibility. Completeness means that honestly generated commitments, proofs, openings, and reconstruction transcripts are accepted and recover the original data. Soundness means that an adversary cannot make verifiers accept a commitment to data that is not available for reconstruction. Consistency means that one commitment cannot lead different honest reconstruction procedures to two different payloads. Subset soundness strengthens sampling soundness by requiring that even a specified subset of accepting clients collectively samples enough information to make reconstruction possible except with small probability. Repairability means that accepted samples are not merely detection evidence, but can be used by the reconstruction algorithm to repair and recover missing data. Local accessibility means that a user can check and retrieve a small local part of the committed data with local openings and authentication information, without downloading the whole encoded object. The formal definitions are given in the DAS foundations paper: https://eprint.iacr.org/2023/1079.pdf.

## 3. The benchmarked class of schemes

The benchmarked class in this report is **Commitments for Arbitrary Codes** from the solution space of the [DAS foundations paper](https://eprint.iacr.org/2023/1079.pdf). In this class, the builder encodes the data with a chosen erasure code, commits to the encoded word, and proves that the committed object is a valid codeword. Sampling opens authenticated positions of that encoded word, while reconstruction uses accepted samples as erasure-code evaluations. This is the class we benchmark because it lets us combine Reed-Solomon rows, hash-based commitments, transparent LeanVM proofs, and cell-level reconstruction in one repairable pipeline.

**Remark on other schemes.** The same solution space also includes commitments for tensor codes and commitments for interleaved codes from the [DAS foundations paper](https://eprint.iacr.org/2023/1079.pdf), as well as later systems such as FRIDA and [ZODA](https://angeris.github.io/papers/da-construction.pdf). Tensor-code schemes arrange data in a multidimensional code and check row/column or higher-dimensional consistency; interleaved-code schemes batch many codewords so one sampled coordinate opens aligned symbols across them; FRIDA uses transparent FRI-style proximity testing for availability; and ZODA uses a modified tensor-code encoding where sampled rows and columns serve as both data and proof material. These are important alternatives, but they are not the benchmark target here: the experiments below focus on the repairable arbitrary-code commitment route, where accepted samples are directly aligned with Reed-Solomon reconstruction.

## 4. Commitments for Arbitrary Codes: Choice for Repairability 

Repairability is the operational requirement that accepted DAS openings can later be used to reconstruct the original payload. This is stronger than saying that a verifier accepted a commitment: it says that the accepted material accumulated by the network is already the material needed by a decoder. In blockchain settings this distinction matters because data may need to be recovered after block acceptance by users, provers, archival nodes, or fraud-proof systems.

Commitments for Arbitrary Codes give a direct way to obtain this property. The builder first encodes the data into an erasure-codeword and then commits to that encoded object. The validity proof binds the committed object to the code, while the opening protocol reveals authenticated pieces of the same object. Therefore, once enough distinct accepted cells are collected, reconstruction can run the corresponding erasure decoder on exactly those cells.

This is the reason this project focuses on Commitments for Arbitrary Codes. If the commitment layer is hash-based and the proof system is transparent and hash/symmetric-based, then the construction can be post-quantum. At the same time, because the opened objects are authenticated codeword cells, the construction retains the repair path from sampling transcripts to decoded data.

## 5. Commitments for Arbitrary Codes: Concrete Construction

- **The Setup algorithm $\mathsf{Setup}(1^{\lambda}) \rightarrow {\sf pp}$:**
    1. Choose a hash function $\mathsf{H}: \{0, 1\}^* \rightarrow \{0, 1\}^{\lambda}$ with domain-separated cell, chain, and Merkle tree calls.
    2. Define the Reed-Solomon code ${\sf RS}[\mathbb{F}, {\sf U}, \rho]$ and its encoding algorithm $\mathcal{C}: \mathbb{F}^k \rightarrow \mathbb{F}^m$, where $\mathbb{F}$ is a finite field, ${\sf U}$ is the evaluation domain, $\rho$ is the code rate, $k$ is the input length, and $m=|{\sf U}|$ satisfies $k=\rho m$.
    3. Define the number of field elements $c$ in a cell.
    4. Define the reconstruction threshold $t=\left\lceil k/c\right\rceil$ in cells.
    5. Define the public LeanVM parameters $\mathsf{pp}_{\sf STARK}$.
    6. Output $\mathsf{pp}=(\mathsf{H},\mathbb{F},{\sf U},m,k,\rho,c,t,\mathsf{pp}_{\sf STARK})$.

- **The encoding algorithm $\mathsf{Com}({\sf pp},{\sf data})\rightarrow({\sf com},{\sf \tau})$:**
    1. Parse ${\sf data}$ into blobs ${\sf data}=(b_1,\ldots,b_n)$, where each blob has $k$ symbols.
    2. RS encode each blob into a codeword with $m$ symbols: for every $i\in[1,n]$, $\mathcal{C}(b_i)=w_i=(w_{i,1},\ldots,w_{i,m})\in\mathbb{F}^m$. The first $k$ symbols are systematic, i.e. for every $s\in[1,k]$, $w_{i,s}=b_{i,s}$.
    3. Form a matrix whose $i$-th row is $w_i$. Group every $c$ consecutive field elements as a cell, so each row has $\ell=m/c$ cells. Let $W_{i,j}=(w_{i,(j-1)c+1},\ldots,w_{i,jc})\in\mathbb{F}^c$ denote the $j$-th cell in row $i$.
    4. Hash every cell into a cell digest: for every $i\in[1,n]$ and $j\in[1,\ell]$, set $e_{i,j}=\mathsf{H}(W_{i,j})$.
    5. Hash-chain the systematic cell digests on each row: for every $i\in[1,n]$, set $r_i=\mathsf{H}(e_{i,1},\ldots,e_{i,t})$.
    6. Merkle-aggregate the row hashes: $\mathsf{root}_{\sf row}=\mathsf{Merkle.Com}(r_1,\ldots,r_n)$.
    7. For every column of cell digests, compute a column root: for every $j\in[1,\ell]$, set $C_j=\mathsf{Merkle.Com}(e_{1,j},\ldots,e_{n,j})$.
    8. Merkle-aggregate all column roots: ${\sf root}_{\sf col}=\mathsf{Merkle.Com}(C_1,\ldots,C_{\ell})$.
    9. Aggregate the row and column roots: $\mathsf{root}=\mathsf{H}({\sf root}_{\sf row},{\sf root}_{\sf col})$.
    10. Compute the public RS check vector $L$ outside the proof from the public parameters and $\mathsf{root}$ as described below.
    11. Generate a LeanVM STARK proof $\pi\leftarrow{\sf LeanVM}.{\sf Prove}({\sf pp}_{\sf STARK},{\sf stmt},{\sf witn},\mathcal{R})$, where
    $$
    \begin{aligned}
    \mathcal{R}=\{({\sf stmt},{\sf witn}) :\;&
    {\sf stmt}=(\{r_i\}_{i\in[1,n]},L,{\sf root}_{\sf col}),\quad {\sf witn}=\{w_i\}_{i\in[1,n]},\\
    &\forall i\in[1,n],j\in[1,\ell],\; e_{i,j}=\mathsf{H}(W_{i,j}),\\
    &\forall i\in[1,n],\; r_i=\mathsf{H}(e_{i,1},\ldots,e_{i,t}),\\
    &\mathsf{root}_{\sf row}=\mathsf{Merkle.Com}(r_1,\ldots,r_n),\\
    &\forall j\in[1,\ell],\; C_j=\mathsf{Merkle.Com}(e_{1,j},\ldots,e_{n,j}),\\
    &{\sf root}_{\sf col}=\mathsf{Merkle.Com}(C_1,\ldots,C_{\ell}),\\
    &\forall i\in[1,n],\; \langle L,w_i\rangle=0\},\\
    &\mathsf{root}=\mathsf{H}({\sf root}_{\sf row},{\sf root}_{\sf col}).
    \end{aligned}
    $$
    12. Open the outer Merkle authentication paths for all column roots: $\{{\sf auth}_j\}_{j\in[1,\ell]}={\sf Merkle.Open}(C_1,\ldots,C_{\ell},{\sf root})$.
    13. Output ${\sf com}=({\sf root},\pi)$ and ${\sf \tau}=(\{w_i\}_{i\in[1,n]},\{{\sf auth}_j\}_{j\in[1,\ell]})$.

- **The query algorithm ${\sf V}^{\pi,Q}_1({\sf com})\rightarrow{\sf tran}$:**
    1. Generate the query index set $Q\leftarrow{\sf Sample}(1^{\lambda})$.
    2. Set ${\sf tran}=(Q,\{W_{1,j},\ldots,W_{n,j},{\sf auth}_j\}_{j\in Q})$.

- **The verification algorithm ${\sf V}_2({\sf com},{\sf tran})\rightarrow b$:**
    1. Recompute $L$ from the same public computations as the prover.
    2. Verify the STARK proof by checking ${\sf LeanVM}.{\sf Verify}({\sf pp}_{\sf STARK},{\sf stmt},\pi)=1$.
    3. Verify the openings: compute $e_{i,j}=\mathsf{H}(W_{i,j})$ for every $i\in[1,n]$ and $j\in Q$, compute $C_j=\mathsf{Merkle.Com}(e_{1,j},\ldots,e_{n,j})$ for every $j\in Q$, and check ${\sf Merkle}.{\sf Verify}({\sf root},\{C_j,{\sf auth}_j\}_{j\in Q})=1$.
    4. If all checks pass, output $b=1$; otherwise output $0$.

- **The reconstruction algorithm ${\sf Ext}({\sf com},{\sf tran}_1,\ldots,{\sf tran}_z)\rightarrow{\sf data}/\bot$:**
    1. For every $a\in[1,z]$, parse ${\sf tran}_a=(Q_a,\{W_{1,j},\ldots,W_{n,j},{\sf auth}_j\}_{j\in Q_a})$.
    2. Check that ${\sf V}_2({\sf com},{\sf tran}_a)=1$ for all $a\in[1,z]$; otherwise return $\bot$.
    3. Let $I=Q_1\cup Q_2\cup\cdots\cup Q_z$ be the union of query index sets.
    4. Check $|I|\geq t$; if not, return $\bot$.
    5. Reconstruct the data from the codeword symbols contained in the cells indexed by $I$: ${\sf data}={\sf Reconst}(\{W_{1,j},\ldots,W_{n,j}\}_{j\in I})$.

- **RS membership check used in the demo:** The current demo uses the special barycentric check for rate $\rho=1/2$.
    1. Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$, where $\omega$ is a primitive $m$-th root of unity, and assume $m=2k=2h$.
    2. Define $x_r=(\omega^2)^r$ for $r\in[0,h-1]$.
    3. For each row $w_i$, define $A_i(x_r)=w_{i,2r}$ and $B_i(x_r)=w_{i,2r+1}$.
    4. Sample $p\leftarrow\mathsf{H}({\sf pp},{\sf root})$ and set $q=p/\omega$.
    5. Define $\ell_r(z)=\frac{z^h-1}{h}\cdot\frac{x_r}{z-x_r}$.
    6. Compute the shared barycentric-check vector $L=(L_0,\ldots,L_{m-1})$, where $L_{2r}=\ell_r(p)$ and $L_{2r+1}=-\ell_r(q)$.
    7. Inside the proof, check for every $i\in[1,n]$ that
    $$
    \begin{aligned}
    \langle L,w_i\rangle
    &=\sum_{j=0}^{m-1}L_jw_{i,j}
      =\sum_{r=0}^{h-1}\ell_r(p)w_{i,2r}-\sum_{r=0}^{h-1}\ell_r(q)w_{i,2r+1}\\
    &=A_i(p)-B_i(q)=A_i(p)-B_i(p/\omega)=0.
    \end{aligned}
    $$

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

- **Main outcome:** A repairable post-quantum Commitments-for-Arbitrary-Codes DAS construction can be implemented with hash-based commitments and LeanVM proofs at roughly $0.9$ MiB/s across the strongest measured repairable profiles, with single-profile runs occasionally reaching about $1$ MiB/s.

- **Design choice:** The construction keeps sampled cells useful for reconstruction. This is the practical advantage of the Commitments-for-Arbitrary-Codes route over approaches optimized only for sampling or proximity testing.

- **Parameter choice:** Cell size $c=32$ is the strongest current point for the 2x profile, $c=32$ and $c=64$ are both competitive for the 4x profile, and row counts around $n=12$ to $n=14$ avoid the large proving-time cliffs seen at exact larger powers of two.

- **Main bottleneck:** The proof relation is still dominated by Poseidon calls for cell/row/column commitments and extension-field operations for RS membership. Reducing these costs inside LeanVM is the clearest path toward higher throughput.
