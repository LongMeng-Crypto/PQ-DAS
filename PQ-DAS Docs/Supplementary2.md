# PQ-DAS Supplementary Material 2

## Benchmark Tables

These tables report the latest V3-ext server sweep results measured with the same benchmark parameters as the main report on a stronger server: AMD EPYC 9V74 processor, 32 logical CPUs (16 cores with 2 threads per core), 62 GiB memory and AVX-512 support. The tables below record the server-side throughput, timing, proof-size, sample-size, and VM-counter results for comparison with the local-PC measurements in the main report.

- **All-path supplement:** All path size and Open all paths were measured separately on the same AMD EPYC 9V74 server, using one assembly thread, 3 warmups, and 100 assemblies per case; [raw measurements](benchmark-data/server-all-paths.json) and [per-row corrections](benchmark-data/server-all-paths-corrections.json) are retained. The original proving, sampling, and other measurements were not rerun.
- **Path accounting:** All seven tables use independent paths to the public column root: $D_{\mathrm{open,all}}=\ell(4+32\log_2\ell)$ bytes, including a 4-byte index per column. Whole-column openings require no inner-column paths; tree construction is already included in Encode + commit, while Open all paths measures extraction and serialization.
- **Full DAS correction:** $R_{\mathrm{new}}=P/(P/R_{\mathrm{old}}+T_{\mathrm{open,all}}+D_{\mathrm{open,all}}/6{,}250{,}000)$, where $P$ is payload in KiB, $R$ is in KiB/s, $T_{\mathrm{open,all}}$ is in seconds, and $D_{\mathrm{open,all}}$ is in bytes. This adds the omitted assembly and 50 Mbps upload costs to the prior total time, retaining sampled-response costs; corrected values inherit the rounding of the previously published throughput.

### Table 1. Blob-Size Sweep

- Fixed parameters: $\ell=1024$, $n=14$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: blob size, with $c$ scaled so that $\ell=m/c$ stays fixed.

| Profile | Blob size | $k$ | $m$ | $c$ | $\ell$ | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-14` | 1x | 8192 | 16384 | 16 | 1024 | 2170 KiB | 0.47 KB | 417.67 KB | 86.26 KB | 324.00 KiB | 0.023s | 0.089s | 2.057s | 0.113991 | 0.086s | 0.051s | 0.001s | 0.029s | 943222 | 152577 | 229376 | 1054.78 KiB/s | 668.40 KiB/s | accepted |
| `blob-ext-2x-14` | 2x | 16384 | 32768 | 32 | 1024 | 4340 KiB | 0.47 KB | 441.44 KB | 169.14 KB | 324.00 KiB | 0.033s | 0.112s | 3.851s | 0.113991 | 0.111s | 0.053s | 0.002s | 0.057s | 1373302 | 295937 | 458752 | 1126.89 KiB/s | 741.09 KiB/s | accepted |
| `blob-ext-4x-14` | 4x | 32768 | 65536 | 64 | 1024 | 8680 KiB | 0.47 KB | 464.17 KB | 335.61 KB | 324.00 KiB | 0.061s | 0.161s | 7.419s | 0.113991 | 0.161s | 0.056s | 0.004s | 0.119s | 2233462 | 582657 | 917504 | 1170.00 KiB/s | 784.99 KiB/s | accepted |

### Table 2. Cell-Size Sweep at 2x Blob Size

- Fixed parameters: blob size $=2x$, $n=14$, $k=16384$, $m=32768$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Profile | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-2x-c16-14` | 16 | 2048 | 1024 | 29 | -58.625 | 4340 KiB | 0.47 KB | 443.55 KB | 131.83 KB | 712.00 KiB | 0.055s | 0.175s | 3.928s | 0.251540 | 0.170s | 0.054s | 0.002s | 0.060s | 1886326 | 305153 | 458752 | 1104.98 KiB/s | 707.42 KiB/s | accepted |
| `blob-ext-2x-14` | 32 | 1024 | 512 | 19 | -83.398 | 4340 KiB | 0.47 KB | 441.44 KB | 169.14 KB | 324.00 KiB | 0.033s | 0.113s | 3.782s | 0.113991 | 0.112s | 0.054s | 0.002s | 0.057s | 1373302 | 295937 | 458752 | 1147.42 KiB/s | 749.62 KiB/s | accepted |
| `blob-ext-2x-c64-14` | 64 | 512 | 256 | 14 | -97.448 | 4340 KiB | 0.47 KB | 440.94 KB | 246.90 KB | 146.00 KiB | 0.031s | 0.085s | 3.749s | 0.051190 | 0.084s | 0.054s | 0.002s | 0.055s | 1116790 | 291329 | 458752 | 1157.78 KiB/s | 763.80 KiB/s | accepted |
| `blob-ext-2x-c128-14` | 128 | 256 | 128 | 11 | -57.495 | 4340 KiB | 0.47 KB | 416.99 KB | 386.32 KB | 65.00 KiB | 0.030s | 0.080s | 3.468s | 0.022806 | 0.080s | 0.054s | 0.003s | 0.054s | 988534 | 289025 | 458752 | 1251.33 KiB/s | 804.62 KiB/s | accepted |

### Table 3. Row-Count Sweep at 2x Blob Size

- Fixed parameters: blob size $=2x$, $c=32$, $k=16384$, $m=32768$, $\ell=1024$, $t=512$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: row count $n$.

| Profile | $n$ | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-2x-1` | 1 | 310 KiB | 0.06 KB | 329.96 KB | 14.79 KB | 324.00 KiB | 0.010s | 0.115s | 0.492s | 0.113991 | 0.111s | 0.044s | 0.000s | 0.040s | 146529 | 20991 | 32768 | 630.39 KiB/s | 298.02 KiB/s | accepted |
| `blob-ext-2x-2` | 2 | 620 KiB | 0.09 KB | 327.86 KB | 26.82 KB | 324.00 KiB | 0.009s | 0.110s | 0.742s | 0.113991 | 0.111s | 0.047s | 0.001s | 0.042s | 241764 | 41983 | 65536 | 835.67 KiB/s | 444.66 KiB/s | accepted |
| `blob-ext-2x-4` | 4 | 1240 KiB | 0.16 KB | 357.86 KB | 50.48 KB | 324.00 KiB | 0.013s | 0.111s | 1.349s | 0.113991 | 0.111s | 0.051s | 0.001s | 0.046s | 432232 | 83967 | 131072 | 919.38 KiB/s | 555.04 KiB/s | accepted |
| `blob-ext-2x-6` | 6 | 1860 KiB | 0.22 KB | 387.92 KB | 74.32 KB | 324.00 KiB | 0.018s | 0.112s | 1.729s | 0.113991 | 0.112s | 0.051s | 0.001s | 0.048s | 650371 | 128001 | 196608 | 1075.53 KiB/s | 653.87 KiB/s | accepted |
| `blob-ext-2x-8` | 8 | 2480 KiB | 0.28 KB | 380.83 KB | 97.73 KB | 324.00 KiB | 0.021s | 0.112s | 2.564s | 0.113991 | 0.112s | 0.053s | 0.001s | 0.050s | 812144 | 167935 | 262144 | 967.32 KiB/s | 636.44 KiB/s | accepted |
| `blob-ext-2x-10` | 10 | 3100 KiB | 0.34 KB | 384.35 KB | 121.64 KB | 324.00 KiB | 0.026s | 0.112s | 2.859s | 0.113991 | 0.111s | 0.053s | 0.001s | 0.053s | 996506 | 216069 | 327680 | 1084.44 KiB/s | 703.02 KiB/s | accepted |
| `blob-ext-2x-12` | 12 | 3720 KiB | 0.41 KB | 409.32 KB | 145.36 KB | 324.00 KiB | 0.029s | 0.113s | 3.351s | 0.113991 | 0.111s | 0.053s | 0.002s | 0.054s | 1184904 | 256003 | 393216 | 1109.95 KiB/s | 725.30 KiB/s | accepted |
| `blob-ext-2x-14` | 14 | 4340 KiB | 0.47 KB | 441.44 KB | 169.14 KB | 324.00 KiB | 0.033s | 0.113s | 3.823s | 0.113991 | 0.111s | 0.053s | 0.002s | 0.057s | 1373302 | 295937 | 458752 | 1135.29 KiB/s | 744.55 KiB/s | accepted |
| `blob-ext-2x-15` | 15 | 4650 KiB | 0.50 KB | 440.92 KB | 180.98 KB | 324.00 KiB | 0.034s | 0.113s | 3.877s | 0.113991 | 0.112s | 0.053s | 0.002s | 0.058s | 1467501 | 315904 | 491520 | 1199.36 KiB/s | 776.01 KiB/s | accepted |
| `blob-ext-2x-16` | 16 | 4960 KiB | 0.53 KB | 400.77 KB | 192.86 KB | 324.00 KiB | 0.036s | 0.113s | 4.952s | 0.113991 | 0.112s | 0.056s | 0.002s | 0.060s | 1561700 | 335871 | 524288 | 1001.68 KiB/s | 692.23 KiB/s | accepted |
| `blob-ext-2x-18` | 18 | 5580 KiB | 0.59 KB | 402.35 KB | 216.86 KB | 324.00 KiB | 0.042s | 0.114s | 5.087s | 0.113991 | 0.113s | 0.056s | 0.003s | 0.063s | 1954079 | 392205 | 589824 | 1096.83 KiB/s | 741.68 KiB/s | accepted |
| `blob-ext-2x-20` | 20 | 6200 KiB | 0.66 KB | 430.44 KB | 240.73 KB | 324.00 KiB | 0.045s | 0.114s | 6.232s | 0.113991 | 0.113s | 0.056s | 0.003s | 0.065s | 2117901 | 432139 | 655360 | 994.82 KiB/s | 697.02 KiB/s | accepted |
| `blob-ext-2x-22` | 22 | 6820 KiB | 0.72 KB | 431.98 KB | 264.39 KB | 324.00 KiB | 0.049s | 0.115s | 6.370s | 0.113991 | 0.114s | 0.056s | 0.003s | 0.067s | 2281723 | 472073 | 720896 | 1070.71 KiB/s | 737.19 KiB/s | accepted |
| `blob-ext-2x-24` | 24 | 7440 KiB | 0.78 KB | 431.25 KB | 288.17 KB | 324.00 KiB | 0.052s | 0.115s | 6.493s | 0.113991 | 0.117s | 0.056s | 0.002s | 0.068s | 2445545 | 512007 | 786432 | 1145.89 KiB/s | 775.53 KiB/s | accepted |
| `blob-ext-2x-26` | 26 | 8060 KiB | 0.84 KB | 464.33 KB | 311.82 KB | 324.00 KiB | 0.056s | 0.114s | 7.326s | 0.113991 | 0.114s | 0.056s | 0.003s | 0.071s | 2609367 | 551941 | 851968 | 1100.24 KiB/s | 756.70 KiB/s | accepted |
| `blob-ext-2x-28` | 28 | 8680 KiB | 0.91 KB | 464.93 KB | 335.64 KB | 324.00 KiB | 0.060s | 0.114s | 7.393s | 0.113991 | 0.113s | 0.056s | 0.002s | 0.073s | 2773189 | 591875 | 917504 | 1174.15 KiB/s | 793.81 KiB/s | accepted |
| `blob-ext-2x-30` | 30 | 9300 KiB | 0.97 KB | 464.86 KB | 359.20 KB | 324.00 KiB | 0.065s | 0.114s | 7.528s | 0.113991 | 0.114s | 0.056s | 0.003s | 0.075s | 2937011 | 631809 | 983040 | 1235.45 KiB/s | 823.73 KiB/s | accepted |
| `blob-ext-2x-32` | 32 | 9920 KiB | 1.03 KB | 427.46 KB | 383.23 KB | 324.00 KiB | 0.067s | 0.114s | 9.510s | 0.113991 | 0.115s | 0.058s | 0.003s | 0.077s | 3098784 | 671743 | 1048576 | 1043.08 KiB/s | 735.88 KiB/s | accepted |

### Table 4. WHIR-Rate Sweep

- Fixed candidates: `blob-ext-2x-14` and `blob-ext-4x-14`.
- Variable parameter: WHIR log inverse rate $r\in\{1,2\}$.

| Profile | WHIR log inv rate | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-2x-14` | 1 | 4340 KiB | 0.47 KB | 441.44 KB | 169.14 KB | 324.00 KiB | 0.035s | 0.114s | 3.888s | 0.113991 | 0.111s | 0.053s | 0.002s | 0.057s | 1373302 | 295937 | 458752 | 1116.34 KiB/s | 736.04 KiB/s | accepted |
| `blob-ext-2x-14` | 2 | 4340 KiB | 0.47 KB | 292.73 KB | 169.14 KB | 324.00 KiB | 0.033s | 0.112s | 4.617s | 0.113991 | 0.110s | 0.034s | 0.002s | 0.057s | 1373302 | 295937 | 458752 | 940.03 KiB/s | 662.31 KiB/s | accepted |
| `blob-ext-4x-14` | 1 | 8680 KiB | 0.47 KB | 464.17 KB | 335.61 KB | 324.00 KiB | 0.061s | 0.161s | 7.467s | 0.113991 | 0.161s | 0.056s | 0.003s | 0.121s | 2233462 | 582657 | 917504 | 1162.45 KiB/s | 781.58 KiB/s | accepted |
| `blob-ext-4x-14` | 2 | 8680 KiB | 0.47 KB | 305.97 KB | 335.61 KB | 324.00 KiB | 0.061s | 0.161s | 9.025s | 0.113991 | 0.162s | 0.036s | 0.003s | 0.121s | 2233462 | 582657 | 917504 | 961.73 KiB/s | 689.25 KiB/s | accepted |

### Table 5. Cell-Size Sweep at 4x Blob Size

- Fixed parameters: blob size $=4x$, $n=14$, $k=32768$, $m=65536$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Profile | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-4x-c16-14` | 16 | 4096 | 2048 | 50 | -110.560 | 8680 KiB | 0.47 KB | 465.85 KB | 227.45 KB | 1552.00 KiB | 0.078s | 0.344s | 7.649s | 0.549051 | 0.332s | 0.058s | 0.002s | 0.129s | 3772534 | 610305 | 917504 | 1134.75 KiB/s | 732.83 KiB/s | accepted |
| `blob-ext-4x-c32-14` | 32 | 2048 | 1024 | 29 | -58.625 | 8680 KiB | 0.47 KB | 464.75 KB | 258.86 KB | 712.00 KiB | 0.065s | 0.219s | 7.373s | 0.251540 | 0.219s | 0.056s | 0.002s | 0.122s | 2746486 | 591873 | 917504 | 1177.24 KiB/s | 776.17 KiB/s | accepted |
| `blob-ext-4x-14` | 64 | 1024 | 512 | 19 | -83.398 | 8680 KiB | 0.47 KB | 464.17 KB | 335.61 KB | 324.00 KiB | 0.061s | 0.162s | 7.336s | 0.113991 | 0.164s | 0.056s | 0.003s | 0.118s | 2233462 | 582657 | 917504 | 1183.20 KiB/s | 790.72 KiB/s | accepted |
| `blob-ext-4x-c128-14` | 128 | 512 | 256 | 14 | -97.448 | 8680 KiB | 0.47 KB | 440.29 KB | 491.99 KB | 146.00 KiB | 0.059s | 0.133s | 6.657s | 0.051190 | 0.133s | 0.056s | 0.004s | 0.118s | 1976950 | 578049 | 917504 | 1303.89 KiB/s | 848.64 KiB/s | accepted |

### Table 6. Row-Count Sweep at 4x Blob Size

- Fixed parameters: blob size $=4x$, $c=32$, $k=32768$, $m=65536$, $\ell=2048$, $t=1024$, opened cells $=29$, WHIR log inverse rate $=1$.
- Variable parameter: row count $n$.

| Profile | $n$ | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-4x-c32-1` | 1 | 620 KiB | 0.06 KB | 355.07 KB | 23.08 KB | 712.00 KiB | 0.015s | 0.225s | 0.873s | 0.251540 | 0.218s | 0.048s | 0.001s | 0.088s | 292961 | 41983 | 65536 | 710.40 KiB/s | 339.52 KiB/s | accepted |
| `blob-ext-4x-c32-2` | 2 | 1240 KiB | 0.09 KB | 362.45 KB | 41.43 KB | 712.00 KiB | 0.019s | 0.217s | 1.434s | 0.251540 | 0.217s | 0.051s | 0.001s | 0.091s | 483428 | 83967 | 131072 | 865.01 KiB/s | 476.84 KiB/s | accepted |
| `blob-ext-4x-c32-4` | 4 | 2480 KiB | 0.16 KB | 379.29 KB | 77.30 KB | 712.00 KiB | 0.025s | 0.218s | 2.556s | 0.251540 | 0.218s | 0.054s | 0.001s | 0.095s | 864360 | 167935 | 262144 | 970.44 KiB/s | 595.48 KiB/s | accepted |
| `blob-ext-4x-c32-6` | 6 | 3720 KiB | 0.22 KB | 409.45 KB | 113.83 KB | 712.00 KiB | 0.034s | 0.218s | 3.344s | 0.251540 | 0.218s | 0.054s | 0.002s | 0.102s | 1300611 | 256001 | 393216 | 1112.44 KiB/s | 689.15 KiB/s | accepted |
| `blob-ext-4x-c32-8` | 8 | 4960 KiB | 0.28 KB | 403.03 KB | 150.05 KB | 712.00 KiB | 0.042s | 0.219s | 4.924s | 0.251540 | 0.219s | 0.056s | 0.002s | 0.108s | 1624176 | 335871 | 524288 | 1007.40 KiB/s | 669.07 KiB/s | accepted |
| `blob-ext-4x-c32-10` | 10 | 6200 KiB | 0.34 KB | 405.74 KB | 186.71 KB | 712.00 KiB | 0.050s | 0.220s | 5.527s | 0.251540 | 0.219s | 0.056s | 0.002s | 0.114s | 1992858 | 432133 | 655360 | 1121.69 KiB/s | 733.51 KiB/s | accepted |
| `blob-ext-4x-c32-12` | 12 | 7440 KiB | 0.41 KB | 431.20 KB | 222.49 KB | 712.00 KiB | 0.058s | 0.220s | 6.512s | 0.251540 | 0.219s | 0.057s | 0.002s | 0.119s | 2369672 | 512003 | 786432 | 1142.46 KiB/s | 752.95 KiB/s | accepted |
| `blob-ext-4x-c32-14` | 14 | 8680 KiB | 0.47 KB | 464.75 KB | 258.86 KB | 712.00 KiB | 0.066s | 0.221s | 7.509s | 0.251540 | 0.219s | 0.056s | 0.003s | 0.123s | 2746486 | 591873 | 917504 | 1155.96 KiB/s | 766.67 KiB/s | accepted |
| `blob-ext-4x-c32-16` | 16 | 9920 KiB | 0.53 KB | 428.29 KB | 295.14 KB | 712.00 KiB | 0.073s | 0.220s | 9.604s | 0.251540 | 0.221s | 0.059s | 0.003s | 0.128s | 3123300 | 671743 | 1048576 | 1032.88 KiB/s | 716.69 KiB/s | accepted |

### Table 7. LeanVM Precompile Benchmarks

| Profile | $n$ | Payload | Commitment size | Proof size | Sample size | All path size | Encode + commit | Prover preprocess | LeanVM prove | Open all paths (ms) | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `blob-ext-2x-15` | 15 | 4650 KiB | 0.50 KB | 400.56 KB | 180.98 KB | 324.00 KiB | 0.043s | 0.066s | 3.393s | 0.113991 | 0.065s | 0.053s | 0.002s | 0.058s | 6 | 315904 | 491520 | 1370.57 KiB/s | 859.65 KiB/s | accepted |
| `blob-ext-2x-c128-14` | 14 | 4340 KiB | 0.47 KB | 400.07 KB | 386.32 KB | 65.00 KiB | 0.030s | 0.065s | 3.214s | 0.022806 | 0.065s | 0.053s | 0.005s | 0.054s | 6 | 289025 | 458752 | 1350.36 KiB/s | 850.12 KiB/s | accepted |
| `blob-ext-2x-30` | 30 | 9300 KiB | 0.97 KB | 420.91 KB | 359.20 KB | 324.00 KiB | 0.063s | 0.066s | 6.438s | 0.113991 | 0.064s | 0.056s | 0.003s | 0.074s | 6 | 631809 | 983040 | 1444.50 KiB/s | 921.93 KiB/s | accepted |
| `blob-ext-4x-c128-14` | 14 | 8680 KiB | 0.47 KB | 421.18 KB | 491.99 KB | 146.00 KiB | 0.060s | 0.114s | 6.238s | 0.051190 | 0.115s | 0.056s | 0.005s | 0.117s | 6 | 578049 | 917504 | 1391.57 KiB/s | 888.77 KiB/s | accepted |
| `blob-ext-4x-14` | 14 | 8680 KiB | 0.47 KB | 420.08 KB | 335.61 KB | 324.00 KiB | 0.062s | 0.114s | 6.224s | 0.113991 | 0.115s | 0.056s | 0.003s | 0.119s | 6 | 582657 | 917504 | 1394.65 KiB/s | 889.77 KiB/s | accepted |

## Summary

- Best LeanVM proving throughput in Tables 1-6 is `blob-ext-4x-c128-14` at 1303.89 KiB/s; it is also the best Full DAS throughput point at 848.64 KiB/s.
- Best 2x point in the cell-size sweep is `blob-ext-2x-c128-14`, with 1251.33 KiB/s LeanVM proving throughput and 804.62 KiB/s Full DAS throughput.
- Best 2x row-count point is `blob-ext-2x-30`, with 1235.45 KiB/s LeanVM proving throughput and 823.73 KiB/s Full DAS throughput.
- WHIR log inverse rate 2 reduces proof size but is slower overall: for 2x, Full DAS throughput drops from 736.04 to 662.31 KiB/s; for 4x, it drops from 781.58 to 689.25 KiB/s.
- Larger cells help proving throughput in this server run. At 4x and $n=14$, increasing $c$ from 64 to 128 improves Full DAS throughput from 790.72 to 848.64 KiB/s, despite the larger sample size.
- The LeanVM precompile results in Table 7 are the strongest server measurements: `blob-ext-2x-30` reaches 1444.50 KiB/s LeanVM proving throughput and 921.93 KiB/s Full DAS throughput, while `blob-ext-4x-14` is close behind at 1394.65 KiB/s and 889.77 KiB/s.
