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

### C-Parallel Optimization Attempt

The following results were measured on the ax42u server after trying C1, C2, C4, C5, C6, C7, C8, C9, and C10. This run uses the corrected DAS sampling rule with $T=128$, so opened cells and sample sizes are not directly comparable to the original fixed-withholding-set sampling rule. The C1/C2/C4/C5/C6/C7 code changes were later reverted because they did not improve total proving time; only the sampling correction and C8/C9/C10 environment support are retained.

#### End-to-End Runtime Table

| Profile | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 0.000 | 0.006 | 0.033 | 0.027 | 0.000 |
| `medium` | 0.001 | 0.005 | 0.167 | 0.035 | 0.000 |
| `large` | 0.006 | 0.010 | 1.119 | 0.043 | 0.000 |
| `stress` | 0.046 | 0.014 | 9.403 | 0.053 | 0.000 |
| `blob-128k-1` | 0.026 | 0.084 | 4.666 | 0.122 | 0.000 |
| `blob-128k-4` | 0.096 | 0.084 | 19.218 | 0.125 | 0.001 |

- **Summary:** LeanVM proving time is essentially unchanged compared with the previous server baseline. The guest-loop parallelism reduces some execution-side sub-stages, but the dominant WHIR, stack-and-commit, and logup costs still control total prover time.

#### Sampling and Opening Size

| Profile | Opened cells before | Opened cells after | Sample size before | Sample size after | Sample-size change |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny` | 1 | 1 | 100 B | 100 B | 0.0% |
| `medium` | 16 | 2 | 6,720 B | 840 B | -87.5% |
| `large` | 63 | 2 | 46,620 B | 1,480 B | -96.8% |
| `stress` | 105 | 5 | 138,180 B | 6,580 B | -95.2% |
| `blob-128k-1` | 114 | 9 | 66,120 B | 5,220 B | -92.1% |
| `blob-128k-4` | 114 | 9 | 153,672 B | 12,132 B | -92.1% |

- **Summary:** The large reduction in sample size comes from the corrected formal DAS sampler-quality calculation, not from C-level parallelism.

#### Prover Sub-Stage Comparison

| Profile | Bytecode execution | Trace generation | Memory access count | Bytecode access count | Statement finalization |
| --- | ---: | ---: | ---: | ---: | ---: |
| `medium` | 0.004s -> 0.004s (-14.7%) | 0.006s -> 0.005s (-2.2%) | 0.001s -> 0.002s (+110.3%) | 0.000s -> 0.001s (+132.6%) | 0.000s -> 0.000s (+4.6%) |
| `large` | 0.035s -> 0.029s (-17.6%) | 0.039s -> 0.038s (-3.1%) | 0.008s -> 0.018s (+125.6%) | 0.002s -> 0.006s (+177.1%) | 0.000s -> 0.000s (+9.8%) |
| `stress` | 0.295s -> 0.249s (-15.7%) | 0.317s -> 0.306s (-3.4%) | 0.064s -> 0.152s (+138.0%) | 0.016s -> 0.049s (+202.8%) | 0.000s -> 0.000s (-0.2%) |
| `blob-128k-1` | 0.138s -> 0.135s (-2.4%) | 0.153s -> 0.148s (-2.8%) | 0.032s -> 0.084s (+164.2%) | 0.008s -> 0.026s (+227.2%) | 0.001s -> 0.001s (+28.5%) |
| `blob-128k-4` | 0.569s -> 0.501s (-11.8%) | 0.617s -> 0.595s (-3.5%) | 0.127s -> 0.328s (+157.4%) | 0.032s -> 0.105s (+224.8%) | 0.001s -> 0.001s (+7.5%) |

- **Summary:** C1/C2/C4 reduced bytecode execution by roughly 2-18%, but this did not translate into end-to-end proving-time improvement. C5/C6's atomic-counter implementation made access-count construction slower on this server. These C1/C2/C4/C5/C6/C7 code changes were reverted; future C5/C6 work should use a lower-overhead sharded reduction before being treated as a net optimization.
