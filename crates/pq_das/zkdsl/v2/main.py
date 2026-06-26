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
SYSTEMATIC_STRIDE = SYSTEMATIC_STRIDE_PLACEHOLDER
ROW_CHUNKS = ROW_CHUNKS_PLACEHOLDER
CELL_CHUNKS = CELL_CHUNKS_PLACEHOLDER
OUTER_MERKLE_DEPTH = OUTER_MERKLE_DEPTH_PLACEHOLDER
OUTER_TREE_DIGESTS = OUTER_TREE_DIGESTS_PLACEHOLDER
OUTER_LEVEL_SIZES = OUTER_LEVEL_SIZES_PLACEHOLDER
OUTER_LEVEL_OFFSETS = OUTER_LEVEL_OFFSETS_PLACEHOLDER

PUBLIC_ROW_HASHES_PTR = PUBLIC_ROW_HASHES_PTR_PLACEHOLDER
PUBLIC_ROOT_COL_PTR = PUBLIC_ROOT_COL_PTR_PLACEHOLDER
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


# Proves V2's cell-first commitment, public row digests, column root, and RS checks.
def main():
    codewords = Array(N * M)
    hint_witness("codewords", codewords)
    public_row_hashes = PUBLIC_ROW_HASHES_PTR
    public_root_col = PUBLIC_ROOT_COL_PTR
    check_vector = CHECK_VECTOR_PTR

    # Cell digests are stored column-major so every column's Merkle leaves are contiguous.
    cell_digests = Array(N_CELLS * N_PADDED * DIGEST_LEN)

    for row in range(0, N):
        row_base = codewords + row * M
        row_digest = hash_contiguous_chunks(row_base, ROW_CHUNKS)
        for i in unroll(0, DIGEST_LEN):
            assert row_digest[i] == public_row_hashes[row * DIGEST_LEN + i]

        for cell in range(0, N_CELLS):
            digest = hash_contiguous_chunks(row_base + cell * C, CELL_CHUNKS)
            copy_digest(digest, cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    for cell in range(0, N_CELLS):
        for row in unroll(N, N_PADDED):
            zero_digest(cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    column_roots = Array(N_CELLS * DIGEST_LEN)
    for cell in range(0, N_CELLS):
        root = merkle_root_from_digests(cell_digests + cell * N_PADDED * DIGEST_LEN, LOG_N_PADDED)
        copy_digest(root, column_roots + cell * DIGEST_LEN)

    outer_tree = Array(OUTER_TREE_DIGESTS * DIGEST_LEN)
    for cell in range(0, N_CELLS):
        copy_digest(column_roots + cell * DIGEST_LEN, outer_tree + cell * DIGEST_LEN)

    for level in unroll(0, OUTER_MERKLE_DEPTH):
        input_offset = OUTER_LEVEL_OFFSETS[level]
        output_offset = OUTER_LEVEL_OFFSETS[level + 1]
        for node in range(0, OUTER_LEVEL_SIZES[level + 1]):
            poseidon16_compress_half(
                outer_tree + (input_offset + 2 * node) * DIGEST_LEN,
                outer_tree + (input_offset + 2 * node + 1) * DIGEST_LEN,
                outer_tree + (output_offset + node) * DIGEST_LEN,
            )

    root_offset = OUTER_LEVEL_OFFSETS[OUTER_MERKLE_DEPTH] * DIGEST_LEN
    for i in unroll(0, DIGEST_LEN):
        assert outer_tree[root_offset + i] == public_root_col[i]

    for row in range(0, N):
        result = Array(DIM)
        dot_product_be(codewords + row * M, check_vector, result, M)
        assert_ext_zero(result)
    return
