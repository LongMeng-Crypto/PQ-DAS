from snark_lib import *

DIM = 5
DIGEST_LEN = 8
N = N_PLACEHOLDER
N_PADDED = N_PADDED_PLACEHOLDER
LOG_N_PADDED = LOG_N_PADDED_PLACEHOLDER
M = M_PLACEHOLDER
K = K_PLACEHOLDER
C = C_PLACEHOLDER
N_CELLS = N_CELLS_PLACEHOLDER
SYSTEMATIC_CELLS = SYSTEMATIC_CELLS_PLACEHOLDER
SYSTEMATIC_STRIDE = SYSTEMATIC_STRIDE_PLACEHOLDER
ROW_CHUNKS = ROW_CHUNKS_PLACEHOLDER
CELL_CHUNKS = CELL_CHUNKS_PLACEHOLDER
OUTER_MERKLE_DEPTH = OUTER_MERKLE_DEPTH_PLACEHOLDER
OUTER_TREE_DIGESTS = OUTER_TREE_DIGESTS_PLACEHOLDER
OUTER_LEVEL_SIZES = OUTER_LEVEL_SIZES_PLACEHOLDER
OUTER_LEVEL_OFFSETS = OUTER_LEVEL_OFFSETS_PLACEHOLDER

PUBLIC_ROOT_PTR = PUBLIC_ROOT_PTR_PLACEHOLDER
CHECK_VECTOR_PTR = CHECK_VECTOR_PTR_PLACEHOLDER


@inline
# Copies one eight-field digest.
def copy_digest(src, dest):
    for i in unroll(0, DIGEST_LEN):
        dest[i] = src[i]
    return


@inline
# Sets one eight-field digest to zero for padded column-Merkle rows.
def zero_digest(dest):
    for i in unroll(0, DIGEST_LEN):
        dest[i] = 0
    return


@inline
# Constrains one extension element to be zero in all five coordinates.
def assert_ext_zero(a):
    for i in unroll(0, DIM):
        assert a[i] == 0
    return


# Builds one zero digest.
def zero_digest_ret():
    zero = Array(DIGEST_LEN)
    zero_digest(zero)
    return zero


# Hashes contiguous rate-eight blocks as a fixed Poseidon16 compression chain.
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


# Hashes the base-field cell size c=64, i.e. 8 rate-eight chunks.
def hash_cell_8_chunks(cell):
    states = Array(6 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 6):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    out = Array(DIGEST_LEN)
    poseidon16_compress_half(
        states + 5 * DIGEST_LEN,
        cell + 7 * DIGEST_LEN,
        out,
    )
    return out


# Hashes the doubled base-field cell size c=128, i.e. 16 rate-eight chunks.
def hash_cell_16_chunks(cell):
    states = Array(14 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 14):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    out = Array(DIGEST_LEN)
    poseidon16_compress_half(
        states + 13 * DIGEST_LEN,
        cell + 15 * DIGEST_LEN,
        out,
    )
    return out


# Keeps small test profiles generic while specializing production V3 base profiles.
def hash_cell(cell):
    if CELL_CHUNKS == 8:
        return hash_cell_8_chunks(cell)
    if CELL_CHUNKS == 16:
        return hash_cell_16_chunks(cell)
    return hash_contiguous_chunks(cell, CELL_CHUNKS)


# Builds a complete binary Merkle root over an in-memory digest array.
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


# Hashes the first t systematic cell digests for one row.
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


# Proves V2's cell-first commitment, public row digests, column root, and RS checks.
def main():
    codewords = Array(N * M)
    hint_witness("codewords", codewords)
    public_root = PUBLIC_ROOT_PTR
    check_vector = CHECK_VECTOR_PTR

    # Cell digests are stored column-major so every column's Merkle leaves are contiguous.
    cell_digests = Array(N_CELLS * N_PADDED * DIGEST_LEN)

    for row in range(0, N):
        row_base = codewords + row * M
        for cell in range(0, N_CELLS):
            digest = hash_cell(row_base + cell * C)
            copy_digest(digest, cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    row_hashes = Array(N_PADDED * DIGEST_LEN)
    for row in range(0, N):
        row_digest = hash_systematic_cell_digests(cell_digests, row)
        copy_digest(row_digest, row_hashes + row * DIGEST_LEN)
    for row in unroll(N, N_PADDED):
        zero_digest(row_hashes + row * DIGEST_LEN)
    root_row = merkle_root_from_digests(row_hashes, LOG_N_PADDED)

    for cell in range(0, N_CELLS):
        for row in unroll(N, N_PADDED):
            zero_digest(cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    column_roots = Array(N_CELLS * DIGEST_LEN)
    for cell in range(0, N_CELLS):
        root = merkle_root_from_digests(cell_digests + cell * N_PADDED * DIGEST_LEN, LOG_N_PADDED)
        copy_digest(root, column_roots + cell * DIGEST_LEN)

    root_col = merkle_root_from_digests(column_roots, OUTER_MERKLE_DEPTH)
    root = Array(DIGEST_LEN)
    poseidon16_compress_half(root_row, root_col, root)
    for i in unroll(0, DIGEST_LEN):
        assert root[i] == public_root[i]

    for row in range(0, N):
        result = Array(DIM)
        dot_product_be(codewords + row * M, check_vector, result, M)
        assert_ext_zero(result)
    return
