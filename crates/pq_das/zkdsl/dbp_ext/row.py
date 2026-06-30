from snark_lib import *

DIM = 5
DIGEST_LEN = 8
M_EXT = M_EXT_PLACEHOLDER
C_EXT = C_EXT_PLACEHOLDER
CELL_BASE_LEN = CELL_BASE_LEN_PLACEHOLDER
N_CELLS = N_CELLS_PLACEHOLDER
CHECK_VECTOR_PTR = CHECK_VECTOR_PTR_PLACEHOLDER
PUBLIC_ROW_HASH_PTR = PUBLIC_ROW_HASH_PTR_PLACEHOLDER


@inline
def copy_digest(src, dest):
    for i in unroll(0, DIGEST_LEN):
        dest[i] = src[i]
    return


@inline
def zero_digest(dest):
    for i in unroll(0, DIGEST_LEN):
        dest[i] = 0
    return


@inline
def assert_ext_zero(a):
    for i in unroll(0, DIM):
        assert a[i] == 0
    return


def hash_cell_10_chunks(cell):
    states = Array(8 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 8):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    out = Array(DIGEST_LEN)
    poseidon16_compress_half(states + 7 * DIGEST_LEN, cell + 9 * DIGEST_LEN, out)
    return out


def hash_full_row_cell_digests(cell_digests):
    if N_CELLS == 1:
        out_one = Array(DIGEST_LEN)
        zero = Array(DIGEST_LEN)
        zero_digest(zero)
        poseidon16_compress_half(zero, cell_digests, out_one)
        return out_one
    if N_CELLS == 2:
        out_two = Array(DIGEST_LEN)
        poseidon16_compress_half(cell_digests, cell_digests + DIGEST_LEN, out_two)
        return out_two

    states = Array((N_CELLS - 2) * DIGEST_LEN)
    poseidon16_compress_half(cell_digests, cell_digests + DIGEST_LEN, states)
    for cell in range(1, N_CELLS - 2):
        poseidon16_compress_half(
            states + (cell - 1) * DIGEST_LEN,
            cell_digests + (cell + 1) * DIGEST_LEN,
            states + cell * DIGEST_LEN,
        )
    out_many = Array(DIGEST_LEN)
    poseidon16_compress_half(
        states + (N_CELLS - 3) * DIGEST_LEN,
        cell_digests + (N_CELLS - 1) * DIGEST_LEN,
        out_many,
    )
    return out_many


def main():
    codeword = Array(M_EXT * DIM)
    hint_witness("codeword", codeword)
    public_row_hash = PUBLIC_ROW_HASH_PTR
    check_vector = CHECK_VECTOR_PTR

    cell_digests = Array(N_CELLS * DIGEST_LEN)
    for cell in range(0, N_CELLS):
        digest = hash_cell_10_chunks(codeword + cell * CELL_BASE_LEN)
        copy_digest(digest, cell_digests + cell * DIGEST_LEN)

    row_digest = hash_full_row_cell_digests(cell_digests)
    for i in unroll(0, DIGEST_LEN):
        assert row_digest[i] == public_row_hash[i]

    result = Array(DIM)
    dot_product_ee(codeword, check_vector, result, M_EXT)
    assert_ext_zero(result)
    return
