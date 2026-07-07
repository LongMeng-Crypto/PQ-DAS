# PQ-DAS V3 Demo


<style>
table th, table td { white-space: nowrap; }
</style>

V3 keeps the V2 cell-digest commitment path, but replaces the public commitment components with one final digest. The row hashes are Merkle-aggregated into $root_{\rm row}$, the column roots are Merkle-aggregated into $root_{\rm col}$, and the public commitment is $root=H(root_{\rm row},root_{\rm col})$. This document now records the current benchmark organization: old V1/V2/V3-base results are kept as baselines, and the main V3-ext section is organized as one-variable-at-a-time parameter sweeps.


## Benchmark Naming

- **Format:** <span style="white-space:nowrap">`vX-field-bY-cZ-rN-wR`</span>.
- **Version:** `v1`, `v2`, or `v3` records the construction version.
- **Field:** `base` means KoalaBear payload symbols, and `ext` means quintic-extension payload symbols.
- **Blob size:** `b1`, `b2`, and `b4` denote the 1x, 2x, and 4x row payload profiles.
- **Cell size:** `c16`, `c32`, `c64`, or `c128` records the number of extension symbols per cell for extension profiles, or KoalaBear symbols per cell for base profiles.
- **Rows and WHIR:** `r14` means $n=14$ rows, and `w1` means WHIR log inverse rate $1$.

## Payload Convention

- **Effective payload:** one KoalaBear symbol contributes $31$ bits, and one quintic-extension symbol contributes $5\cdot31$ bits.
- **Canonical serialization:** one KoalaBear limb occupies four bytes, so extension rows are $160$, $320$, or $640$ KiB on disk for 1x, 2x, and 4x profiles.
- **LeanVM proving throughput:** effective payload divided by LeanVM proving time.
- **Full DAS throughput:** effective payload divided by the end-to-end builder-to-validator acceptance time.
- **WHIR rate notation:** `WHIR log inv rate = r` means the WHIR backend starts with inverse rate $2^r$; this does not change the PQ-DAS RS code rate $k/m$.

## Baseline Parameters

| Benchmark IDs | Rows $n$ | Symbol field | Challenge field | $k$ | $m$ | Cell size $c$ | Cells $\ell$ | Threshold $t$ | Opened cells | Public commitment | Membership |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| <span style="white-space:nowrap">`v1-base-b1-c64-r1-w1`</span>, <span style="white-space:nowrap">`v1-base-b1-c64-r4-w1`</span> | 1, 4 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 114 | <span style="white-space:normal">row hashes + column root</span> | `dot_product_be` |
| <span style="white-space:nowrap">`v2-base-b1-c64-r1-w1`</span>, <span style="white-space:nowrap">`v2-base-b1-c64-r14-w1`</span>, <span style="white-space:nowrap">`v2-base-b1-c64-r16-w1`</span> | 1, 14, 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | <span style="white-space:normal">row hashes + column root</span> | `dot_product_be` |
| <span style="white-space:nowrap">`v2-ext-b1-c16-r1-w1`</span>, <span style="white-space:nowrap">`v2-ext-b1-c16-r14-w1`</span>, <span style="white-space:nowrap">`v2-ext-b1-c16-r16-w1`</span> | 1, 14, 16 | Quintic extension | Quintic extension | 8192 | 16384 | 16 | 1024 | 512 | 19 | <span style="white-space:normal">row hashes + column root</span> | `dot_product_ee` |
| <span style="white-space:nowrap">`v3-base-b1-c64-r1-w1`</span>, <span style="white-space:nowrap">`v3-base-b1-c64-r14-w1`</span>, <span style="white-space:nowrap">`v3-base-b1-c64-r16-w1`</span> | 1, 14, 16 | KoalaBear | Quintic extension | 32768 | 65536 | 64 | 1024 | 512 | 19 | <span style="white-space:normal">final root</span> | `dot_product_be` |
| <span style="white-space:nowrap">`v3-base-b2-c128-r1-w1`</span>, <span style="white-space:nowrap">`v3-base-b2-c128-r14-w1`</span>, <span style="white-space:nowrap">`v3-base-b2-c128-r16-w1`</span> | 1, 14, 16 | KoalaBear | Quintic extension | 65536 | 131072 | 128 | 1024 | 512 | 19 | <span style="white-space:normal">final root</span> | `dot_product_be` |

## Baseline Benchmarks

| Benchmark ID | Payload | Read-only elements | Opened cells | Commitment size | Proof size | Sample size | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v1-base-b1-c64-r1-w1`</span> | 124 KiB | 327696 | 114 | 0.06 KB | 357.51 KB | 64.57 KB | 3.600s | 0.130s | included | 0.003s | 0.067s | n/a | n/a | n/a | 34.44 KiB/s | n/a | accepted |
| <span style="white-space:nowrap">`v1-base-b1-c64-r4-w1`</span> | 496 KiB | 327720 | 114 | 0.16 KB | 398.50 KB | 150.07 KB | 17.400s | 0.209s | included | 0.008s | 0.143s | n/a | n/a | n/a | 28.51 KiB/s | n/a | accepted |
| <span style="white-space:nowrap">`v2-base-b1-c64-r14-w1`</span> | 1736 KiB | 327800 | 19 | 0.47 KB | 333.95 KB | 72.51 KB | 4.049s | 0.127s | 0.044s | 0.004s | 0.237s | 1263288 | 123889 | 917504 | 428.75 KiB/s | n/a | accepted |
| <span style="white-space:nowrap">`v2-ext-b1-c16-r14-w1`</span> | 2170 KiB | 82040 | 19 | 0.47 KB | 327.61 KB | 89.14 KB | 3.882s | 0.077s | 0.044s | 0.005s | 0.169s | 1248980 | 152561 | 229376 | 559.00 KiB/s | n/a | accepted |
| <span style="white-space:nowrap">`v3-base-b1-c64-r14-w1`</span> | 1736 KiB | 327688 | 19 | 0.03 KB | 334.26 KB | 73.11 KB | 4.011s | 0.116s | 0.040s | 0.004s | 0.232s | 1263336 | 123905 | 917504 | 432.81 KiB/s | n/a | accepted |
| <span style="white-space:nowrap">`v3-base-b2-c128-r14-w1`</span> | 3472 KiB | 655368 | 19 | 0.03 KB | 331.42 KB | 139.61 KB | 7.274s | 0.207s | 0.045s | 0.008s | 0.524s | 1607400 | 238593 | 1835008 | 477.32 KiB/s | n/a | accepted |

## Throughput Definitions

- **LeanVM proving throughput:** useful effective payload divided by LeanVM proving time: $D_{\rm payload}/T_{\rm prove}$.
- **Full DAS throughput:** useful effective payload divided by the end-to-end time from builder receipt of data until validator acceptance.
- **Network assumption:** builder upload and verifier download use $B_{\rm up}=B_{\rm down}=50$ Mbps.
- **Stage data:** for extension profiles, $D_{\rm payload}=n\cdot k\cdot5\cdot31/8$, $D_{\rm codeword}=n\cdot m\cdot5\cdot4$, $D_{\rm up}=D_{\rm codeword}+D_{\rm commitment}+D_{\rm proof}$, and $D_{\rm down}=D_{\rm commitment}+D_{\rm proof}+D_{\rm sample}$.
- **Full denominator:** $T_{\rm full}=T_{\rm encode+commit}+T_{\rm preprocess}+T_{\rm prove}+T_{\rm opening}+T_{\rm verifier\ rebuild}+T_{\rm proof\ verify}+T_{\rm verify\ openings}+D_{\rm up}/B_{\rm up}+D_{\rm down}/B_{\rm down}$.
- **Full DAS throughput:** $D_{\rm payload}/T_{\rm full}$. Reconstruction is excluded because it is not part of the normal validator acceptance path.

## V3-Ext Sweep A: Blob Size

- Fixed parameters: $\ell=1024$, $n=14$, $t=512$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: blob size, with $c$ scaled so that $\ell=m/c$ stays fixed.

| Benchmark ID | Blob size | $k$ | $m$ | $c$ | $\ell$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b1-c16-r14-w1`</span> | 1x | 8192 | 16384 | 16 | 1024 | 2170 KiB | 327.36 KB | 89.73 KB | 0.243s | 0.063s | 3.757s | 0.062s | 0.041s | 0.005s | 0.147s | 1349380 | 152577 | 229376 | 577.59 KiB/s | 431.67 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 2x | 16384 | 32768 | 32 | 1024 | 4340 KiB | 367.91 KB | 172.86 KB | 0.495s | 0.076s | 4.808s | 0.081s | 0.040s | 0.009s | 0.313s | 1779460 | 295937 | 458752 | 902.66 KiB/s | 609.05 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 4x | 32768 | 65536 | 64 | 1024 | 8680 KiB | 388.76 KB | 339.11 KB | 0.932s | 0.125s | 9.566s | 0.121s | 0.045s | 0.020s | 0.636s | 2639620 | 582657 | 917504 | 907.38 KiB/s | 623.21 KiB/s | accepted |

## V3-Ext Sweep B: Cell Size

- Fixed parameters: blob size $=2x$, $n=14$, $k=16384$, $m=32768$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Benchmark ID | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b2-c16-r14-w1`</span> | 16 | 2048 | 1024 | 29 | -58.625 | 4340 KiB | 343.39 KB | 137.86 KB | 0.514s | 0.131s | 9.346s | 0.139s | 0.046s | 0.008s | 0.356s | 2697988 | 305153 | 458752 | 464.37 KiB/s | 368.20 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 32 | 1024 | 512 | 19 | -83.398 | 4340 KiB | 367.91 KB | 172.86 KB | 0.521s | 0.088s | 5.128s | 0.092s | 0.045s | 0.010s | 0.344s | 1779460 | 295937 | 458752 | 846.33 KiB/s | 578.60 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c64-r14-w1`</span> | 64 | 512 | 256 | 14 | -97.448 | 4340 KiB | 369.10 KB | 249.43 KB | 0.516s | 0.066s | 5.350s | 0.071s | 0.046s | 0.017s | 0.351s | 1320196 | 291329 | 458752 | 811.21 KiB/s | 563.94 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c128-r14-w1`</span> | 128 | 256 | 128 | 11 | -57.495 | 4340 KiB | 368.95 KB | 388.14 KB | 0.531s | 0.063s | 5.441s | 0.072s | 0.047s | 0.024s | 0.356s | 1090564 | 289025 | 458752 | 797.65 KiB/s | 554.24 KiB/s | accepted |

## V3-Ext Sweep C: Row Count

- Fixed parameters: blob size $=2x$, $c=32$, $k=16384$, $m=32768$, $\ell=1024$, $t=512$, opened cells $=19$, WHIR log inverse rate $=1$.
- Variable parameter: row count $n$.

| Benchmark ID | $n$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r1-w1`</span> | 1 | 310 KiB | 277.36 KB | 18.48 KB | 0.038s | 0.088s | 0.667s | 0.075s | 0.029s | 0.001s | 0.045s | 172682 | 20991 | 32768 | 464.77 KiB/s | 282.65 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r2-w1`</span> | 2 | 620 KiB | 295.76 KB | 30.36 KB | 0.064s | 0.073s | 1.168s | 0.075s | 0.033s | 0.001s | 0.064s | 294073 | 41983 | 65536 | 530.82 KiB/s | 359.29 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r4-w1`</span> | 4 | 1240 KiB | 316.00 KB | 54.11 KB | 0.125s | 0.072s | 1.878s | 0.077s | 0.036s | 0.003s | 0.104s | 536855 | 83967 | 131072 | 660.28 KiB/s | 455.40 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r6-w1`</span> | 6 | 1860 KiB | 325.06 KB | 77.86 KB | 0.199s | 0.083s | 2.286s | 0.078s | 0.037s | 0.004s | 0.144s | 807308 | 128001 | 196608 | 813.65 KiB/s | 541.42 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r8-w1`</span> | 8 | 2480 KiB | 312.74 KB | 101.61 KB | 0.268s | 0.080s | 3.374s | 0.079s | 0.038s | 0.005s | 0.182s | 1021395 | 167935 | 262144 | 735.03 KiB/s | 516.45 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r10-w1`</span> | 10 | 3100 KiB | 342.79 KB | 125.36 KB | 0.335s | 0.079s | 4.014s | 0.078s | 0.036s | 0.006s | 0.215s | 1347188 | 216069 | 327680 | 772.30 KiB/s | 541.06 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r12-w1`</span> | 12 | 3720 KiB | 343.16 KB | 149.11 KB | 0.378s | 0.074s | 4.261s | 0.084s | 0.039s | 0.008s | 0.270s | 1563324 | 256003 | 393216 | 873.03 KiB/s | 596.23 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 14 | 4340 KiB | 367.91 KB | 172.86 KB | 0.483s | 0.081s | 4.892s | 0.085s | 0.039s | 0.009s | 0.311s | 1779460 | 295937 | 458752 | 887.16 KiB/s | 602.28 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r16-w1`</span> | 16 | 4960 KiB | 331.52 KB | 196.61 KB | 0.547s | 0.083s | 7.310s | 0.090s | 0.050s | 0.010s | 0.360s | 1993547 | 335871 | 524288 | 678.52 KiB/s | 500.58 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r18-w1`</span> | 18 | 5580 KiB | 352.98 KB | 220.36 KB | 0.640s | 0.084s | 8.578s | 0.091s | 0.044s | 0.013s | 0.419s | 2424900 | 392205 | 589824 | 650.50 KiB/s | 485.67 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r20-w1`</span> | 20 | 6200 KiB | 361.98 KB | 244.11 KB | 0.820s | 0.090s | 9.607s | 0.101s | 0.044s | 0.014s | 0.462s | 2641036 | 432139 | 655360 | 645.36 KiB/s | 479.44 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r22-w1`</span> | 22 | 6820 KiB | 362.04 KB | 267.86 KB | 0.822s | 0.090s | 10.542s | 0.096s | 0.045s | 0.015s | 0.514s | 2857172 | 472073 | 720896 | 646.94 KiB/s | 484.40 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r24-w1`</span> | 24 | 7440 KiB | 361.67 KB | 291.61 KB | 0.894s | 0.090s | 10.705s | 0.099s | 0.045s | 0.015s | 0.545s | 3073308 | 512007 | 786432 | 695.00 KiB/s | 512.01 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r26-w1`</span> | 26 | 8060 KiB | 388.51 KB | 315.36 KB | 0.955s | 0.093s | 11.664s | 0.101s | 0.045s | 0.018s | 0.610s | 3289444 | 551941 | 851968 | 691.02 KiB/s | 510.73 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r28-w1`</span> | 28 | 8680 KiB | 390.23 KB | 339.11 KB | 1.065s | 0.090s | 12.433s | 0.101s | 0.045s | 0.019s | 0.655s | 3505580 | 591875 | 917504 | 698.14 KiB/s | 514.45 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r30-w1`</span> | 30 | 9300 KiB | 388.67 KB | 362.86 KB | 1.137s | 0.093s | 12.130s | 0.102s | 0.046s | 0.020s | 0.710s | 3721716 | 631809 | 983040 | 766.69 KiB/s | 551.58 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r32-w1`</span> | 32 | 9920 KiB | 350.37 KB | 386.61 KB | 1.211s | 0.095s | 21.419s | 0.322s | 0.058s | 0.023s | 1.080s | 3935803 | 671743 | 1048576 | 463.14 KiB/s | 372.07 KiB/s | accepted |

## V3-Ext Sweep D: 4x Cell Size

- Fixed parameters: blob size $=4x$, $n=14$, $k=32768$, $m=65536$, WHIR log inverse rate $=1$.
- Variable parameter: cell size $c$, which changes $\ell=m/c$, $t=k/c$, and the opened-cell count.

| Benchmark ID | $c$ | $\ell$ | $t$ | Opened cells | $\log_2\nu_{\rm rep}$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b4-c16-r14-w1`</span> | 16 | 4096 | 2048 | 50 | -110.560 | 8680 KiB | 366.48 KB | 239.26 KB | 1.050s | 0.276s | 18.111s | 0.371s | 0.045s | 0.012s | 0.841s | 5395204 | 610305 | 917504 | 479.27 KiB/s | 378.04 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r14-w1`</span> | 32 | 2048 | 1024 | 29 | -58.625 | 8680 KiB | 390.29 KB | 264.74 KB | 0.925s | 0.154s | 9.838s | 0.160s | 0.039s | 0.013s | 0.622s | 3558148 | 591873 | 917504 | 882.29 KiB/s | 609.71 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 64 | 1024 | 512 | 19 | -83.398 | 8680 KiB | 388.76 KB | 339.11 KB | 0.960s | 0.117s | 10.026s | 0.131s | 0.045s | 0.019s | 0.661s | 2639620 | 582657 | 917504 | 865.75 KiB/s | 602.07 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c128-r14-w1`</span> | 128 | 512 | 256 | 14 | -97.448 | 8680 KiB | 388.73 KB | 494.43 KB | 0.996s | 0.100s | 10.616s | 0.109s | 0.044s | 0.027s | 0.674s | 2180356 | 578049 | 917504 | 817.63 KiB/s | 577.27 KiB/s | accepted |

## V3-Ext Sweep E: 4x Row Count

- Fixed parameters: blob size $=4x$, $c=32$, $k=32768$, $m=65536$, $\ell=2048$, $t=1024$, opened cells $=29$, WHIR log inverse rate $=1$.
- Variable parameter: row count $n$.

| Benchmark ID | $n$ | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r1-w1`</span> | 1 | 620 KiB | 293.60 KB | 29.11 KB | 0.079s | 0.159s | 1.085s | 0.164s | 0.034s | 0.002s | 0.108s | 345226 | 41983 | 65536 | 571.43 KiB/s | 338.11 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r2-w1`</span> | 2 | 1240 KiB | 324.15 KB | 47.24 KB | 0.144s | 0.157s | 2.064s | 0.164s | 0.039s | 0.003s | 0.148s | 587961 | 83967 | 131072 | 600.78 KiB/s | 399.43 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r4-w1`</span> | 4 | 2480 KiB | 333.82 KB | 83.49 KB | 0.272s | 0.157s | 4.209s | 0.176s | 0.042s | 0.005s | 0.243s | 1073431 | 167935 | 262144 | 589.21 KiB/s | 425.90 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r6-w1`</span> | 6 | 3720 KiB | 342.45 KB | 119.74 KB | 0.444s | 0.168s | 4.674s | 0.179s | 0.044s | 0.007s | 0.340s | 1614220 | 256001 | 393216 | 795.89 KiB/s | 538.65 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r8-w1`</span> | 8 | 4960 KiB | 331.17 KB | 155.99 KB | 0.596s | 0.171s | 7.119s | 0.180s | 0.047s | 0.009s | 0.431s | 2042323 | 335871 | 524288 | 696.73 KiB/s | 499.31 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r10-w1`</span> | 10 | 6200 KiB | 360.48 KB | 192.24 KB | 0.782s | 0.173s | 9.641s | 0.212s | 0.048s | 0.012s | 0.564s | 2693748 | 432133 | 655360 | 643.09 KiB/s | 472.75 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r12-w1`</span> | 12 | 7440 KiB | 361.54 KB | 228.49 KB | 0.948s | 0.177s | 11.070s | 0.188s | 0.046s | 0.013s | 0.629s | 3125948 | 512003 | 786432 | 672.09 KiB/s | 492.24 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r14-w1`</span> | 14 | 8680 KiB | 390.29 KB | 264.74 KB | 1.038s | 0.168s | 11.937s | 0.192s | 0.045s | 0.015s | 0.703s | 3558148 | 591873 | 917504 | 727.15 KiB/s | 525.99 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c32-r16-w1`</span> | 16 | 9920 KiB | 353.09 KB | 300.99 KB | 1.199s | 0.180s | 21.605s | 0.404s | 0.051s | 0.017s | 1.062s | 3986251 | 671743 | 1048576 | 459.15 KiB/s | 367.73 KiB/s | accepted |

## V3-Ext Sweep F: WHIR Rate

- Fixed candidates: <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> and <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span>.
- Variable parameter: WHIR log inverse rate $r\in\{1,2\}$ under the default LeanVM folding factors.

| Benchmark ID | WHIR log inv rate | Payload | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w1`</span> | 1 | 4340 KiB | 367.91 KB | 172.86 KB | 0.517s | 0.091s | 5.498s | 0.093s | 0.047s | 0.011s | 0.359s | 1779460 | 295937 | 458752 | 789.38 KiB/s | 551.19 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b2-c32-r14-w2`</span> | 2 | 4340 KiB | 243.73 KB | 172.86 KB | 0.544s | 0.090s | 7.138s | 0.097s | 0.029s | 0.009s | 0.350s | 1779460 | 295937 | 458752 | 608.01 KiB/s | 457.65 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w1`</span> | 1 | 8680 KiB | 388.76 KB | 339.11 KB | 1.076s | 0.130s | 11.287s | 0.138s | 0.050s | 0.020s | 0.780s | 2639620 | 582657 | 917504 | 769.03 KiB/s | 548.67 KiB/s | accepted |
| <span style="white-space:nowrap">`v3-ext-b4-c64-r14-w2`</span> | 2 | 8680 KiB | 253.98 KB | 339.11 KB | 1.076s | 0.136s | 15.620s | 0.142s | 0.032s | 0.021s | 0.733s | 2639620 | 582657 | 917504 | 555.70 KiB/s | 431.80 KiB/s | accepted |

- WHIR log inverse rate $r$ means the WHIR proof-system RS rate is $2^{-r}$, so $r=1$ is rate $1/2$ and $r=2$ is rate $1/4$.
- Rates $r=3,4$ correspond to WHIR rates $1/8$ and $1/16$, but the target PQ-DAS V3-ext profiles panic in WHIR config construction with `Increase folding_factor_0` under LeanVM's default `WHIR_INITIAL_FOLDING_FACTOR=7`. Supporting them would require changing the global WHIR initial folding factor and synchronizing verifier/recursion configuration, so they are not included as a one-variable benchmark sweep.

## Current Takeaways

- **Blob-size sweep:** 2x and 4x both approach $0.9$ MiB/s in LeanVM proving throughput at $n=14$; under the end-to-end payload-throughput definition, the recorded 2x and 4x full DAS throughput values are lower because upload includes the full encoding.
- **2x cell-size sweep:** $c=32$ remains the best current point for LeanVM proving throughput; $c=16$ creates too many cells, and $c=64/128$ reduce VM cycles without improving wall-clock proving time.
- **2x row-count sweep:** the newest full row-count sweep shows $n=12$ and $n=14$ as the strongest practical points in this run, with $n=14$ reaching $887.16$ KiB/s LeanVM proving throughput and about $602$ KiB/s full DAS payload throughput.
- **2x high-row behavior:** rows $18$ through $30$ are stable but do not beat $n=12/14$ in this run; $n=32$ again shows a proving-time cliff and should not be used as the local default.
- **4x cell-size sweep:** with $n=14$, $c=32$ is fastest in this run and $c=64$ is close; $c=16$ is clearly slower, while $c=128$ has fewer VM cycles but worse wall-clock proving time and larger samples.
- **4x row-count sweep:** the local 4x row-count sweep is intentionally kept at $n\leq16$ because larger 4x profiles exceed the workstation memory budget; server-side single-profile tests can still use the larger profile definitions.
- **WHIR-rate sweep:** WHIR log inverse rate $2$ reduces proof size but slows proving; rates $3/4$ need a LeanVM WHIR folding-factor change, so rate $1$ remains the throughput setting.
