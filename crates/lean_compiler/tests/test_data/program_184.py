from snark_lib import *

"""
Exercises ALL 7 poseidon16 precompile variants and cross-checks them against
each other, so that a regression in the compiler's variant -> flag lowering
(permute / half_output / hardcoded_offset_left, cf. a_simplify_lang) breaks
this program. It also guarantees every variant appears in the test_data corpus
for differential bytecode comparisons.

Semantics (zkDSL.md, "Poseidon16 family"); P = raw Poseidon2 permutation:
  input = left(8 cells) || right(8 cells)
  permute family:  output = P(input)           (16 cells; low 8 for `_half`)
  compress family: output = P(input) + input   (low 8 cells; low 4 for `_quarter`)
  `_hardcoded_left(L, R, O, off)`: left = m[off..off+4] || m[L..L+4]
Cells past a variant's output length are NOT written; this is checked below by
writing sentinels afterwards (the write-once rule would reject a conflict).

Limitation (verified by mutation testing): a variant writing FEWER cells than
expected is NOT caught at execution, because the VM's memory solver back-fills
unwritten cells from the asserts (the mechanism powering range checks). The
guard for that direction is differential bytecode comparison across compiler
changes -- which this program enables by putting all 7 variants in test_data.
"""

HARD_PTR = 8  # 4 preamble cells, right after the 8-cell public input region


def main():
    # Known, pairwise-distinct values at the compile-time address HARD_PTR,
    # so a corrupted `off` would change the hardcoded-left results below.
    hard = HARD_PTR
    hard[0] = 101
    hard[1] = 102
    hard[2] = 103
    hard[3] = 104

    left = Array(8)
    right = Array(8)
    for i in unroll(0, 8):
        left[i] = 10 + i
        right[i] = 20 + i

    # The effective left input of the hardcoded-left variants, materialized.
    left_hard = Array(8)
    for i in unroll(0, 4):
        left_hard[i] = hard[i]
        left_hard[i + 4] = left[i]

    # permute vs permute_half: same low 8 cells; permute_half writes ONLY 8.
    out_p = Array(16)
    out_ph = Array(16)
    poseidon16_permute(left, right, out_p)
    poseidon16_permute_half(left, right, out_ph)
    for i in unroll(0, 8):
        assert out_ph[i] == out_p[i]
    for i in unroll(8, 16):
        out_ph[i] = 0  # would conflict if permute_half had written 16 cells

    # compress_half = permutation + feed-forward of the left half.
    out_ch = Array(8)
    poseidon16_compress_half(left, right, out_ch)
    for i in unroll(0, 8):
        assert out_ch[i] == out_p[i] + left[i]

    # compress_quarter: same low 4 cells as compress_half, writes ONLY 4.
    out_cq = Array(8)
    poseidon16_compress_quarter(left, right, out_cq)
    for i in unroll(0, 4):
        assert out_cq[i] == out_ch[i]
        out_cq[i + 4] = 0  # would conflict if compress_quarter had written 8 cells

    # hardcoded-left variants == plain variants applied to `left_hard`.
    out_chl = Array(8)
    out_ch2 = Array(8)
    poseidon16_compress_half_hardcoded_left(left, right, out_chl, HARD_PTR)
    poseidon16_compress_half(left_hard, right, out_ch2)
    for i in unroll(0, 8):
        assert out_chl[i] == out_ch2[i]

    out_cql = Array(8)
    poseidon16_compress_quarter_hardcoded_left(left, right, out_cql, HARD_PTR)
    for i in unroll(0, 4):
        assert out_cql[i] == out_ch2[i]
        out_cql[i + 4] = 0  # would conflict on an 8-cell write

    out_phl = Array(8)
    out_ph2 = Array(8)
    poseidon16_permute_half_hardcoded_left(left, right, out_phl, HARD_PTR)
    poseidon16_permute_half(left_hard, right, out_ph2)
    for i in unroll(0, 8):
        assert out_phl[i] == out_ph2[i]

    return
