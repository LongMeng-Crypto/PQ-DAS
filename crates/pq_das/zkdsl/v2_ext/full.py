from snark_lib import *

DIM = 5
DIGEST_LEN = 8
N = N_PLACEHOLDER
N_PADDED = N_PADDED_PLACEHOLDER
LOG_N_PADDED = LOG_N_PADDED_PLACEHOLDER
M_EXT = M_EXT_PLACEHOLDER
K_EXT = K_EXT_PLACEHOLDER
C_EXT = C_EXT_PLACEHOLDER
CELL_BASE_LEN = CELL_BASE_LEN_PLACEHOLDER
N_CELLS = N_CELLS_PLACEHOLDER
SYSTEMATIC_CELLS = SYSTEMATIC_CELLS_PLACEHOLDER
CELL_CHUNKS = CELL_CHUNKS_PLACEHOLDER
OUTER_MERKLE_DEPTH = OUTER_MERKLE_DEPTH_PLACEHOLDER
OUTER_TREE_DIGESTS = OUTER_TREE_DIGESTS_PLACEHOLDER
OUTER_LEVEL_SIZES = OUTER_LEVEL_SIZES_PLACEHOLDER
OUTER_LEVEL_OFFSETS = OUTER_LEVEL_OFFSETS_PLACEHOLDER

PUBLIC_ROW_HASHES_PTR = PUBLIC_ROW_HASHES_PTR_PLACEHOLDER
PUBLIC_ROOT_COL_PTR = PUBLIC_ROOT_COL_PTR_PLACEHOLDER
CHECK_VECTOR_PTR = CHECK_VECTOR_PTR_PLACEHOLDER


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


def zero_digest_ret():
    zero = Array(DIGEST_LEN)
    zero_digest(zero)
    return zero


def hash_contiguous_chunks(data, num_chunks: Const):
    if num_chunks == 1:
        out_one = Array(DIGEST_LEN)
        poseidon16_compress_half(zero_digest_ret(), data, out_one)
        return out_one
    if num_chunks == 2:
        out_two = Array(DIGEST_LEN)
        poseidon16_compress_half(data, data + DIGEST_LEN, out_two)
        return out_two

    states = Array((num_chunks - 2) * DIGEST_LEN)
    poseidon16_compress_half(data, data + DIGEST_LEN, states)
    for chunk in range(1, num_chunks - 2):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            data + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    out_many = Array(DIGEST_LEN)
    poseidon16_compress_half(
        states + (num_chunks - 3) * DIGEST_LEN,
        data + (num_chunks - 1) * DIGEST_LEN,
        out_many,
    )
    return out_many


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
    poseidon16_compress_half(
        states + 7 * DIGEST_LEN,
        cell + 9 * DIGEST_LEN,
        out,
    )
    return out


def merkle_root_from_digests(leaves, log_num_leaves: Const):
    layer: Mut = leaves
    for level in unroll(1, log_num_leaves + 1):
        layer_size = 2 ** (log_num_leaves - level)
        new_layer = Array(layer_size * DIGEST_LEN)
        for node in unroll(0, layer_size):
            poseidon16_compress_half(
                layer + (2 * node) * DIGEST_LEN,
                layer + (2 * node + 1) * DIGEST_LEN,
                new_layer + node * DIGEST_LEN,
            )
        layer = new_layer
    return layer


def hash_systematic_cell_digests(cell_digests, row):
    if SYSTEMATIC_CELLS == 1:
        out_one = Array(DIGEST_LEN)
        poseidon16_compress_half(zero_digest_ret(), cell_digests + row * DIGEST_LEN, out_one)
        return out_one
    if SYSTEMATIC_CELLS == 2:
        out_two = Array(DIGEST_LEN)
        poseidon16_compress_half(
            cell_digests + row * DIGEST_LEN,
            cell_digests + (N_PADDED + row) * DIGEST_LEN,
            out_two,
        )
        return out_two

    states = Array((SYSTEMATIC_CELLS - 2) * DIGEST_LEN)
    poseidon16_compress_half(
        cell_digests + row * DIGEST_LEN,
        cell_digests + (N_PADDED + row) * DIGEST_LEN,
        states,
    )
    for cell in range(1, SYSTEMATIC_CELLS - 2):
        poseidon16_compress_half(
            states + (cell - 1) * DIGEST_LEN,
            cell_digests + ((cell + 1) * N_PADDED + row) * DIGEST_LEN,
            states + cell * DIGEST_LEN,
        )
    out_many = Array(DIGEST_LEN)
    poseidon16_compress_half(
        states + (SYSTEMATIC_CELLS - 3) * DIGEST_LEN,
        cell_digests + ((SYSTEMATIC_CELLS - 1) * N_PADDED + row) * DIGEST_LEN,
        out_many,
    )
    return out_many


def main():
    codewords = Array(N * M_EXT * DIM)
    hint_witness("codewords", codewords)
    public_row_hashes = PUBLIC_ROW_HASHES_PTR
    public_root_col = PUBLIC_ROOT_COL_PTR
    check_vector = CHECK_VECTOR_PTR

    cell_digests = Array(N_CELLS * N_PADDED * DIGEST_LEN)

    for row in range(0, N):
        row_base = codewords + row * M_EXT * DIM
        for cell in range(0, N_CELLS):
            digest = hash_cell_10_chunks(row_base + cell * CELL_BASE_LEN)
            copy_digest(digest, cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    for row in range(0, N):
        row_digest = hash_systematic_cell_digests(cell_digests, row)
        for i in unroll(0, DIGEST_LEN):
            assert row_digest[i] == public_row_hashes[row * DIGEST_LEN + i]

    for cell in range(0, N_CELLS):
        for row in unroll(N, N_PADDED):
            zero_digest(cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    column_roots = Array(N_CELLS * DIGEST_LEN)
    for cell in range(0, N_CELLS):
        root = merkle_root_from_digests(cell_digests + cell * N_PADDED * DIGEST_LEN, LOG_N_PADDED)
        copy_digest(root, column_roots + cell * DIGEST_LEN)

    outer_root = merkle_root_from_digests(column_roots, OUTER_MERKLE_DEPTH)
    for i in unroll(0, DIGEST_LEN):
        assert outer_root[i] == public_root_col[i]

    for row in range(0, N):
        result = Array(DIM)
        dot_product_ee(codewords + row * M_EXT * DIM, check_vector, result, M_EXT)
        assert_ext_zero(result)
    return
