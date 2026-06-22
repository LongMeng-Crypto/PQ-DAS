---
title: PQ-DAS V2 Design
---

# PQ-DAS V2 Design

## Purpose

This document collects the second version of the PQ-DAS Encoding + Prove design, creates the spreadsheet with flexible variables in the design, and records the cell-digest commitment layout.

## DAS Construction

![PQ-DAS V2 commitment layout](<V2 Design Diagram.png>)

- The Setup algorithm $\mathsf{Setup}(1^{\lambda}) \rightarrow {\sf pp}$:
    1. Choose a hash function $\mathsf{H}: \{0, 1\}^* \rightarrow \{0, 1\}^{\lambda}$ with domain-separated cell, chain, Merkle-node, and final-root calls. Define $\mathsf{Chain}(x_1,\ldots,x_s)$ by $z_0=0^\lambda$, $z_j=\mathsf{H}(z_{j-1}\parallel x_j)$, and output $z_s$.
    2. Define the Reed-Solomon (RS) code ${\sf RS}[\mathbb{F}, {\sf U}, \rho]$ and its corresponding encoding algorithm $\mathcal{C}: \mathbb{F}^k \rightarrow \mathbb{F}^m$, where $\mathbb{F}$ is a finite field, ${\sf U}$ is the evaluation domain, $\rho$ is the code rate, $k$ is the length of the input vector, $m = |{\sf U}|$ is the length of the output vector, which satisfies $k = \rho m$.
    3. Define the number of field elements $c$ in a cell.
    4. Define the reconstruction threshold $t = \left\lceil \frac{k}{c} \right\rceil$ in cells.
    5. Define the public parameters used in LeanVM as $\mathsf{pp}_{\sf STARK}$.
    6. Output $\mathsf{pp} = (\mathsf{H}, \mathbb{F}, {\sf U}, m, k, \rho, c, t, \mathsf{pp}_{\sf STARK})$.

- The encoding algorithm $\mathsf{Com}({\sf pp}, {\sf data}) \rightarrow ({\sf com}, {\sf \tau})$:
    1. Parse ${\sf data}$ into a set of blobs ${\sf data} = (b_1, ..., b_n)$, each blob has $k$ symbols.
    2. RS Encode each blob such that each codeword has $m$ symbols, i.e., $\forall i \in [1, n]$: $\mathcal{C}(b_i) = w_i = (w_{i,1},\ldots,w_{i,m}) \in \mathbb{F}^m$. For each codeword, the first $k$ symbols are the systematic data, i.e., $\forall i \in [1, n], s \in [1, k]: w_{i,s} = b_{i,s}$.
    3. Form a matrix such that each row is a codeword $w_i$. We group every $c$ consecutive field elements in each $w_i$ as a cell, so there are in total $\ell = \frac{m}{c}$ cells on each row. We use $W_{i,j} = (w_{i,(j-1)c+1},\ldots,w_{i,jc}) \in \mathbb{F}^c$ to denote the $j$-th cell on the $i$-th row in the matrix.
    4. Hash each cell once, i.e., $\forall i \in [1,n], j \in [1,\ell]$: $e_{i,j}=\mathsf{H}(W_{i,j})$.
    5. Hash-chain the systematic cell digests on each row, i.e., $\forall i \in [1,n]$: $r_i=\mathsf{Chain}(e_{i,1},\ldots,e_{i,t})$.
    6. Hash-chain all row digests, i.e., ${\sf root}_{\sf row}=\mathsf{Chain}(r_1,\ldots,r_n)$.
    7. Generate one Merkle root for each column of cell digests, i.e., $\forall j \in [1,\ell]$: $C_j=\mathsf{Merkle.Com}(e_{1,j}\parallel\cdots\parallel e_{n,j})$.
    8. Generate a Merkle tree for all column roots, i.e., ${\sf root}_{\sf col}=\mathsf{Merkle.Com}(C_1\parallel\cdots\parallel C_{\ell})$.
    9. Aggregate the row and column commitments, i.e., ${\sf root}=\mathsf{H}({\sf root}_{\sf row}\parallel{\sf root}_{\sf col})$.
    10. Use LeanVM to generate a STARK proof $\pi \leftarrow {\sf LeanVM}.{\sf Prove}({\sf pp}_{\sf STARK}, {\sf stmt}, {\sf witn}, \mathcal{R})$, where
    \begin{aligned}
    \mathcal{R}
    =
    \{(\mathsf{stmt},\mathsf{witn}) \;:\;&
    \mathsf{stmt}=({\sf root}_{\sf row},{\sf root}_{\sf col},{\sf root}),
    \ \mathrm{witn}= \{w_i\}_{i \in [1, n]},\\
    &
    \forall i\in[1,n],j\in[1,\ell],\;
    e_{i,j}=\mathsf{H}(W_{i,j}),\\
    &
    \forall i\in[1,n],\;
    r_i=\mathsf{Chain}(e_{i,1},\ldots,e_{i,t}),\\
    &
    {\sf root}_{\sf row}=\mathsf{Chain}(r_1,\ldots,r_n),\\
    &
    \forall j\in[1,\ell],\;
    C_j=\mathsf{Merkle.Com}(e_{1,j}\parallel\cdots\parallel e_{n,j}),\\
    &
    {\sf root}_{\sf col}=\mathsf{Merkle.Com}(C_1\parallel\cdots\parallel C_{\ell}),\\
    &
    {\sf root}=\mathsf{H}({\sf root}_{\sf row}\parallel{\sf root}_{\sf col}),\\
    &
    \forall i\in[1,n],\;
    w_i\in \mathrm{RS}[\mathbb{F},\mathrm{U},\rho]
    \}.
    \end{aligned}
    11. Open the outer Merkle authentication paths for all column roots, i.e., $\{{\sf auth}_j\}_{j \in [1, \ell]} = {\sf Merkle.Open}(C_1, ..., C_{\ell}, {\sf root}_{\sf col})$.
    12. Output ${\sf com} = ({\sf root}_{\sf row},{\sf root}_{\sf col},{\sf root},\pi)$, ${\sf \tau} = \left (\{w_i\}_{i \in [1, n]}, \{{\sf auth}_j\}_{j \in [1, \ell]}\right)$.

- The query algorithm ${\sf V}^{\pi, Q}_1({\sf com}) \rightarrow {\sf tran}$:
    1. Generate the query index set $Q \leftarrow {\sf Sample}(1^{\lambda})$.
    2. Set the transcript ${\sf tran} = (Q, \{W_{1, j}, ..., W_{n, j}, {\sf auth}_j\}_{j \in Q})$.

- The verification algorithm ${\sf V}_2({\sf com}, {\sf tran}) \rightarrow b$:
    1. Verify the STARK proof: Check if ${\sf LeanVM}.{\sf Verify}({{\sf pp}_{\sf STARK}, \sf stmt}, \pi) = 1$.
    2. Verify the openings: Compute $\forall i\in[1,n],j\in Q:e_{i,j}=\mathsf{H}(W_{i,j})$ and $\forall j\in Q:C_j=\mathsf{Merkle.Com}(e_{1,j}\parallel\cdots\parallel e_{n,j})$, then check if ${\sf Merkle}.{\sf Verify}({\sf root}_{\sf col}, \{C_j, {\sf auth}_j\}_{j \in Q}) = 1$.
    3. Check if ${\sf root}=\mathsf{H}({\sf root}_{\sf row}\parallel{\sf root}_{\sf col})$. If all checks pass, output $b = 1$. Otherwise, output $0$.

- The reconstruction algorithm ${\sf Ext}({\sf com}, {\sf tran}_1, ..., {\sf tran}_L) \rightarrow {\sf data}/\bot$:
    1. For $i \in [1, L]$: Parse ${\sf tran}_i = (Q_i, \{W_{1, j}, ..., W_{n, j}, {\sf auth}_j\}_{j \in Q_i})$.
    2. Check if $\forall i \in [1, L]$: ${\sf V}_2({\sf com}, {\sf tran}_i) = 1$. Otherwise return $\bot$.
    3. Find the union set $I$ for all query index sets, i.e., $I = Q_1 \cup Q_2 \cdots \cup Q_L$.
    4. Check if set $I$ has size over the threshold, i.e., $|I| \geq t$. If not return $\bot$.
    5. Reconstruct the data from the codeword symbols contained in the cells indexed by $I$, i.e., ${\sf data} = {\sf Reconst}\left(\{W_{1, j}, ..., W_{n, j}\}_{j \in I}\right)$.

---

## RS Membership Check Instantiations

The condition $w_i\in \mathrm{RS}[\mathbb{F},{\sf U},\rho]$ in the STARK relation can be instantiated by one of the following linear checks.

### 1. Parity-check: $\deg(P_i)<k \Rightarrow c_{i,k}=\cdots=c_{i,m-1}=0$

#### Preprocessing outside the proof:

* Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$, where $\omega$ is a primitive $m$-th root of unity.
* Let $i$ denote the row index, $j$ denote the codeword-symbol index on each row, and $r$ denote the coefficient index of the interpolated polynomial.
* For each row $w_i$, let $P_i(X)=\sum_{r=0}^{m-1}c_{i,r}X^r$ be the polynomial interpolated from its field symbols, where $\forall j \in[0, m-1]: P_i(\omega^j) = w_{i,j}$.
* The coefficients are given by $c_{i,r}=\frac{1}{m}\sum_{j=0}^{m-1} w_{i,j}\omega^{-jr}$.
* Sample $\{\alpha_r\}_{r \in [k,m-1]} \leftarrow \mathsf{H}({\sf pp},{\sf root})$.
* Compute the shared parity-check vector $L=(L_0,\ldots,L_{m-1})$, where $\forall j\in[0,m-1]: L_j=\frac{1}{m}\sum_{r=k}^{m-1}\alpha_r\omega^{-jr}$.

#### Inner product inside the proof:

$$ \begin{aligned} \forall i\in[1,n]:\quad \langle L,w_i\rangle &= \sum_{j=0}^{m-1}L_jw_{i,j} = \sum_{j=0}^{m-1}\left(\frac{1}{m}\sum_{r=k}^{m-1}\alpha_r\omega^{-jr}\right)w_{i,j} \\ &= \sum_{r=k}^{m-1}\alpha_r\left(\frac{1}{m}\sum_{j=0}^{m-1}w_{i,j}\omega^{-jr}\right) = \sum_{r=k}^{m-1}\alpha_rc_{i,r} = 0. \end{aligned} $$

### 2. General barycentric check

#### Preprocessing outside the proof:

* Let ${\sf U} = \{u_0,u_1,\ldots,u_{m-1}\}$.
* Let $i$ denote the row index, $j$ denote the codeword-symbol index on each row, and $s,t$ denote the interpolation and check positions.
* Choose $S\subseteq[0,m-1]$ with $|S|=k$, and let $T=[0,m-1]\setminus S$.
* For each $s\in S$, define the Lagrange basis polynomial $\ell_s(X)$ over ${u_s:s\in S}$, where $\ell_s(u_{s'})=1$ if $s=s'$ and $\ell_s(u_{s'})=0$ otherwise.
* For each row $w_i$, define $P_i(X)=\sum_{s\in S}\ell_s(X)w_{i,s}$.
* Sample $\{\alpha_t\}_{t\in T}\leftarrow\mathsf{H}({\sf pp},{\sf root})$.
* Compute the shared barycentric-check vector $L=(L_0,\ldots,L_{m-1})$, where $\forall t\in T:L_t=\alpha_t$ and $\forall s\in S:L_s=-\sum_{t\in T}\alpha_t\ell_s(u_t)$.

#### Inner product inside the proof:

$$ \begin{aligned} \forall i\in[1,n]:\quad \langle L,w_i\rangle &= \sum_{j=0}^{m-1}L_jw_{i,j} = \sum_{t\in T}L_tw_{i,t}+\sum_{s\in S}L_sw_{i,s} \\ &= \sum_{t\in T}\alpha_tw_{i,t} -\sum_{s\in S}\left(\sum_{t\in T}\alpha_t\ell_s(u_t)\right)w_{i,s} \\ &= \sum_{t\in T}\alpha_t\left(w_{i,t}-\sum_{s\in S}\ell_s(u_t)w_{i,s}\right) = \sum_{t\in T}\alpha_t\left(w_{i,t}-P_i(u_t)\right) = 0. \end{aligned} $$

### 3. Special barycentric check $(\rho = 1/2)$:

#### Preprocessing outside the proof:

* Let ${\sf U}=\{\omega^0,\omega^1,\ldots,\omega^{m-1}\}$, where $\omega$ is a primitive $m$-th root of unity, and assume $m=2k=2h$.
* Let $i$ denote the row index, $j$ denote the codeword-symbol index on each row, and $r$ denote the index on the half-size domain.
* Define $x_r=(\omega^2)^r$ for $r\in[0,h-1]$.
* For each row $w_i$, define $A_i(x_r)=w_{i,2r}$ and $B_i(x_r)=w_{i,2r+1}$.
* Sample $p \leftarrow\mathsf{H}({\sf pp},{\sf root})$ and set $q=p/\omega$.
* Define $\ell_r(z)=\frac{z^h-1}{h}\cdot\frac{x_r}{z-x_r}$.
* Compute the shared barycentric-check vector $L=(L_0,\ldots,L_{m-1})$, where $\forall r\in[0,h-1]:L_{2r}=\ell_r(p)$ and $L_{2r+1}=-\ell_r(q)$.

#### Inner product inside the proof:

$$ \begin{aligned} \forall i\in[1,n]:\quad \langle L,w_i\rangle &= \sum_{j=0}^{m-1}L_jw_{i,j} = \sum_{r=0}^{h-1}L_{2r}w_{i,2r} +\sum_{r=0}^{h-1}L_{2r+1}w_{i,2r+1} \\ &= \sum_{r=0}^{h-1}\ell_r(p)w_{i,2r} -\sum_{r=0}^{h-1}\ell_r(q)w_{i,2r+1} = A_i(p)-B_i(q) \\ &= A_i(p)-B_i(p/\omega) = 0. \end{aligned} $$

---

## Arbitrary-Cell Reconstruction

Let $S\subseteq[0,m-1]$ be the codeword-symbol indices contained in the verified cells and let $E=[0,m-1]\setminus S$ be the erased indices. The same sets $S$ and $E$ are shared by every row.

- Construct the shared erasure-locator polynomial
$$
Z(X)=\prod_{j\in E}(X-\omega^j).
$$
- For each row polynomial $f_i(X)$, define the complete evaluation vector of $N_i(X)=f_i(X)Z(X)$ by
$$
N_i(\omega^j)=
\begin{cases}
w_{i,j}Z(\omega^j), & j\in S,\\
0, & j\in E.
\end{cases}
$$
- Apply a size-$m$ IFFT to recover $N_i(X)$, perform the exact polynomial division $f_i(X)=N_i(X)/Z(X)$ using the shared reversed-polynomial Newton inverse, and evaluate $f_i$ on the size-$k$ systematic domain with a size-$k$ FFT.
- The locator polynomial, its size-$m$ domain evaluations, and its Newton inverse are prepared once and reused for all $n$ rows.

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
| $N_{\sf clients}$ | Total number of client transcripts considered by subset-soundness. |
| $\epsilon$ | Fraction of clients that the adversary attempts to make accept unavailable data. |
| $L_{\sf sub}=\lceil\epsilon N_{\sf clients}\rceil$ | Number of accepting transcripts in the adversarially selected subset. |
| $\Delta=t-1$ | Largest number of served cell columns that remains below the reconstruction threshold. |
| $p_{\sf bad}=\binom{\Delta}{\lvert Q\rvert}/\binom{\ell}{\lvert Q\rvert}$ | Probability that one transcript queries only a fixed non-reconstructing set of $\Delta$ served columns. |
| $\nu_{\sf sub}$ | Conservative subset-soundness bound $\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}p_{\sf bad}^{L_{\sf sub}}$. |
| $\kappa_{\sf samp}$ | Target sampling failure bits, requiring $\nu_{\sf sub}\leq2^{-\kappa_{\sf samp}}$. |
