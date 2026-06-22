---
title: PQ-DAS V3 Design
---

# PQ-DAS V3 Design

## Purpose

This document defines a distributed PQ-DAS construction in which independent provers prove individual blobs and recursive merge provers combine adjacent row segments into one column-openable matrix commitment.

## Setup

- The Setup algorithm $\mathsf{Setup}(1^\lambda)\rightarrow{\sf pp}$:
    1. Choose domain-separated hash functions $\mathsf{H}_{\sf cell}$, $\mathsf{H}_{\sf row}$, $\mathsf{H}_{\sf col}$, $\mathsf{H}_{\sf outer}$, and $\mathsf{H}_{\sf state}$.
    2. Define the Reed-Solomon code ${\sf RS}[\mathbb{F},{\sf U},\rho]$ and its encoding algorithm $\mathcal{C}:\mathbb{F}^k\rightarrow\mathbb{F}^m$, where $k=\rho m$.
    3. Define the cell size $c$, the number of cells $\ell=m/c$, and the reconstruction threshold $t=\lceil k/c\rceil$.
    4. Define the LeanVM leaf program ${\sf P}_{\sf leaf}$, merge program ${\sf P}_{\sf merge}$, recursive verifier, and public parameters $\mathsf{pp}_{\sf STARK}$.
    5. Define a public context ${\sf ctx}$ that binds the protocol version, RS parameters, cell size, block or slot identifier, and row-ordering rule.
    6. Output ${\sf pp}=(\mathsf{H}_{\sf cell},\mathsf{H}_{\sf row},\mathsf{H}_{\sf col},\mathsf{H}_{\sf outer},\mathsf{H}_{\sf state},\mathbb{F},{\sf U},m,k,\rho,c,\ell,t,\mathsf{pp}_{\sf STARK})$.

## Segment Commitment

For every non-empty row interval $I=[a,b]\subseteq[1,n]$, define:

- The row-segment root $R^{I}_{\sf row}$.
- The column-segment vector
$$
\mathbf{C}^{I}=(C^{I}_1,\ldots,C^{I}_{\ell}).
$$
- The commitment to the column-segment vector
$$
R^{I}_{\sf col}=\mathsf{Merkle.Com}_{\sf outer}(C^{I}_1,\ldots,C^{I}_{\ell}).
$$
- The segment-state digest
$$
D_I=\mathsf{H}_{\sf state}({\sf ctx}\parallel a\parallel b\parallel R^{I}_{\sf row}\parallel R^{I}_{\sf col}).
$$
- The public segment state
$$
\sigma_I=({\sf ctx},a,b,R^{I}_{\sf row},R^{I}_{\sf col},D_I).
$$
- The segment bundle
$$
\mathcal{B}_I=(\sigma_I,\pi_I,\mathbf{C}^{I}),
$$
where $\pi_I$ is a proof with public input $D_I$. The vector $\mathbf{C}^{I}$ is not part of the final public commitment, but it is bound by $R^{I}_{\sf col}$ and passed to the next merge prover.

## Canonical Aggregation Tree

The ordered rows use one canonical binary aggregation tree $\mathcal{T}_{a:b}$:

- If $a=b$, $\mathcal{T}_{a:b}$ is a leaf.
- If $a<b$, let $s=b-a+1$ and let $p$ be the largest power of two strictly smaller than $s$. The left child is $[a,a+p-1]$ and the right child is $[a+p,b]$.

This rule supports arbitrary $n$, fixes the row order, and avoids dummy codewords. For power-of-two sizes it gives the usual balanced binary tree.

## Leaf Proving

- The leaf algorithm ${\sf LeafProve}({\sf pp},{\sf ctx},i,b_i)\rightarrow\mathcal{B}_{[i,i]}$:
    1. RS encode the blob:
    $$
    w_i=\mathcal{C}(b_i)=(w_{i,1},\ldots,w_{i,m})\in\mathbb{F}^m.
    $$
    2. Group every $c$ symbols into a cell:
    $$
    W_{i,j}=(w_{i,(j-1)c+1},\ldots,w_{i,jc})\in\mathbb{F}^c.
    $$
    3. Hash every cell exactly once:
    $$
    e_{i,j}=\mathsf{H}_{\sf cell}({\sf ctx}\parallel i\parallel j\parallel W_{i,j}).
    $$
    4. Hash-chain the systematic cell digests:
    $$
    r_i=\mathsf{Chain}_{\sf row}(e_{i,1},\ldots,e_{i,t}).
    $$
    5. Define the leaf column-segment vector:
    $$
    \forall j\in[1,\ell]:\quad C^{[i,i]}_j=e_{i,j}.
    $$
    6. Compute
    $$
    R^{[i,i]}_{\sf row}=r_i,
    \qquad
    R^{[i,i]}_{\sf col}=\mathsf{Merkle.Com}_{\sf outer}(e_{i,1},\ldots,e_{i,\ell}),
    $$
    and
    $$
    D_{[i,i]}=\mathsf{H}_{\sf state}({\sf ctx}\parallel i\parallel i\parallel r_i\parallel R^{[i,i]}_{\sf col}).
    $$
    7. Generate the leaf proof
    $$
    \pi_{[i,i]}\leftarrow{\sf LeanVM.Prove}({\sf P}_{\sf leaf},D_{[i,i]},(\sigma_{[i,i]},w_i,\{e_{i,j}\}_{j=1}^{\ell})),
    $$
    for the relation
    \begin{aligned}
    \mathcal{R}_{\sf leaf}=\{(D_{[i,i]},\mathsf{witn}):\;&
    w_i\in{\sf RS}[\mathbb{F},{\sf U},\rho],\\
    &\forall j,\ e_{i,j}=\mathsf{H}_{\sf cell}({\sf ctx}\parallel i\parallel j\parallel W_{i,j}),\\
    &r_i=\mathsf{Chain}_{\sf row}(e_{i,1},\ldots,e_{i,t}),\\
    &R^{[i,i]}_{\sf col}=\mathsf{Merkle.Com}_{\sf outer}(e_{i,1},\ldots,e_{i,\ell}),\\
    &D_{[i,i]}=\mathsf{H}_{\sf state}({\sf ctx}\parallel i\parallel i\parallel r_i\parallel R^{[i,i]}_{\sf col})\}.
    \end{aligned}
    8. Output
    $$
    \mathcal{B}_{[i,i]}=(\sigma_{[i,i]},\pi_{[i,i]},(e_{i,1},\ldots,e_{i,\ell})).
    $$
    9. Disseminate the encoded cells $\{W_{i,j}\}_{j=1}^{\ell}$ to the corresponding column custodians independently of the proof bundle.

## Merge Proving

Let $A=[a,u]$ and $B=[u+1,b]$ be the canonical left and right children of $I=[a,b]$.

- The merge algorithm ${\sf Merge}({\sf pp},\mathcal{B}_A,\mathcal{B}_B)\rightarrow\mathcal{B}_I$:
    1. Parse
    $$
    \mathcal{B}_A=(\sigma_A,\pi_A,\mathbf{C}^A),
    \qquad
    \mathcal{B}_B=(\sigma_B,\pi_B,\mathbf{C}^B).
    $$
    2. Check that both children use the same ${\sf ctx}$, their ranges are adjacent, and $(A,B)$ is the canonical split of $I$.
    3. Recompute the child state digests from $\sigma_A$ and $\sigma_B$.
    4. Recursively verify $\pi_A$ against $D_A$ and $\pi_B$ against $D_B$. The recursive verifier accepts ${\sf P}_{\sf leaf}$ for leaf children and ${\sf P}_{\sf merge}$ for internal children.
    5. Bind the supplied child column vectors to the child public states:
    $$
    \mathsf{Merkle.Com}_{\sf outer}(\mathbf{C}^A)=R^A_{\sf col},
    \qquad
    \mathsf{Merkle.Com}_{\sf outer}(\mathbf{C}^B)=R^B_{\sf col}.
    $$
    6. Merge every column root:
    $$
    \forall j\in[1,\ell]:\quad
    C^I_j=\mathsf{H}_{\sf col}({\sf ctx}\parallel A\parallel B\parallel j\parallel C^A_j\parallel C^B_j).
    $$
    7. Merge the row roots:
    $$
    R^I_{\sf row}=\mathsf{H}_{\sf row}({\sf ctx}\parallel A\parallel B\parallel R^A_{\sf row}\parallel R^B_{\sf row}).
    $$
    8. Commit the parent column vector and compute the parent state digest:
    $$
    R^I_{\sf col}=\mathsf{Merkle.Com}_{\sf outer}(C^I_1,\ldots,C^I_{\ell}),
    $$
    $$
    D_I=\mathsf{H}_{\sf state}({\sf ctx}\parallel a\parallel b\parallel R^I_{\sf row}\parallel R^I_{\sf col}).
    $$
    9. Generate the merge proof
    $$
    \pi_I\leftarrow{\sf LeanVM.Prove}({\sf P}_{\sf merge},D_I,(\sigma_A,\pi_A,\mathbf{C}^A,\sigma_B,\pi_B,\mathbf{C}^B)),
    $$
    proving all checks in Steps 2--8.
    10. Output
    $$
    \mathcal{B}_I=(\sigma_I,\pi_I,(C^I_1,\ldots,C^I_{\ell})).
    $$

The merge prover never receives $w_a,\ldots,w_b$ and never repeats RS membership or cell hashing. It receives only two child proofs, two constant-size child states, and two vectors of $\ell$ digests.

The recursive verifier binds the accepted inner-program hash. A height-zero child must verify under ${\sf P}_{\sf leaf}$, while an internal child must verify under ${\sf P}_{\sf merge}$; a proof for any other LeanVM program is rejected.

### Four-row example

For four leaf rows and every column $j$:

$$
C^{[1,2]}_j=\mathsf{H}_{\sf col}(C^{[1,1]}_j\parallel C^{[2,2]}_j)
=\mathsf{H}_{\sf col}(e_{1,j}\parallel e_{2,j}),
$$

$$
C^{[3,4]}_j=\mathsf{H}_{\sf col}(e_{3,j}\parallel e_{4,j}),
$$

$$
C^{[1,4]}_j=\mathsf{H}_{\sf col}(C^{[1,2]}_j\parallel C^{[3,4]}_j).
$$

Thus $C^{[1,4]}_j$ is exactly the Merkle root of the four cell digests in column $j$. The final outer commitment is

$$
R^{[1,4]}_{\sf col}=\mathsf{Merkle.Com}_{\sf outer}(C^{[1,4]}_1,\ldots,C^{[1,4]}_{\ell}).
$$

The formulas above suppress the public context, ranges, and column index inside each domain-separated $\mathsf{H}_{\sf col}$ call.

## Distributed Aggregation

- The distributed aggregation algorithm ${\sf Aggregate}(\mathcal{B}_{[1,1]},\ldots,\mathcal{B}_{[n,n]})\rightarrow\mathcal{B}_{[1,n]}$:
    1. The coordinator fixes ${\sf ctx}$ and assigns each blob its final row index $i$. Reordering a blob or moving it to another context invalidates its leaf proof.
    2. Generate all leaf proofs independently and in parallel.
    3. Whenever the two canonical children of a segment are available, assign their merge to any merge prover.
    4. Run all independent merges at the same tree level in parallel.
    5. Continue until the root bundle $\mathcal{B}_{[1,n]}$ is produced.

The total number of merge proofs is $n-1$. For power-of-two $n$, the proving critical path contains $\log_2n$ merge levels.

For blobs arriving sequentially, maintain a forest of completed power-of-two segments. Insert each new leaf and repeatedly merge the two rightmost equal-size adjacent segments. A checkpoint for the first $s$ blobs recursively combines the remaining forest roots according to $\mathcal{T}_{1:s}$.

## Final Commitment

Parse the root bundle as
$$
\mathcal{B}_{[1,n]}=(\sigma_{[1,n]},\pi_{[1,n]},\mathbf{C}^{[1,n]}).
$$

- The final commitment is
$$
{\sf com}=({\sf ctx},n,R^{[1,n]}_{\sf row},R^{[1,n]}_{\sf col},D_{[1,n]},\pi_{[1,n]}).
$$
- The auxiliary opening data is
$$
{\sf \tau}=\left(\{W_{i,j}\}_{i\in[1,n],j\in[1,\ell]},\{{\sf auth}_j\}_{j\in[1,\ell]}\right),
$$
where ${\sf auth}_j$ authenticates $C^{[1,n]}_j$ in the outer Merkle tree with root $R^{[1,n]}_{\sf col}$.

The final verifier checks that $D_{[1,n]}$ is the hash of the public root state and verifies only the single recursive proof $\pi_{[1,n]}$.

## Column Query and Verification

- The query algorithm ${\sf V}^{\pi,Q}_1({\sf com})\rightarrow{\sf tran}$:
    1. Sample the column index set $Q\leftarrow{\sf Sample}(1^\lambda)$.
    2. Return
    $$
    {\sf tran}=(Q,\{W_{1,j},\ldots,W_{n,j},{\sf auth}_j\}_{j\in Q}).
    $$

- The verification algorithm ${\sf V}_2({\sf com},{\sf tran})\rightarrow b$:
    1. Verify $\pi_{[1,n]}$ against public input $D_{[1,n]}$.
    2. Check that $D_{[1,n]}$ binds $({\sf ctx},1,n,R^{[1,n]}_{\sf row},R^{[1,n]}_{\sf col})$.
    3. For every $j\in Q$, compute
    $$
    e_{i,j}=\mathsf{H}_{\sf cell}({\sf ctx}\parallel i\parallel j\parallel W_{i,j})
    $$
    for all $i\in[1,n]$.
    4. Recompute $C^{[1,n]}_j$ by applying the canonical tree $\mathcal{T}_{1:n}$ and $\mathsf{H}_{\sf col}$ to $(e_{1,j},\ldots,e_{n,j})$.
    5. Verify ${\sf auth}_j$ from $C^{[1,n]}_j$ to $R^{[1,n]}_{\sf col}$.
    6. Output $1$ if all checks pass; otherwise output $0$.

Each sampled column contains all $n$ cells and one outer Merkle path. No independent row authentication path is required.

## Reconstruction

- The reconstruction algorithm ${\sf Ext}({\sf com},{\sf tran}_1,\ldots,{\sf tran}_L)\rightarrow{\sf data}/\bot$:
    1. Verify every transcript with ${\sf V}_2$.
    2. Form the union $I=Q_1\cup\cdots\cup Q_L$ and return $\bot$ if $|I|<t$.
    3. Let $S$ be the codeword-symbol indices contained in the cells indexed by $I$, and let $E=[0,m-1]\setminus S$.
    4. Construct the shared erasure locator
    $$
    Z(X)=\prod_{j\in E}(X-\omega^j).
    $$
    5. For each row polynomial $f_i(X)$, construct
    $$
    N_i(\omega^j)=
    \begin{cases}
    w_{i,j}Z(\omega^j),&j\in S,\\
    0,&j\in E.
    \end{cases}
    $$
    6. Apply a size-$m$ IFFT to recover $N_i(X)$, divide exactly by the shared $Z(X)$, and evaluate $f_i$ on the systematic domain with a size-$k$ FFT.
    7. Output ${\sf data}=(b_1,\ldots,b_n)$.

## Leaf RS Membership

For the current rate-$1/2$ instantiation, let $m=2k=2h$, $x_r=(\omega^2)^r$, $A_i(x_r)=w_{i,2r}$, and $B_i(x_r)=w_{i,2r+1}$. Derive $p_i$ from $({\sf ctx},i,D_{[i,i]})$, set $q_i=p_i/\omega$, and define
$$
L^{(i)}_{2r}=\ell_r(p_i),
\qquad
L^{(i)}_{2r+1}=-\ell_r(q_i),
$$
where
$$
\ell_r(z)=\frac{z^h-1}{h}\cdot\frac{x_r}{z-x_r}.
$$
The leaf proof checks
$$
\langle L^{(i)},w_i\rangle=A_i(p_i)-B_i(p_i/\omega)=0.
$$

Each leaf has its own Fiat-Shamir challenge because leaf proofs are generated independently. Merge proofs do not repeat this check; recursive verification carries it to the final proof.

## Pipelined Dissemination

At a checkpoint covering the first $s$ blobs, broadcast:

$$
(D_{[1,s]},\pi_{[1,s]},\text{new sampled-column cells since the previous checkpoint}).
$$

Peers cache previously received column cells and only download the new row fragments. Repeating checkpoint proofs increases total proof bytes but spreads proof verification and column traffic across the slot.

## Security Invariant

By induction over $\mathcal{T}_{1:n}$:

- A valid leaf proof binds one RS-valid codeword to its complete cell-digest vector.
- A valid merge proof verifies both child proofs, binds both supplied child vectors to the child public roots, and computes each parent column root from the corresponding child roots.
- Therefore, the final proof binds every RS-valid row to the same cell digests used by the final column commitment.
- Changing any final opened cell without changing $D_{[1,n]}$ requires breaking leaf-proof soundness, recursive-proof soundness, or hash binding.

## Cost Summary

| Stage | Proved work | Data received by prover |
|---|---|---|
| Leaf | One RS membership check, all cell hashes, systematic row digest, one outer vector root | One blob/codeword |
| Merge | Two recursive verifications, two child-vector root checks, $\ell$ column merges, one parent vector root | Two child proofs and two $\ell$-digest vectors |
| Final verifier | One recursive proof and queried column openings | Constant-size commitment and sampled columns |

The baseline merge performs approximately $2(\ell-1)+\ell+(\ell-1)$ hash compressions for child-vector binding, per-column merging, and parent-vector commitment, in addition to two recursive proof verifications.

## Parameter Spreadsheet

| Parameter | Definition |
|---|---|
| $\lambda$ | Security parameter. |
| $\mathbb{F}$ | Finite field used for RS encoding and STARK arithmetic. |
| ${\sf U}$ | RS evaluation domain. |
| $\mathsf{H}_{\sf cell},\mathsf{H}_{\sf row},\mathsf{H}_{\sf col},\mathsf{H}_{\sf outer},\mathsf{H}_{\sf state}$ | Domain-separated hash functions. |
| $\mathsf{pp}_{\sf STARK}$ | Public parameters for leaf, merge, and recursive proof verification. |
| ${\sf ctx}$ | Public context binding protocol parameters, block or slot, and ordered rows. |
| $n$ | Number of actual blob rows. |
| $m$ | Number of symbols in one RS codeword. |
| $k$ | Number of input symbols in one blob. |
| $\rho=k/m$ | RS code rate. |
| $c$ | Number of field elements in one cell. |
| $\ell=m/c$ | Number of cells and column roots. |
| $t=\lceil k/c\rceil$ | Reconstruction threshold in distinct columns. |
| $I=[a,b]$ | Contiguous ordered row segment. |
| $\mathbf{C}^{I}$ | Vector of $\ell$ column roots for segment $I$. |
| $R^{I}_{\sf row}$ | Ordered row-commitment root for segment $I$. |
| $R^{I}_{\sf col}$ | Merkle commitment to $\mathbf{C}^{I}$. |
| $D_I$ | Public digest of the complete segment state. |
| $\pi_I$ | Leaf or recursive merge proof for public input $D_I$. |
| $\mathcal{B}_I$ | Segment bundle $(\sigma_I,\pi_I,\mathbf{C}^{I})$. |
| $Q$ | Queried column-index set in one transcript. |
| $\lvert Q\rvert$ | Number of distinct sampled columns in one transcript. |
| $L$ | Number of accepted transcripts used for reconstruction. |
| $N_{\sf clients}$ | Total client population used by subset-soundness. |
| $\epsilon$ | Fraction of clients the adversary attempts to fool. |
| $L_{\sf sub}=\lceil\epsilon N_{\sf clients}\rceil$ | Size of the adversarially selected accepting subset. |
| $\Delta=t-1$ | Largest number of served columns below the reconstruction threshold. |
| $p_{\sf bad}=\binom{\Delta}{\lvert Q\rvert}/\binom{\ell}{\lvert Q\rvert}$ | Probability that one transcript stays inside one fixed non-reconstructing set. |
| $\nu_{\sf sub}$ | Bound $\binom{\ell}{\Delta}\binom{N_{\sf clients}}{L_{\sf sub}}p_{\sf bad}^{L_{\sf sub}}$. |
| $\kappa_{\sf samp}$ | Target bits requiring $\nu_{\sf sub}\leq2^{-\kappa_{\sf samp}}$. |
