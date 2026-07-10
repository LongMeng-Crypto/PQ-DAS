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

PUBLIC_ROOT_PTR = PUBLIC_ROOT_PTR_PLACEHOLDER
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


def hash_contiguous_chunks_into(data, num_chunks: Const, dest):
    if num_chunks == 1:
        poseidon16_compress_half(zero_digest_ret(), data, dest)
        return
    if num_chunks == 2:
        poseidon16_compress_half(data, data + DIGEST_LEN, dest)
        return

    states = Array((num_chunks - 2) * DIGEST_LEN)
    poseidon16_compress_half(data, data + DIGEST_LEN, states)
    for chunk in range(1, num_chunks - 2):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            data + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    poseidon16_compress_half(
        states + (num_chunks - 3) * DIGEST_LEN,
        data + (num_chunks - 1) * DIGEST_LEN,
        dest,
    )
    return


def hash_cell_10_chunks_into(cell, dest):
    states = Array(8 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 8):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    poseidon16_compress_half(
        states + 7 * DIGEST_LEN,
        cell + 9 * DIGEST_LEN,
        dest,
    )
    return


def hash_cell_20_chunks_into(cell, dest):
    states = Array(18 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 18):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    poseidon16_compress_half(
        states + 17 * DIGEST_LEN,
        cell + 19 * DIGEST_LEN,
        dest,
    )
    return


def hash_cell_40_chunks_into(cell, dest):
    states = Array(38 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 38):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    poseidon16_compress_half(
        states + 37 * DIGEST_LEN,
        cell + 39 * DIGEST_LEN,
        dest,
    )
    return


def hash_cell_80_chunks_into(cell, dest):
    states = Array(78 * DIGEST_LEN)
    poseidon16_compress_half(cell, cell + DIGEST_LEN, states)
    for chunk in unroll(1, 78):
        poseidon16_compress_half(
            states + (chunk - 1) * DIGEST_LEN,
            cell + (chunk + 1) * DIGEST_LEN,
            states + chunk * DIGEST_LEN,
        )
    poseidon16_compress_half(
        states + 77 * DIGEST_LEN,
        cell + 79 * DIGEST_LEN,
        dest,
    )
    return


def hash_cell_into(cell, dest):
    if CELL_CHUNKS == 10:
        hash_cell_10_chunks_into(cell, dest)
        return
    if CELL_CHUNKS == 20:
        hash_cell_20_chunks_into(cell, dest)
        return
    if CELL_CHUNKS == 40:
        hash_cell_40_chunks_into(cell, dest)
        return
    if CELL_CHUNKS == 80:
        hash_cell_80_chunks_into(cell, dest)
        return
    hash_contiguous_chunks_into(cell, CELL_CHUNKS, dest)
    return


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


def main():
    codewords = Array(N * M_EXT * DIM)
    hint_witness("codewords", codewords)
    public_root = PUBLIC_ROOT_PTR
    check_vector = CHECK_VECTOR_PTR

    cell_digests = Array(N_CELLS * N_PADDED * DIGEST_LEN)
    row_hashes = Array(N_PADDED * DIGEST_LEN)

    for row in range(0, N):
        row_base = codewords + row * M_EXT * DIM
        first_digest = cell_digests + row * DIGEST_LEN
        hash_cell_into(row_base, first_digest)

        if SYSTEMATIC_CELLS == 1:
            poseidon16_compress_half(zero_digest_ret(), first_digest, row_hashes + row * DIGEST_LEN)
        else:
            second_digest = cell_digests + (N_PADDED + row) * DIGEST_LEN
            hash_cell_into(row_base + CELL_BASE_LEN, second_digest)

            if SYSTEMATIC_CELLS == 2:
                poseidon16_compress_half(first_digest, second_digest, row_hashes + row * DIGEST_LEN)
            else:
                row_states = Array((SYSTEMATIC_CELLS - 2) * DIGEST_LEN)
                poseidon16_compress_half(first_digest, second_digest, row_states)
                for cell in range(2, SYSTEMATIC_CELLS - 1):
                    digest = cell_digests + (cell * N_PADDED + row) * DIGEST_LEN
                    hash_cell_into(row_base + cell * CELL_BASE_LEN, digest)
                    poseidon16_compress_half(
                        row_states + (cell - 2) * DIGEST_LEN,
                        digest,
                        row_states + (cell - 1) * DIGEST_LEN,
                    )

                cell = SYSTEMATIC_CELLS - 1
                digest = cell_digests + (cell * N_PADDED + row) * DIGEST_LEN
                hash_cell_into(row_base + cell * CELL_BASE_LEN, digest)
                poseidon16_compress_half(
                    row_states + (SYSTEMATIC_CELLS - 3) * DIGEST_LEN,
                    digest,
                    row_hashes + row * DIGEST_LEN,
                )

        for cell in range(SYSTEMATIC_CELLS, N_CELLS):
            digest = cell_digests + (cell * N_PADDED + row) * DIGEST_LEN
            hash_cell_into(row_base + cell * CELL_BASE_LEN, digest)
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
        dot_product_ee(codewords + row * M_EXT * DIM, check_vector, result, M_EXT)
        assert_ext_zero(result)
    return
