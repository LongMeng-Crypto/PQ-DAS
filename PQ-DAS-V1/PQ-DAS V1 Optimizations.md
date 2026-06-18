# PQ-DAS V1 Runtime Comparison

The following are the performance comparison for the first PQ-DAS V1 demo between my i9-14900 (24 cores / 32 threads, AVX VNNI) and the ax42u (Ryzen 8700GE: 8 cores / 16 threads, AVX512), there is not much engineering-level optimizations on this demo at this time. All timings are in seconds. Both devices ran the same six parameter profiles.

## Desktop PC (i9-14900)

| Profile | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 0.000 | 0.005 | 0.100 | 0.028 | 0.000 |
| `medium` | 0.001 | 0.006 | 0.200 | 0.035 | 0.000 |
| `large` | 0.006 | 0.015 | 0.915 | 0.045 | 0.002 |
| `stress` | 0.043 | 0.016 | 6.812 | 0.055 | 0.006 |
| `blob-128k-1` | 0.023 | 0.077 | 3.600 | 0.130 | 0.003 |
| `blob-128k-4` | 0.097 | 0.079 | 17.400 | 0.209 | 0.008 |

## Server (ax42u)

| Profile | Encode + commit | Prover pareprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 0.000 | 0.005 | 0.031 | 0.027 | 0.000 |
| `medium` | 0.001 | 0.006 | 0.171 | 0.034 | 0.000 |
| `large` | 0.006 | 0.010 | 1.115 | 0.043 | 0.002 |
| `stress` | 0.046 | 0.014 | 9.398 | 0.053 | 0.007 |
| `blob-128k-1` | 0.026 | 0.084 | 4.643 | 0.121 | 0.003 |
| `blob-128k-4` | 0.094 | 0.084 | 19.015 | 0.125 | 0.008 |

## Optimizations 
Optimization list from 06/10/2026:
- Sample and open the minimum number of cells for the formal DAS sampler-quality bound at $124$-bit soundness.
    - For $T$ independent accepting transcripts, each opening $Q$ distinct cells sampled uniformly without replacement, use
      $\nu_{\mathrm{wor}}(\Delta,N,Q,T)=\binom{N}{\Delta}\left(\binom{\Delta}{Q}/\binom{N}{Q}\right)^T\le 2^{-124}$, where 
      - $N$ is the total number of cells
      - $Q$ is the number opened per transcript
      - $T$ is the number of transcripts available to the extractor
      - $\Delta=t-1$ is the largest number of distinct cells insufficient for reconstruction.
    - In this demo, $N=m/c$, $t=\lceil k/c\rceil$, $\Delta=t-1$, the security target is $\lambda=124$, and we set $T=128$ as an explicit benchmark assumption. The minimum integer $Q$ is selected for each profile.
    - Taking base-two logarithms, compute
      $Q_{\min}=\min\left\{q\in [1, N]:\log_2\binom{N}{\Delta}+T\left(\log_2\binom{\Delta}{q}-\log_2\binom{N}{q}\right)\le-\lambda\right\}$.
      We select the first integer $q$ that satisfies the $-124$ bound.
    - For the half-rate profiles, $\Delta\approx N/2$ and roughly
      $\log_2\nu_{\mathrm{wor}}\approx N-TQ$; increasing $N$ creates exponentially more possible unreconstructable sets, so a larger $Q$ is eventually required. 
      
        | Profile | $N$ cells | $t$ | $T$ | Min $Q$ | $\log_2\nu_{\mathrm{wor}}$ |
        | --- | ---: | ---: | ---: | ---: | ---: |
        | `tiny` | 2 | 1 | 128 | 1 | $-\infty$ |
        | `medium` | 32 | 16 | 128 | 2 | -257.64 |
        | `large` | 128 | 64 | 128 | 2 | -139.17 |
        | `stress` | 512 | 256 | 128 | 5 | -140.11 |
        | `blob-128k-1` | 1024 | 512 | 128 | 9 | -143.15 |
        | `blob-128k-4` | 1024 | 512 | 128 | 9 | -143.15 |
        | `blob-128k-16` | 1024 | 512 | 128 | 9 | -143.15 |

- Add proof size, commitment size, and sampled size in benchmark table.
- Reconstruct every systematic RS row from any $t=\lceil k/c\rceil$ distinct cells. 
    - Expanding the cells gives at least $k$ known evaluations $f(\omega^i)$
    - For missing symbol positions $E$, construct one shared erasure locator $Z(X)=\prod_{i\in E}(X-\omega^i)$.
    - Each row $r$ is a separate RS polynomial $f_r(X)$ and defines its own numerator evaluations:
         - $N_r(\omega^i)=f_r(\omega^i)Z(\omega^i)$ at known positions, while $N_r(\omega^i)=0$ at erasures because $Z(\omega^i)=0$.
        - Thus reconstruction uses one shared $Z(X)$ and $n$ independent numerators $N_r(X)$. For each row, recover $N_r(X)$ by an $m$-point IFFT, compute $f_r(X)=N_r(X)/Z(X)$ and evaluate $f_r$ on the systematic domain by a $k$-point FFT. 
- Add detailed baseline tables
    - LeanVM table with actual and power-of-two padded row counts.
    - Stage-level prover timing for execution, trace generation, access counting, commitments, logup, AIR sumcheck, WHIR, and grinding.
    - LeanVM VM profiling for instruction cycles, memory usage, Poseidon calls, and extension-field operations.
    - Relation isolation for row hashing, column hashing plus Merkle construction, and RS membership.

### Detailed benchmark tables

The following results were measured on the ax42u server with
`RUSTFLAGS="-C target-cpu=native"`. Runtime values are averages of three runs.

#### LeanVM Proving Share of End-to-End Runtime

| Profile | Average end-to-end time | Average LeanVM proving time | Proving share |
| --- | ---: | ---: | ---: |
| `tiny` | 0.062s | 0.030s | 48.7% |
| `medium` | 0.213s | 0.173s | 81.2% |
| `large` | 1.210s | 1.148s | 94.9% |
| `stress` | 9.539s | 9.417s | 98.7% |
| `blob-128k-1` | 4.900s | 4.665s | 95.2% |
| `blob-128k-4` | 19.296s | 18.985s | 98.4% |

- **Summary:** LeanVM proving accounts for 95-99% of runtime on the large profiles, so it is the primary optimization target.

#### LeanVM Prover Internal Cost

| Profile | WHIR | Stack + commit | Logup | AIR prep + sumcheck | Execute + trace | Other |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `large` | 35.3% | 26.8% | 21.0% | 7.1% | 6.5% | 3.3% |
| `stress` | 36.1% | 26.7% | 19.3% | 8.7% | 6.5% | 2.7% |
| `blob-128k-1` | 36.5% | 26.8% | 20.1% | 8.1% | 6.3% | 2.2% |
| `blob-128k-4` | 35.6% | 27.1% | 20.5% | 8.4% | 6.2% | 2.2% |

- **Summary:** WHIR, polynomial stacking and commitment, and logup consistently consume about 82-83% of proving time.

#### Isolated Relation VM Cycle Share

| Profile | Row hashes | Column hashes + Merkle | RS membership |
| --- | ---: | ---: | ---: |
| `medium` | 26.8% | 73.0% | 0.21% |
| `large` | 27.0% | 73.0% | 0.05% |
| `stress` | 27.1% | 72.9% | 0.01% |
| `blob-128k-1` | 28.7% | 71.3% | <0.01% |
| `blob-128k-4` | 29.2% | 70.8% | <0.01% |

- **Summary:** Column hashing and Merkle construction generate about 71-73% of VM cycles, followed by row hashing at about 27-29%.

#### Guest Execution and Memory-Traffic Overhead

| Profile | Total VM cycles | `hash_chunks` exclusive cycles | `main` exclusive cycles | `main` share |
| --- | ---: | ---: | ---: | ---: |
| `medium` | 86,811 | 7,056 | 79,713 | 91.8% |
| `large` | 682,345 | 57,504 | 624,759 | 91.6% |
| `stress` | 5,418,087 | 463,680 | 4,954,245 | 91.4% |
| `blob-128k-1` | 2,558,196 | 227,322 | 2,330,867 | 91.1% |
| `blob-128k-4` | 10,029,486 | 927,720 | 9,101,744 | 90.8% |

- **Summary:** About 91% of VM cycles remain in `main`, dominated by guest loops, indexing, temporary row/column copies, and Merkle-tree memory traffic rather than the `hash_chunks` wrapper itself.

#### LeanVM Table Padding

| Profile | Execution rows (actual/padded) | Utilization | Extension-op utilization | Poseidon utilization |
| --- | ---: | ---: | ---: | ---: |
| `medium` | 86,811 / 131,072 | 66.2% | 50.0% | 81.1% |
| `large` | 682,345 / 1,048,576 | 65.1% | 50.0% | 78.1% |
| `stress` | 5,418,087 / 8,388,608 | 64.6% | 50.0% | 76.6% |
| `blob-128k-1` | 2,558,196 / 4,194,304 | 61.0% | 50.0% | 81.2% |
| `blob-128k-4` | 10,029,486 / 16,777,216 | 59.8% | 50.0% | 76.6% |

- **Summary:** Execution-table padding wastes 34-40% of rows, so an optimization must cross a power-of-two boundary to produce the largest proving-time reduction.

### Sampling Correction + C8/C9/C10 Retained Result

The following results were measured on the ax42u server after reverting C1/C2/C4/C5/C6/C7 and retaining only the formal DAS sampling correction plus C8/C9/C10 environment support. This is the clean post-revert benchmark state.

#### Current Benchmark Table

| Profile | Bytecode instructions | Read-only elements | Opened cells | $\log_2\nu_{\mathrm{wor}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings | Reconstruct | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `tiny` | 512 | 104 | 1 | $-\infty$ | 96 B | 178,596 B | 100 B | 0.000s | 0.006s | 0.030s | 0.027s | 0.000s | 0.000s | Correct |
| `medium` | 512 | 1,352 | 2 | -257.638 | 288 B | 251,152 B | 840 B | 0.001s | 0.005s | 0.169s | 0.034s | 0.000s | 0.001s | Correct |
| `large` | 1,024 | 5,256 | 2 | -139.174 | 544 B | 299,444 B | 1,480 B | 0.006s | 0.010s | 1.112s | 0.043s | 0.000s | 0.004s | Correct |
| `stress` | 1,024 | 20,744 | 5 | -140.113 | 1,056 B | 367,208 B | 6,580 B | 0.047s | 0.014s | 9.327s | 0.056s | 0.000s | 0.029s | Correct |
| `blob-128k-1` | 1,024 | 327,696 | 9 | -143.150 | 64 B | 348,108 B | 5,220 B | 0.026s | 0.083s | 4.589s | 0.125s | 0.000s | 0.070s | Correct |
| `blob-128k-4` | 1,024 | 327,720 | 9 | -143.150 | 160 B | 389,500 B | 12,132 B | 0.095s | 0.083s | 18.948s | 0.124s | 0.001s | 0.111s | Correct |

- **Summary:** All profiles verify and reconstruct correctly. The retained code state mainly reduces sampled opening bandwidth; LeanVM proving remains the dominant cost.

#### Sampling and Opening Size Reduction

| Profile | Opened cells before | Opened cells now | Sample size before | Sample size now | Sample-size reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 1 | 1 | 100 B | 100 B | 0.0% |
| `medium` | 16 | 2 | 6,720 B | 840 B | 87.5% |
| `large` | 63 | 2 | 46,620 B | 1,480 B | 96.8% |
| `stress` | 105 | 5 | 138,180 B | 6,580 B | 95.2% |
| `blob-128k-1` | 114 | 9 | 66,120 B | 5,220 B | 92.1% |
| `blob-128k-4` | 114 | 9 | 153,672 B | 12,132 B | 92.1% |

- **Summary:** The major visible improvement is opening bandwidth: medium and larger profiles now need only 2-9 opened cell columns while still satisfying the $124$-bit sampler-quality target with $T=128$.

#### LeanVM Proving Time vs Previous Server Baseline

| Profile | Previous server LeanVM prove | Current LeanVM prove | Change |
| --- | ---: | ---: | ---: |
| `tiny` | 0.031s | 0.030s | -3.2% |
| `medium` | 0.171s | 0.169s | -1.2% |
| `large` | 1.115s | 1.112s | -0.3% |
| `stress` | 9.398s | 9.327s | -0.8% |
| `blob-128k-1` | 4.643s | 4.589s | -1.2% |
| `blob-128k-4` | 19.015s | 18.948s | -0.4% |

- **Summary:** C8/C9/C10 preserve the expected native-build and thread-configuration setup, but they do not materially change the protocol or the LeanVM proving relation. The small proving-time changes are within normal benchmark noise.

#### C-Parallel Attempt Outcome

| Attempted item | Outcome |
| --- | --- |
| C1/C2/C4 guest-loop parallel rewrites | Reverted. They reduced a small execution-side sub-stage but did not improve total proving time. |
| C5/C6 prover access-count parallelism | Reverted. The atomic-counter implementation made access counting slower on the server. |
| C7 statement finalization parallelism | Reverted. The measured stage was too small to justify keeping extra complexity. |
| C8/C9/C10 native CPU and thread configuration | Retained. These are useful environment-level settings and do not affect protocol soundness. |

- **Summary:** The clean conclusion is that the current successful optimization is the sampling correction; the attempted C-level parallel changes were measured and then removed because they were not net improvements.
