# PQ-DAS Supplementary Material

## Benchmark Tables
- **Format:** `bY-cZ-rN-wR`.
- **Blob size:** `b1`, `b2`, and `b4` denote the 1x, 2x, and 4x row payload profiles.
- **Cell size:** `c16`, `c32`, `c64`, and `c128` record the number of extension-field symbols per cell.
- **Rows and WHIR:** `r14` means $n=14$ rows, and `w1` means WHIR log inverse rate $1$.
### Parameter Summary

All profiles in this table use the KoalaBear quintic extension field for payload symbols, Fiat-Shamir challenge points, and RS membership inner products.

| Profile family | Rows $n$ | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `b1-c16-r14-w1` | 14 | 8192 | 16384 | 16 | 1024 | 512 | 19 |
| `b2-c32-r14-w1` | 14 | 16384 | 32768 | 32 | 1024 | 512 | 19 |
| `b4-c64-r14-w1` | 14 | 32768 | 65536 | 64 | 1024 | 512 | 19 |
| `b2-c16-r14-w1` | 14 | 16384 | 32768 | 16 | 2048 | 1024 | 29 |
| `b2-c64-r14-w1` | 14 | 16384 | 32768 | 64 | 512 | 256 | 14 |
| `b2-c128-r14-w1` | 14 | 16384 | 32768 | 128 | 256 | 128 | 11 |
| `b4-c16-r14-w1` | 14 | 32768 | 65536 | 16 | 4096 | 2048 | 50 |
| `b4-c32-r14-w1` | 14 | 32768 | 65536 | 32 | 2048 | 1024 | 29 |
| `b4-c128-r14-w1` | 14 | 32768 | 65536 | 128 | 512 | 256 | 14 |
### Table 1. Blob-Size Sweep

- Fixed parameters: $\ell=1024$, $n=14$, $t=512$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: blob size, with $c$ scaled so that $\ell=m/c$ stays fixed.

| Profile | Blob size | $k$ | $m$ | $c$ | $\ell$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b1-c16-r14-w1` | 1x | 8192 | 16384 | 16 | 1024 | 2170 KiB | 327.36 KB | 89.73 KB | 0.243s | 0.063s | 3.757s | 0.062s | 0.041s | 0.005s | 0.147s | 1349380 | 152577 | 229376 | 577.59 KiB/s | 431.67 KiB/s | accepted |
| `ext-b2-c32-r14-w1` | 2x | 16384 | 32768 | 32 | 1024 | 4340 KiB | 367.91 KB | 172.86 KB | 0.495s | 0.076s | 4.808s | 0.081s | 0.040s | 0.009s | 0.313s | 1779460 | 295937 | 458752 | 902.66 KiB/s | 609.05 KiB/s | accepted |
| `ext-b4-c64-r14-w1` | 4x | 32768 | 65536 | 64 | 1024 | 8680 KiB | 388.76 KB | 339.11 KB | 0.932s | 0.125s | 9.566s | 0.121s | 0.045s | 0.020s | 0.636s | 2639620 | 582657 | 917504 | 907.38 KiB/s | 623.21 KiB/s | accepted |

### Table 2. Cell-Size Sweep at 2x Blob Size

- Fixed parameters: blob size $=2x$, $n=14$, $k=16384$, $m=32768$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Profile | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b2-c16-r14-w1` | 16 | 2048 | 1024 | 29 | -58.625 | 4340 KiB | 343.39 KB | 137.86 KB | 0.514s | 0.131s | 9.346s | 0.139s | 0.046s | 0.008s | 0.356s | 2697988 | 305153 | 458752 | 464.37 KiB/s | 368.20 KiB/s | accepted |
| `ext-b2-c32-r14-w1` | 32 | 1024 | 512 | 19 | -83.398 | 4340 KiB | 367.91 KB | 172.86 KB | 0.521s | 0.088s | 5.128s | 0.092s | 0.045s | 0.010s | 0.344s | 1779460 | 295937 | 458752 | 846.33 KiB/s | 578.60 KiB/s | accepted |
| `ext-b2-c64-r14-w1` | 64 | 512 | 256 | 14 | -97.448 | 4340 KiB | 369.10 KB | 249.43 KB | 0.516s | 0.066s | 5.350s | 0.071s | 0.046s | 0.017s | 0.351s | 1320196 | 291329 | 458752 | 811.21 KiB/s | 563.94 KiB/s | accepted |
| `ext-b2-c128-r14-w1` | 128 | 256 | 128 | 11 | -57.495 | 4340 KiB | 368.95 KB | 388.14 KB | 0.531s | 0.063s | 5.441s | 0.072s | 0.047s | 0.024s | 0.356s | 1090564 | 289025 | 458752 | 797.65 KiB/s | 554.24 KiB/s | accepted |

### Table 3. Row-Count Sweep at 2x Blob Size

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

### Table 4. Cell-Size Sweep at 4x Blob Size

- Fixed parameters: blob size $=4x$, $n=14$, $k=32768$, $m=65536$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Profile | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `ext-b4-c16-r14-w1` | 16 | 4096 | 2048 | 50 | -110.560 | 8680 KiB | 366.48 KB | 239.26 KB | 1.050s | 0.276s | 18.111s | 0.371s | 0.045s | 0.012s | 0.841s | 5395204 | 610305 | 917504 | 479.27 KiB/s | 378.04 KiB/s | accepted |
| `ext-b4-c32-r14-w1` | 32 | 2048 | 1024 | 29 | -58.625 | 8680 KiB | 390.29 KB | 264.74 KB | 0.925s | 0.154s | 9.838s | 0.160s | 0.039s | 0.013s | 0.622s | 3558148 | 591873 | 917504 | 882.29 KiB/s | 609.71 KiB/s | accepted |
| `ext-b4-c64-r14-w1` | 64 | 1024 | 512 | 19 | -83.398 | 8680 KiB | 388.76 KB | 339.11 KB | 0.960s | 0.117s | 10.026s | 0.131s | 0.045s | 0.019s | 0.661s | 2639620 | 582657 | 917504 | 865.75 KiB/s | 602.07 KiB/s | accepted |
| `ext-b4-c128-r14-w1` | 128 | 512 | 256 | 14 | -97.448 | 8680 KiB | 388.73 KB | 494.43 KB | 0.996s | 0.100s | 10.616s | 0.109s | 0.044s | 0.027s | 0.674s | 2180356 | 578049 | 917504 | 817.63 KiB/s | 577.27 KiB/s | accepted |

### Table 5. Row-Count Sweep at 4x Blob Size

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

### Table 6. WHIR-Rate Sweep

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

## RS Membership Check Instantiations

For the RS membership used in the leanVM proof, we ultimately want to check that each row of codeword $w_i$ is a valid RS codeword. We implement this via a simple inner product with a random vector $L$. Here, we show different ways to instantiate $L$, and explain which one we choose. Note that below we use a different hash function $\mathsf{H}'$ for Fiat-shamir transform, which could be a standard hash function such as SHA256, Keccak, or Blake.

### Parity-check: $\deg(P_i)<k \Rightarrow c_{i,k}=\cdots=c_{i,m-1}=0$

#### Preprocessing outside the proof:

* Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$, where $\omega$ is a primitive $m$-th root of unity.
* Let $i$ denote the row index, $j$ denote the codeword-symbol index on each row, and $r$ denote the coefficient index of the interpolated polynomial.
* For each row $w_i$, let $P_i(X)=\sum_{r=0}^{m-1}c_{i,r}X^r$ be the polynomial interpolated from its field symbols, where $\forall j \in[0, m-1]: P_i(\omega^j) = w_{i,j}$.
* The coefficients are given by $c_{i,r}=\frac{1}{m}\sum_{j=0}^{m-1} w_{i,j}\omega^{-jr}$.
* Use Fiat-shamir transform for deriving the random challenges $\{\alpha_r\}_{r \in [k,m-1]} \leftarrow \mathsf{H}'({\sf pp},{\sf root})$.
* Compute the shared parity-check vector $L=(L_0,\ldots,L_{m-1})$, where $\forall j\in[0,m-1]: L_j=\frac{1}{m}\sum_{r=k}^{m-1}\alpha_r\omega^{-jr}$.

#### Inner product inside the proof:

$$ \begin{aligned} \forall i\in[1,n]:\quad \langle L,w_i\rangle &= \sum_{j=0}^{m-1}L_jw_{i,j} = \sum_{j=0}^{m-1}\left(\frac{1}{m}\sum_{r=k}^{m-1}\alpha_r\omega^{-jr}\right)w_{i,j} \\ &= \sum_{r=k}^{m-1}\alpha_r\left(\frac{1}{m}\sum_{j=0}^{m-1}w_{i,j}\omega^{-jr}\right) = \sum_{r=k}^{m-1}\alpha_rc_{i,r} = 0. \end{aligned} $$

### General barycentric check

#### Preprocessing outside the proof:

* Let ${\sf U} = \{u_0,u_1,\ldots,u_{m-1}\}$.
* Let $i$ denote the row index, $j$ denote the codeword-symbol index on each row, and $s,t$ denote the interpolation and check positions.
* Choose $S\subseteq[0,m-1]$ with $|S|=k$, and let $T=[0,m-1]\setminus S$.
* For each $s\in S$, define the Lagrange basis polynomial $\ell_s(X)$ over ${u_s:s\in S}$, where $\ell_s(u_{s'})=1$ if $s=s'$ and $\ell_s(u_{s'})=0$ otherwise.
* For each row $w_i$, define $P_i(X)=\sum_{s\in S}\ell_s(X)w_{i,s}$.
* Use Fiat-shamir transform for deriving the random challenges $\{\alpha_r\}_{r \in [k,m-1]} \leftarrow \mathsf{H}'({\sf pp},{\sf root})$.
* Compute the shared barycentric-check vector $L=(L_0,\ldots,L_{m-1})$, where $\forall t\in T:L_t=\alpha_t$ and $\forall s\in S:L_s=-\sum_{t\in T}\alpha_t\ell_s(u_t)$.

#### Inner product inside the proof:

$$ \begin{aligned} \forall i\in[1,n]:\quad \langle L,w_i\rangle &= \sum_{j=0}^{m-1}L_jw_{i,j} = \sum_{t\in T}L_tw_{i,t}+\sum_{s\in S}L_sw_{i,s} \\ &= \sum_{t\in T}\alpha_tw_{i,t} -\sum_{s\in S}\left(\sum_{t\in T}\alpha_t\ell_s(u_t)\right)w_{i,s} \\ &= \sum_{t\in T}\alpha_t\left(w_{i,t}-\sum_{s\in S}\ell_s(u_t)w_{i,s}\right) = \sum_{t\in T}\alpha_t\left(w_{i,t}-P_i(u_t)\right) = 0. \end{aligned} $$

### Special barycentric check $(\rho = 1/2)$:

#### Preprocessing outside the proof:

* Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$, where $\omega$ is a primitive $m$-th root of unity, and assume $m=2k=2h$.
* Let $i$ denote the row index, $j$ denote the codeword-symbol index on each row, and $r$ denote the index on the half-size domain.
* Define $x_r=(\omega^2)^r$ for $r\in[0,h-1]$.
* For each row $w_i$, define $A_i(x_r)=w_{i,2r}$ and $B_i(x_r)=w_{i,2r+1}$.
* Use Fiat-shamir transform for deriving the random challenge $p \leftarrow\mathsf{H}'({\sf pp},{\sf root})$ and set $q=p/\omega$.
* Define $\ell_r(z)=\frac{z^h-1}{h}\cdot\frac{x_r}{z-x_r}$.
* Compute the shared barycentric-check vector $L=(L_0,\ldots,L_{m-1})$, where $\forall r\in[0,h-1]:L_{2r}=\ell_r(p)$ and $L_{2r+1}=-\ell_r(q)$.

#### Inner product inside the proof:

$$ \begin{aligned} \forall i\in[1,n]:\quad \langle L,w_i\rangle &= \sum_{j=0}^{m-1}L_jw_{i,j} = \sum_{r=0}^{h-1}L_{2r}w_{i,2r} +\sum_{r=0}^{h-1}L_{2r+1}w_{i,2r+1} \\ &= \sum_{r=0}^{h-1}\ell_r(p)w_{i,2r} -\sum_{r=0}^{h-1}\ell_r(q)w_{i,2r+1} = A_i(p)-B_i(q) \\ &= A_i(p)-B_i(p/\omega) = 0. \end{aligned} $$

## Subset Soundness Formula

The benchmark profiles use the subset-soundness with-replacement formula from [DAS foundations](https://eprint.iacr.org/2023/1079.pdf) (Lemma 3, Page 18). Let $N_{\sf clients}$ be the total number of client transcripts, let $\epsilon$ be the fraction of clients targeted by the adversary, and let $L_{\sf sub}=\lceil \epsilon N_{\sf clients}\rceil$ be the selected accepting subset size. Let $\Delta=t-1$ be the largest number of served cell columns that is still below the reconstruction threshold, and let $\ell=m/c$ be the total number of cell columns.

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
