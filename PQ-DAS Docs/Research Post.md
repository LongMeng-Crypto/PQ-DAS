# PQ-DAS constructed from LeanVM: Design and Benchmark 

<style>
table th, table td { white-space: nowrap; }
table td:nth-child(1) { white-space: nowrap; }
</style>

## 1. Motivation

Ethereum uses data availability sampling (DAS) to let validators check the availability of large blob data by sampling a small number of random positions from an erasure-coded object, rather than downloading the whole payload. As Ethereum moves toward post-quantum security, the current DAS protocol based on KZG polynomial commitment needs post-quantum alternatives. This post first shows an encode + prove type of PQ-DAS construction instantiated with Reed-Solomon code, hash commitments, and the LeanVM proof system, then shows the benchmark for various input parameters and output metrics. The code for the up-to-date implementation is at: [LongMeng-Crypto/PQ-DAS](https://github.com/LongMeng-Crypto/PQ-DAS/tree/V2%2FV3-Demo).

## 2. Encode + Prove DAS: Workflow

In general, a DAS protocol consists of a set of users, a block builder, a set of verifiers. The benchmarked DAS protocol in this report is the **Encode + Prove** paradigm from the solution space of the [DAS foundations, Section 7](https://eprint.iacr.org/2023/1079.pdf). In this class, the builder encodes the data with an erasure code, commits to the codewords by using a vector commitment scheme, and proves that the committed object is a valid codeword by using a SNARK proof system. Sampling opens authenticated positions of the codeword symbols, while reconstruction uses accepted samples as erasure-code evaluations. 

Abstractly, the workflow of such a DAS protocol is as follows:

- **Initialization**: All users send their data to the builder. 
- **Commit**: The builder receives data from the users, first encodes them and commits to the codeword, then it generates a SNARK proof for proving that the data are encoded into the codewords, and the codewords are commited as the commitment. Finally, the builder uploads the commitment, the SNARK proof, and the codewords including commitment openings for all symbols.
- **Download Commitment**: Each verifier downloads the full commitment and SNARK proof. Each verifier checks if the SNARK proof verifies with respect to the commitment. If not, they reject immediately.
- **Sampling and Verification**: Each verifier samples a random set of indices, queries the network, and downloads the corresponding codeword symbols and their openings. Each verifier checks if the openings are valid against the commitment. If not, they reject. If they are all valid, they accept.

Intuitively, if any party collects enough symbols that have been verified, the original data can be reconstructed and missing symbols can be reinserted into the network, to help other verifiers to eventually accept.

Informally, a DAS scheme should satisfy the following security properties: completeness, soundness, consistency, subset soundness, and repairability. 

The formal definition of the DAS syntax and the security properties are defined in [DAS foundations](https://eprint.iacr.org/2023/1079.pdf).


**Other schemes**. We do not focus on other post-quantum alternatives such as [FRIDA](https://eprint.iacr.org/2024/248.pdf) and [ZODA](https://angeris.github.io/papers/da-construction.pdf). This is because they do not satisfy a crucial property called *repairability*, which is achieved by the current KZG solution and implicitly assumed throughout the protocol. Informally, this property ensures that reconstructed symbols can be reinserted into the network and verify with respect to the (potentially maliciously generated) commitment. This ensures that parties will eventually agree on whether data is available or not. Without this property, one would have to significantly adapt the surrounding protocol. 

## 3. Concrete Construction
After introducing the overall workflow, we will now make more precise how the class of constructions work. In particular, we will define how the commitment is computed and which codes are used. While doing so, we also introduce the parameters that we will vary in our benchmarks.

The following image illustrates the workflow:

![image](https://hackmd.io/_uploads/S13ZCjfEzg.png)

For the construction, we use the Poseidon hash function, denoted as $\mathsf{H}$; we fix the erasure code as Reed-Solomon code ${\sf RS}[\mathbb{F},{\sf U},\rho]$; we choose Merkle tree commitment as the vector commitment scheme; and we use [LeanVM](https://github.com/leanEthereum/leanVM) as the SNARK proof system. 

Then we set the row length $k$, the encoded row length $m$, the cell size $c$, the number of cells per row $\ell=m/c$, and the reconstruction threshold $t=\lceil k/c\rceil$. Each data object is parsed into $n$ blob rows, each row is encoded as one Reed-Solomon codeword, and the first $k$ symbols are systematic payload symbols. With these paramerers, the protocol workflow is described as follows:

- **Initialization**: All users send their data to the builder.
- **Commit**: The builder operates following steps:
    - Encode each each blob of data into a RS codeword.
    - Arranges all codewords into a $n\times m$ matrix, where each row is one codeword. Then each row is split into $\ell$ consecutive cells $W_{i,j}$ of $c$ field elements, every cell is hashed into a digest $e_{i,j}=\mathsf{H}(W_{i,j})$, so the construction works from a matrix of cell digests rather than directly from raw codeword symbols after this point.
    - For every row $i$, the first $t$ cell digests, which cover the systematic payload cells, are hash-chained into a row digest $r_i=\mathsf{H}(e_{i,1},\ldots,e_{i,t})$. The row digests are then Merkle-aggregated into $\mathsf{root}_{\sf row}$. This row commitment gives a compact binding to each row's systematic data.
    - For every cell column $j$, the digests $(e_{1,j},\ldots,e_{n,j})$ are Merkle-aggregated into a column root $C_j$. The column roots $(C_1,\ldots,C_{\ell})$ are then Merkle-aggregated into $\mathsf{root}_{\sf col}$. 
    - The row root $\mathsf{root}_{\sf row}$ and column root are further hashed together to form the public commitment $\mathsf{root}=\mathsf{H}(\mathsf{root}_{\sf row},\mathsf{root}_{\sf col})$.
    - The LeanVM proof $\pi$ binds these commitments to valid Reed-Solomon codewords. The private witness is the codeword matrix. Inside the proof, LeanVM proves the computations of: (1) the witness cells $W_{i,j}$ are hashed to all cell digests $e_{i,j}$, (2) the systematic cell digests are hash chained to row hashes and all row hashes are Merkle-aggregated into a $\mathsf{root}_{\sf row}$, (3) each column of cells are Merkle aggregated into $(C_1,\ldots,C_{\ell})$ and further aggregated into $\mathsf{root}_{\sf col}$, and the final aggregation equals the public $\mathsf{root}$. It also checks Reed-Solomon membership from the barycentric check for every row using a public vector $L$.
    - The vector $L$ is computed outside the proof from public data. In the benchmarked rate-$1/2$ setting, let ${\sf U}=\{1,\omega,\ldots,\omega^{m-1}\}$ and $m=2k=2h$. For $x_r=(\omega^2)^r$, each row can be viewed as two length-$h$ evaluations $A_i(x_r)=w_{i,2r}$ and $B_i(x_r)=w_{i,2r+1}$. Fiat-Shamir samples $p$ from the public parameters and $\mathsf{root}$, sets $q=p/\omega$, and defines the barycentric coefficients $L_{2r}=\ell_r(p)$ and $L_{2r+1}=-\ell_r(q)$. The proof checks $\langle L,w_i\rangle=0$ for every row, which is equivalent to $A_i(p)=B_i(p/\omega)$ for valid rate-$1/2$ codewords. The verifier independently recomputes the same $L$ from public data before verifying the LeanVM proof.
    - The builder generates the Merkle authentication paths for each column codeword cells $W_{1, j}, ..., W_{n, j}$ for $j \in [1, \ell]$.
    - Finally, the builder uploads all codeword cells $W_{i, j}$, the commitment $\mathsf{root}$, the leanVM proof $\pi$, and the Merkle tree openings for all column codeword cells.
- **Download Commitment**: Each verifier downloads the commitment $\mathsf{root}$ and leanVM proof $\pi$. Each verifier recomputes the vector $L$ in the same way as the builder does, checks if $\pi$ verifies with respect to $\mathsf{root}$. If not, they reject immediately.
- **Sampling and Verification**: A verifier samples a set $Q$ of cell-column indices, queries the network, then downloads the sampled columns $W_{1,j},\ldots,W_{n,j}$ for $j \in Q$, and their Merkle tree paths to the final $\mathsf{root}$. Each verifier checks if Merkle paths are valid against $L$ and $\mathsf{root}$. If not, they reject. If they are all valid, they accept.

## 4. Input Parameters
This section summarizes the input parameters of our benchmark. In our experiments, we keep some of them fixed, and vary the others for observing how they affect the efficency of the DAS protocol.
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

**Remarks.** The number of sampled cell columns opened by a verifier, denoted by $|Q|$, is decided by the desired subset-soundness level. The formula for deriving it is given in the [subset soundness formula](supplementary.md#subset-soundness-formula) section of the supplementary material. 

## 5. Benchmark Metrics
The benchmark numbers in this report were measured on a local PC with an Intel Core i9-14900 CPU, 32 logical CPUs (16 cores with 2 threads per core), 32 GiB memory, a single NUMA node, 36 MiB L3 cache, and AVX2 support. The benchmark uses the local default Rayon thread pool on this machine.

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
| Full DAS throughput | Effective payload divided by the critical builder-to-validator workflow time until a verifier accepts the block. |

These metrics are chosen because the main system quantity we care about is **Full DAS throughput**: how much useful blob payload can pass through the builder-to-validator acceptance path per second. The full DAS throughput is computed as

$$\frac{D_{\mathrm{payload}}}{T_{\mathrm{total}}}$$

where $D_{\mathrm{payload}}$ is the useful blob payload size, i.e. number of blobs times blob size with the extension-field bit-size correction. The total workflow contains one builder and $N_{\mathrm{clients}}$ verifiers:

$$T_{\mathrm{total}}=T_{\mathrm{builder}}+T_{\mathrm{verifiers}}$$

The builder-side time is

$$T_{\mathrm{builder}}=T_{\mathrm{encode+commit}}+T_{\mathrm{preprocess}}+T_{\mathrm{prove}}+T_{\mathrm{open}}+\frac{D_{\mathrm{codeword}}+D_{\mathrm{commit}}+D_{\mathrm{proof}}}{B_{\mathrm{upload}}}$$

and the verifier-side time is written as

$$T_{\mathrm{verifiers}}=\max_{a\in\{1,\ldots,N_{\mathrm{clients}}\}}T_{\mathrm{verifier}}^{(a)}$$

for

$$T_{\mathrm{verifier}}^{(a)}=T_{\mathrm{verifier rebuild}}+T_{\mathrm{verify proof}}+T_{\mathrm{verify openings}}+\frac{D_{\mathrm{commit}}+D_{\mathrm{proof}}+D_{\mathrm{sample}}}{B_{\mathrm{download}}}.$$ 

The formula above is an optimistic upper-bound model. It assumes all parties execute the protocol stages without idle gaps and only charges the measured local computation plus the modeled upload/download time; it is not an actual network simulation and does not include gossip latency, peer scheduling, or mempool/block-propagation effects. The upload and download times are computed from the uploaded/downloaded byte sizes and the assumed $50$ Mbps bandwidth (the assumption is made in terms of https://eips.ethereum.org/EIPS/eip-7870). The endpoint is verifier acceptance of a block, after proof verification and opening verification. Reconstruction is not included in Full DAS throughput, because reconstruction happens after validators have accepted the block rather than on the acceptance critical path.

$N_{\mathrm{clients}}$ is the number of verifier/client transcripts. In the benchmark tables, we set $N_{\mathrm{clients}}=10000$. The formula explicitly includes $N_{\mathrm{clients}}$ verifiers, but the throughput numbers assume the ideal parallel case where all verifiers compute at the same time and spend almost equal amount of time, so $T_{\mathrm{verifiers}}$ is the wall-clock time of one verifier rather than the sum over all verifiers.

## 6. Benchmark Profile Names

- **Format:** `ext-bY-cZ-rN-wR`.
- **Field:** `ext` means quintic-extension payload symbols and quintic-extension RS membership checks.
- **Blob size:** `b1`, `b2`, and `b4` denote the 1x, 2x, and 4x row payload profiles.
- **Cell size:** `c16`, `c32`, `c64`, and `c128` record the number of extension-field symbols per cell.
- **Rows and WHIR:** `r14` means $n=14$ rows, and `w1` means WHIR log inverse rate $1$.

### Extension-Field Parameter Summary

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

## 7. Benchmark Results

The benchmark sweeps vary blob size, cell size, row count, and WHIR rate around the extension-field construction summarized above. The raw tables are collected in the [benchmark tables](supplementary.md#benchmark-tables) section of the supplementary material; the main takeaways are:

- **Blob-size sweep:** At fixed $\ell=1024$ and $n=14$, moving from 1x to 2x/4x payloads amortizes fixed proof overhead. The best Full DAS throughput in this sweep is `ext-b4-c64-r14-w1` at $623.21$ KiB/s, while 2x and 4x have almost identical LeanVM proving throughput around $0.9$ MiB/s ([Table 1](supplementary.md#table-1-blob-size-sweep)).

- **2x cell-size sweep:** For $k=16384$, $m=32768$, and $n=14$, $c=32$ is the best measured point, with $846.33$ KiB/s LeanVM proving throughput and $578.60$ KiB/s Full DAS throughput. Larger cells reduce VM cycles but increase opening size and do not improve the full throughput in this run ([Table 2](supplementary.md#table-2-cell-size-sweep-at-2x-blob-size)).

- **2x row-count sweep:** Increasing $n$ amortizes fixed overhead until padding cliffs appear. The best measured Full DAS throughput is at $n=14$ with $602.28$ KiB/s, while $n=16$ and $n=32$ show sharp proving-time cliffs ([Table 3](supplementary.md#table-3-row-count-sweep-at-2x-blob-size)).

- **4x cell-size sweep:** At 4x blob size, $c=32$ is the best measured point, with $882.29$ KiB/s LeanVM proving throughput and $609.71$ KiB/s Full DAS throughput. The $c=64$ point is close, but $c=16$ is much slower because it doubles the number of cells ([Table 4](supplementary.md#table-4-cell-size-sweep-at-4x-blob-size)).

- **4x row-count sweep:** The best measured Full DAS throughput is at $n=6$ with $538.65$ KiB/s. Larger row counts do not monotonically improve throughput because proof-system padding costs dominate at several boundaries, especially $n=16$ ([Table 5](supplementary.md#table-5-row-count-sweep-at-4x-blob-size)).

- **WHIR-rate sweep:** WHIR log inverse rate $1$ is consistently faster than rate $2$ for both tested profiles. Rate $2$ reduces proof size but increases proving time enough to lower Full DAS throughput ([Table 6](supplementary.md#table-6-whir-rate-sweep)).

For the complete measured values, including proof size, sample size, VM cycles, Poseidon16 calls, ExtensionOp calls, and reconstruction time, see the [benchmark tables](supplementary.md#benchmark-tables) in the supplementary material.

## Results Summary

- **Main outcome:** A repairable post-quantum Commitments-for-Arbitrary-Codes DAS construction can be implemented with hash-based commitments and LeanVM proofs at roughly $0.9$ MiB/s across the strongest measured repairable profiles, with single-profile runs occasionally reaching about $1$ MiB/s.

- **Design choice:** The construction keeps sampled cells useful for reconstruction. This is the practical advantage of the Commitments-for-Arbitrary-Codes route over approaches optimized only for sampling or proximity testing.

- **Parameter choice:** Cell size $c=32$ is the strongest current point for the 2x profile, $c=32$ and $c=64$ are both competitive for the 4x profile, and row counts around $n=12$ to $n=14$ avoid the large proving-time cliffs seen at exact larger powers of two.

- **Main bottleneck:** The proof relation is still dominated by Poseidon calls for cell/row/column commitments and extension-field operations for RS membership. Reducing these costs inside LeanVM is the clearest path toward higher throughput.
