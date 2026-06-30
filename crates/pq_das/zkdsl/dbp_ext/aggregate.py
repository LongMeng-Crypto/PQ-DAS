from snark_lib import *

DIGEST_LEN = 8
N = N_PLACEHOLDER
N_PADDED = N_PADDED_PLACEHOLDER
LOG_N_PADDED = LOG_N_PADDED_PLACEHOLDER
N_CELLS = N_CELLS_PLACEHOLDER
OUTER_MERKLE_DEPTH = OUTER_MERKLE_DEPTH_PLACEHOLDER
PUBLIC_ROOT_PTR = PUBLIC_ROOT_PTR_PLACEHOLDER


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


def hash_full_row_cell_digests(cell_digests, row):
    if N_CELLS == 1:
        out_one = Array(DIGEST_LEN)
        zero = Array(DIGEST_LEN)
        zero_digest(zero)
        poseidon16_compress_half(zero, cell_digests + row * DIGEST_LEN, out_one)
        return out_one
    if N_CELLS == 2:
        out_two = Array(DIGEST_LEN)
        poseidon16_compress_half(
            cell_digests + row * DIGEST_LEN,
            cell_digests + (N_PADDED + row) * DIGEST_LEN,
            out_two,
        )
        return out_two

    states = Array((N_CELLS - 2) * DIGEST_LEN)
    poseidon16_compress_half(
        cell_digests + row * DIGEST_LEN,
        cell_digests + (N_PADDED + row) * DIGEST_LEN,
        states,
    )
    for cell in range(1, N_CELLS - 2):
        poseidon16_compress_half(
            states + (cell - 1) * DIGEST_LEN,
            cell_digests + ((cell + 1) * N_PADDED + row) * DIGEST_LEN,
            states + cell * DIGEST_LEN,
        )
    out_many = Array(DIGEST_LEN)
    poseidon16_compress_half(
        states + (N_CELLS - 3) * DIGEST_LEN,
        cell_digests + ((N_CELLS - 1) * N_PADDED + row) * DIGEST_LEN,
        out_many,
    )
    return out_many


def main():
    public_root = PUBLIC_ROOT_PTR
    cell_digests = Array(N_CELLS * N_PADDED * DIGEST_LEN)
    hint_witness("cell_digests", cell_digests)

    for cell in range(0, N_CELLS):
        for row in unroll(N, N_PADDED):
            zero_digest(cell_digests + (cell * N_PADDED + row) * DIGEST_LEN)

    row_leaves = Array(N_PADDED * DIGEST_LEN)
    for row in range(0, N):
        row_digest = hash_full_row_cell_digests(cell_digests, row)
        copy_digest(row_digest, row_leaves + row * DIGEST_LEN)
    for row in unroll(N, N_PADDED):
        zero_digest(row_leaves + row * DIGEST_LEN)
    root_row = merkle_root_from_digests(row_leaves, LOG_N_PADDED)

    column_roots = Array(N_CELLS * DIGEST_LEN)
    for cell in range(0, N_CELLS):
        root = merkle_root_from_digests(cell_digests + cell * N_PADDED * DIGEST_LEN, LOG_N_PADDED)
        copy_digest(root, column_roots + cell * DIGEST_LEN)
    root_col = merkle_root_from_digests(column_roots, OUTER_MERKLE_DEPTH)

    final_root = Array(DIGEST_LEN)
    poseidon16_compress_half(root_row, root_col, final_root)
    for i in unroll(0, DIGEST_LEN):
        assert final_root[i] == public_root[i]
    return
