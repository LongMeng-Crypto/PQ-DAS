# PQ-DAS Distributed Blob Proving

## Purpose

This document defines a distributed PQ-DAS construction in which each prover encodes and proves one blob, and sends the row hash, cell digests, and a proof to an aggregator. The aggregator proves the commitment for a full matrix of cell digests + all individual proofs are valid.

## DAS Construction

- The Setup algorithm $\mathsf{Setup}(1^{\lambda})\rightarrow{\sf pp}$:
    1. Choose a hash function $\mathsf{H}:\{0,1\}^*\rightarrow\{0,1\}^{\lambda}$ with domain-separated cell, row-chain, row-aggregation, Merkle-tree, and final-root calls. Every variable-length hash input is length-binding.
    2. Define the Reed-Solomon (RS) code ${\sf RS}[\mathbb{F},{\sf U},\rho]$ and its encoding algorithm $\mathcal{C}:\mathbb{F}^k\rightarrow\mathbb{F}^m$, where $k=\rho m$.
    3. Define the number of field elements $c$ in one cell, the number of cells $\ell=m/c$ in one codeword, and the reconstruction threshold $t=\lceil k/c\rceil$ in cells.
    4. Define the LeanVM programs for the first-row proof, recursive next-row proof, and aggregator proof, together with the recursive verifier and public parameters $\mathsf{pp}_{\sf STARK}$.
    5. Output $\mathsf{pp}=(\mathsf{H},\mathbb{F},{\sf U},m,k,\rho,c,\ell,t,\mathsf{pp}_{\sf STARK})$.
    6. The parameters $m$, $k$, $c$, $\ell$, and $t$ are fixed by the blob size and are therefore known to every prover.

- The distributed encoding algorithm $\forall i\in[1,n]: \mathsf{DisCom}({\sf pp}, b_i)\rightarrow({\sf com},{\sf \tau}_i)$:
    1. RS encode the blob: $w_i = \mathcal{C}(b_i)=(w_{i,1},\ldots,w_{i,m})\in\mathbb{F}^m$.
    2. Group every $c$ consecutive symbols into one cell: $\forall j\in[1,\ell]: W_{i,j}=(w_{i,(j-1)c+1},\ldots,w_{i,jc})\in\mathbb{F}^c$.
    3. Hash every cell into a cell digest:$\forall j\in[1,\ell]: e_{i,j}=\mathsf{H}(W_{i,j})$.
    4. Hash-chain all cell digests into the first row hash: $r_i = \mathsf{H}(e_{i,1},\ldots,e_{i,\ell})$.
    5. $\forall i \in[1, n]$: Compute the public RS check vector $L^{(i)}$ outside the proof from all public parameters so far. The details of how to compute $L^{(i)}$ can be referred to section "RS Membership Check Instantiations".
    6. Generate the row proof
    $\pi_i \leftarrow{\sf LeanVM.Prove}({\sf pp}_{\sf STARK}, r_i, w_i, \mathcal{R}_i)$, where
    $$
    \begin{aligned}
    \mathcal{R}_i = \{(\mathsf{stmt},\mathsf{witn}): & \mathsf{stmt} = (r_i, L^{(i)}), \mathsf{witn} = w_i \\
    \;&
    \langle L^{(i)}, w_i\rangle=0,\\
    &\forall j\in[1,\ell],\ e_{i,j} = \mathsf{H}(W_{i,j}),\\
    &r_i = \mathsf{H}(e_{i,1},\ldots,e_{i,\ell})\}.
    \end{aligned}
    $$
    7. Output ${\sf com}_i = (r_i,\pi_i)$ and ${\sf \tau}_i = (\{W_{i,j}\}_{j=1}^{\ell},\{e_{i,j}\}_{j=1}^{\ell})$.

- The aggregated encoding algorithm $\mathsf{AggCom}({\sf pp},\{{\sf com}_i\}_{i \in [1, n]},\{e_{i,j}\}_{i\in[1,n],j\in[1,\ell]})\rightarrow({\sf com},{\sf \tau})/\bot$:
    1. $\forall i \in [1, n]$: Parse ${\sf com}_i = (r_i,\pi_i)$, recompute $L^{(i)}$ from the same computations as done by the prover.
    2. Recompute every row hash: $\forall i\in[1,n]: r'_i=\mathsf{H}(e_{i,1},\ldots,e_{i,\ell})$, then check if $\forall i\in[1,n]: r_i = r'_i$. If not, output $\bot$.
    3. Verify all individual proofs: $\forall i \in [1, n]$: set $\mathsf{stmt}_i = (r_i, L^{(i)})$ and check ${\sf LeanVM.Verify}({\sf pp}_{\sf STARK}, \mathsf{stmt}_i ,\pi_i) = 1$. If any fails, output $\bot$.
    4. Arrange the cell digests into a $n\times\ell$ matrix $\{e_{i,j}\}_{i\in[1,n],j\in[1,\ell]}$.
    5. Generate a Merkle root for all row hashes:
    $\mathsf{root_{\sf row}} = \mathsf{Merkle.Com}(r_1, ...,  r_n)$.
    6. Generate one Merkle root for each column of cell digests: $\forall j\in[1,\ell]: C_j=\mathsf{Merkle.Com}(e_{1,j}, ...,  e_{n,j})$.
    7. Generate an outer column Merkle root: ${\sf root}_{\sf col}=\mathsf{Merkle.Com}(C_1, ...,  C_{\ell})$.
    8. Aggregate the row and column roots: ${\sf root}=\mathsf{H}({\sf root}_{\sf row}\parallel{\sf root}_{\sf col})$.
    9. Generate the aggregator proof
    $\pi \leftarrow{\sf LeanVM.Prove}({\sf pp}_{\sf STARK},{\sf root},\{e_{i,j}\}_{i\in[1,n],j\in[1,\ell]},\mathcal{R}_{\sf agg})$, where
    $$
    \begin{aligned}
    \mathcal{R}_{\sf agg}=\{(\mathsf{stmt}, \mathsf{witn}): & \mathsf{stmt} = \mathsf{root}, \mathsf{witn} = (\{\pi_i\}_{i \in [1, n]}, \{e_{i,j}\}_{i\in[1,n],j\in[1,\ell]}) \\ 
    &\forall i\in[1,n],\ r_i=\mathsf{H}(e_{i,1},\ldots,e_{i,\ell}),\\
    &\mathsf{root_{\sf row}} = \mathsf{Merkle.Com}(r_1, ...,  r_n),\\
    &\forall j\in[1,\ell],\ C_j=\mathsf{Merkle.Com}(e_{1,j}, ..., e_{n,j}),\\
    &{\sf root}_{\sf col}=\mathsf{Merkle.Com}(C_1, ..., C_{\ell}),\\
    &{\sf root}=\mathsf{H}({\sf root}_{\sf row}\parallel{\sf root}_{\sf col}), \\
    &\forall i \in[1, n]: {\sf Recompute} \ L^{(i)} \ {\sf from} \ r_i, {\sf pp}_{\sf STARK} \\
    &\forall i \in[1, n]: {\sf LeanVM.Verify}({\sf pp}_{\sf STARK}, (r_i, L^{(i)}) ,\pi_i) = 1\}.
    \end{aligned}
    $$
    10. Open the outer Merkle authentication path for every column root:
    $\{{\sf auth}_j\}_{j\in[1,\ell]}={\sf Merkle.Open}(C_1,\ldots,C_{\ell},{\sf root})$.
    12. Output ${\sf com}=({\sf root},\pi), {\sf \tau} = \{{\sf auth}_j\}_{j=1}^{\ell}$.

- The query algorithm ${\sf V}^{\pi,Q}_1({\sf com})\rightarrow{\sf tran}$:
    1. Generate the query index set $Q\leftarrow{\sf Sample}(1^{\lambda})$.
    2. Each prover $i\in[1,n]$ opens its cells $\{W_{i,j}\}_{j\in Q}$, and the aggregator supplies the corresponding outer paths $\{{\sf auth}_j\}_{j\in Q}$.
    3. Set ${\sf tran}=(Q,\{W_{1,j},\ldots,W_{n,j},{\sf auth}_j\}_{j\in Q})$.

- The verification algorithm ${\sf V}_2({\sf com},{\sf tran})\rightarrow b$:
    1. Parse ${\sf com}=({\sf root}, \pi)$, ${\sf tran}=(Q,\{W_{1,j},\ldots,W_{n,j},{\sf auth}_j\}_{j\in Q})$.
    2. Verify the STARK proof: ${\sf LeanVM.Verify}({\sf pp}_{\sf STARK},{\sf root},\pi)=1$.
    3. Verify the openings: Compute $\forall i\in[1,n],j\in Q:e_{i,j}=\mathsf{H}(W_{i,j})$ and $\forall j\in Q:C_j=\mathsf{Merkle.Com}(e_{1,j}, ..., e_{n,j})$, then check if ${\sf Merkle}.{\sf Verify}({\sf root}, \{C_j, {\sf auth}_j\}_{j \in Q}) = 1$.
    4. If all checks pass, output $b=1$; otherwise output $0$.

- The reconstruction algorithm ${\sf Ext}({\sf com},{\sf tran}_1,\ldots,{\sf tran}_L)\rightarrow{\sf data}/\bot$:
    1. For $a\in[1,L]$, parse ${\sf tran}_a=(Q_a,\{W_{1,j},\ldots,W_{n,j},{\sf auth}_j\}_{j\in Q_a})$.
    2. Check ${\sf V}_2({\sf com},{\sf tran}_a)=1$ for every $a\in[1,L]$; otherwise return $\bot$.
    3. Form the union $I=Q_1\cup\cdots\cup Q_L$ and return $\bot$ if $|I|<t$.
    4. Reconstruct
    $$
    {\sf data}={\sf Reconst}(\{W_{1,j},\ldots,W_{n,j}\}_{j\in I}).
    $$

---

## RS Membership Check Instantiations

Each prover proves only the membership of its own codeword $w_i\in{\sf RS}[\mathbb{F},{\sf U},\rho]$. Its check vector $L^{(i)}$ is derived independently outside the proof from public values, and only the inner product $\langle L^{(i)},w_i\rangle=0$ is proved inside LeanVM.

### 1. Parity-check: $\deg(P_i)<k\Rightarrow c_{i,k}=\cdots=c_{i,m-1}=0$

#### Preprocessing outside the proof:

* Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$, where $\omega$ is a primitive $m$-th root of unity.
* Let $P_i(X)=\sum_{r=0}^{m-1}c_{i,r}X^r$ be interpolated from $w_i$, where $P_i(\omega^j)=w_{i,j}$.
* The coefficients are $c_{i,r}=\frac{1}{m}\sum_{j=0}^{m-1}w_{i,j}\omega^{-jr}$.
* Sample $\{\alpha_{i,r}\}_{r\in[k,m-1]}\leftarrow\mathsf{H}({\sf pp},R_i)$.
* Compute $L^{(i)}=(L^{(i)}_0,\ldots,L^{(i)}_{m-1})$, where $L^{(i)}_j=\frac{1}{m}\sum_{r=k}^{m-1}\alpha_{i,r}\omega^{-jr}$.

#### Inner product inside the proof:

$$
\begin{aligned}
\langle L^{(i)},w_i\rangle
&=\sum_{j=0}^{m-1}L^{(i)}_jw_{i,j}
=\sum_{r=k}^{m-1}\alpha_{i,r}c_{i,r}=0.
\end{aligned}
$$

### 2. General barycentric check

#### Preprocessing outside the proof:

* Let ${\sf U}=\{u_0,u_1,\ldots,u_{m-1}\}$.
* Choose $S\subseteq[0,m-1]$ with $|S|=k$, and let $T=[0,m-1]\setminus S$.
* For each $s\in S$, define the Lagrange basis polynomial $\ell_s(X)$ over $\{u_s:s\in S\}$.
* Define $P_i(X)=\sum_{s\in S}\ell_s(X)w_{i,s}$.
* Sample $\{\alpha_{i,t}\}_{t\in T}\leftarrow\mathsf{H}({\sf pp},R_i)$.
* Compute $L^{(i)}$, where $L^{(i)}_t=\alpha_{i,t}$ for $t\in T$ and $L^{(i)}_s=-\sum_{t\in T}\alpha_{i,t}\ell_s(u_t)$ for $s\in S$.

#### Inner product inside the proof:

$$
\begin{aligned}
\langle L^{(i)},w_i\rangle
&=\sum_{t\in T}\alpha_{i,t}
\left(w_{i,t}-\sum_{s\in S}\ell_s(u_t)w_{i,s}\right)\\
&=\sum_{t\in T}\alpha_{i,t}(w_{i,t}-P_i(u_t))=0.
\end{aligned}
$$

### 3. Special barycentric check $(\rho=1/2)$

#### Preprocessing outside the proof:

* Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$ and assume $m=2k=2h$.
* Define $x_r=(\omega^2)^r$, $A_i(x_r)=w_{i,2r}$, and $B_i(x_r)=w_{i,2r+1}$ for $r\in[0,h-1]$.
* Sample $p_i\leftarrow\mathsf{H}({\sf pp},R_i)$ and set $q_i=p_i/\omega$.
* Define $\ell_r(z)=\frac{z^h-1}{h}\cdot\frac{x_r}{z-x_r}$.
* Compute $L^{(i)}$, where $L^{(i)}_{2r}=\ell_r(p_i)$ and $L^{(i)}_{2r+1}=-\ell_r(q_i)$.

#### Inner product inside the proof:

$$
\begin{aligned}
\langle L^{(i)},w_i\rangle
&=\sum_{r=0}^{h-1}\ell_r(p_i)w_{i,2r}
-\sum_{r=0}^{h-1}\ell_r(q_i)w_{i,2r+1}\\
&=A_i(p_i)-B_i(p_i/\omega)=0.
\end{aligned}
$$

---

## Arbitrary-Cell Reconstruction

Let $S\subseteq[0,m-1]$ be the codeword-symbol indices contained in the verified cells and let $E=[0,m-1]\setminus S$ be the erased indices. The same sets $S$ and $E$ are shared by every row.

- Construct the shared erasure-locator polynomial
$$
Z(X)=\prod_{j\in E}(X-\omega^j).
$$
- For each row polynomial $f_i(X)$, define
$$
N_i(\omega^j)=
\begin{cases}
w_{i,j}Z(\omega^j),&j\in S,\\
0,&j\in E.
\end{cases}
$$
- Apply a size-$m$ IFFT to recover $N_i(X)$, divide exactly by the shared $Z(X)$, and evaluate $f_i$ on the systematic domain with a size-$k$ FFT.
- Prepare the locator polynomial, its size-$m$ domain evaluations, and its Newton inverse once and reuse them for all $n$ rows.

---

## Parameter Spreadsheet

The following table summarizes the flexible parameters in the current PQ-DAS V2 construction.

| Parameter | Definition |
|---|---|
| $\lambda$ | Security parameter. |
| $\mathsf{H}$ | Hash function and hash-to-field function. |
| $\mathbb{F}$ | Finite field used for RS encoding and STARK arithmetic. |
| ${\sf U}$ | RS evaluation domain. |
| $m$ | Number of RS symbols in each encoded codeword. |
| $k$ | Number of input symbols in each blob, satisfying $k=\rho m$. |
| $\rho$ | RS code rate. |
| $n$ | Number of blobs, equivalently the number of rows in the matrix. |
| $c$ | Number of field elements grouped into one cell. |
| $\ell$ | Number of cells on each row, where $\ell=m/c$. |
| $t = \left\lceil \frac{k}{c} \right\rceil$ | Reconstruction threshold measured in cells. |
| $\lvert Q\rvert$ | Number of queried columns/cells in one transcript. |
| $L$ | Number of accepted transcripts used for reconstruction. |
| Membership check type | Choice of RS membership check: parity-check, general barycentric check, or special barycentric check for $\rho=1/2$. |
| $\mathsf{pp}_{\sf STARK}$ | Public parameters used by LeanVM. |

## Subset-soundness parameters

In this section we define the subset-soundness formula and its security parameters , where we follow the subset-soundness/security-definition style of Hall-Andersen, Simkin, and Wagner, *Foundations of data availability sampling*, IACR ePrint 2023/1079, in the DAS security definitions section. We spell out both sampling modes because implementations may sample one verifier's queried columns either without replacement or with replacement.

- $N_{\sf clients}$: total number of client transcripts considered by subset-soundness.
- $\epsilon$: fraction of clients that the adversary attempts to make accept unavailable data.
- $L_{\sf sub}=\lceil\epsilon N_{\sf clients}\rceil$: size of the adversarially selected accepting client subset.
- $\Delta=t-1$: largest number of served cell columns that is still below the reconstruction threshold.
- $\ell=m/c$: total number of cell columns per encoded row.
- $|Q|$: number of columns opened by one verifier; in the with-replacement model this counts random draws, so duplicate draws may later be de-duplicated by an implementation.
- $\nu_{\sf sub}$: subset-soundness failure bound.
- $\lambda$: target sampling security level in bits.
- In words, $\nu_{\sf sub}$ upper-bounds the probability that there exists a non-reconstructing set of $\Delta<t$ cell columns and a subset of $L_{\sf sub}$ clients such that every selected client opens only columns inside that same $\Delta$-set; equivalently, the union of all openings from those clients still remains below the reconstruction threshold.

### Sampling without replacement

Subset-soundness formula: $\nu_{\sf sub}=\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}\left(\frac{\binom{\Delta}{|Q|}}{\binom{\ell}{|Q|}}\right)^{L_{\sf sub}} \leq 2^{-\lambda}$. 

Opening cells formula derivation:

- For one fixed bad served set of size $\Delta$, one verifier misses all unavailable columns with probability $p_{\sf bad}(\Delta,\ell,q)=\frac{\binom{\Delta}{q}}{\binom{\ell}{q}}$.
- Union-bound over the bad served set and the accepting client subset: $\nu_{\sf sub}=\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}\left(p_{\sf bad}(\Delta,\ell,|Q|)\right)^{L_{\sf sub}}$.
- Therefore $|Q|_{\min}=\min\left\{q\in\{1,\ldots,\ell\}:\log_2\binom{\ell}{\Delta}+\log_2\binom{N_{\sf clients}}{L_{\sf sub}}+L_{\sf sub}\left(\log_2\binom{\Delta}{q}-\log_2\binom{\ell}{q}\right)\leq-\lambda\right\}$.
- Since $\frac{\binom{\Delta}{q}}{\binom{\ell}{q}}=\prod_{a=0}^{q-1}\frac{\Delta-a}{\ell-a}\leq\left(\frac{\Delta}{\ell}\right)^q$, a conservative closed-form estimate is $\log_2\binom{\ell}{\Delta}+\log_2\binom{N_{\sf clients}}{L_{\sf sub}}+qL_{\sf sub}\log_2(\Delta/\ell)\leq-\lambda$.
- Thus $\displaystyle |Q|\geq\left\lceil \frac{\lambda + \log_2\binom{\ell}{\Delta} +\log_2\binom{N_{\sf clients}}{L_{\sf sub}}}{L_{\sf sub}\log_2(\ell/\Delta)} \right\rceil$.
- If $q>\Delta$, then $\binom{\Delta}{q}=0$ and the without-replacement failure probability is $0$ for this worst-case model, so the exact search always terminates by $q=\Delta+1$.

A concrete 128 KiB parameter set with $\epsilon=0.01$ and $2^{-40}$ target is:

| Parameter | Value |
|---|---:|
| $m$ | $65,536$ |
| $k$ | $32,768$ |
| $\rho$ | $1/2$ |
| $c$ | $64$ |
| $\ell=m/c$ | $1,024$ |
| $t=\lceil k/c\rceil$ | $512$ |
| $\Delta=t-1$ | $511$ |
| $N_{\sf clients}$ | $10,000$ |
| $\epsilon$ | $0.01$ |
| $L_{\sf sub}$ | $100$ |
| $\lambda$ | $40$ bits |
| $\|Q\|_{\min}$ | $19$ |

For this parameter set, $|Q|=19$ gives $\log_2\nu_{\sf sub}\approx -108.031$, while $|Q|=18$ gives $\log_2\nu_{\sf sub}\approx -5.134$. Thus, $|Q|=19$ can be a suitable choice in this setting.

### Sampling with replacement

Subset-soundness formula: $\nu_{\sf sub}=\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}\left(\frac{\Delta}{\ell}\right)^{|Q|L_{\sf sub}} \leq 2^{-\lambda}$. 

Opening cells formula derivation:

- For one fixed bad served set of size $\Delta$, one verifier makes $q$ independent draws with replacement and misses all unavailable columns with probability $p_{\sf bad}(\Delta,\ell,q)=\left(\frac{\Delta}{\ell}\right)^q$.
- Union-bound over the bad served set and the accepting client subset: $\nu_{\sf sub}=\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}\left(p_{\sf bad}(\Delta,\ell,|Q|)\right)^{L_{\sf sub}}$.
- Therefore $|Q|_{\min}=\min\left\{q\in\mathbb{Z}_{\geq1}:\log_2\binom{\ell}{\Delta}+\log_2\binom{N_{\sf clients}}{L_{\sf sub}}+qL_{\sf sub}\log_2(\Delta/\ell)\leq-\lambda\right\}$.
- Since $\log_2(\Delta/\ell)<0$, this gives the closed-form requirement $\displaystyle |Q|\geq\left\lceil \frac{\lambda + \log_2\binom{\ell}{\Delta} +\log_2\binom{N_{\sf clients}}{L_{\sf sub}}}{L_{\sf sub}\log_2(\ell/\Delta)} \right\rceil$.

Using the same 128 KiB parameter set:

| Parameter | Value |
|---|---:|
| $m$ | $65,536$ |
| $k$ | $32,768$ |
| $\rho$ | $1/2$ |
| $c$ | $64$ |
| $\ell=m/c$ | $1,024$ |
| $t=\lceil k/c\rceil$ | $512$ |
| $\Delta=t-1$ | $511$ |
| $N_{\sf clients}$ | $10,000$ |
| $\epsilon$ | $0.01$ |
| $L_{\sf sub}$ | $100$ |
| $\lambda$ | $40$ bits |
| $\|Q\|_{\min}$ | $19$ |

For this parameter set, $|Q|=19$ gives $\log_2\nu_{\sf sub}\approx -83.398$, while $|Q|=18$ gives $\log_2\nu_{\sf sub}\approx 16.884$. Thus, $|Q|=19$ can be a suitable choice in this setting.

