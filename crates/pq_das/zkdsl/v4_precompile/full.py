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

PUBLIC_ROOT_PTR = PUBLIC_ROOT_PTR_PLACEHOLDER
CHECK_VECTOR_PTR = CHECK_VECTOR_PTR_PLACEHOLDER
PQ_DAS_COMMITMENT_SCRATCH_LEN = PQ_DAS_COMMITMENT_SCRATCH_LEN_PLACEHOLDER


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


def merkle_root_16_from_digests_into(leaves, dest):
    level_8 = Array(8 * DIGEST_LEN)
    for node in unroll(0, 8):
        poseidon16_compress_half(
            leaves + (2 * node) * DIGEST_LEN,
            leaves + (2 * node + 1) * DIGEST_LEN,
            level_8 + node * DIGEST_LEN,
        )

    level_4 = Array(4 * DIGEST_LEN)
    for node in unroll(0, 4):
        poseidon16_compress_half(
            level_8 + (2 * node) * DIGEST_LEN,
            level_8 + (2 * node + 1) * DIGEST_LEN,
            level_4 + node * DIGEST_LEN,
        )

    level_2 = Array(2 * DIGEST_LEN)
    for node in unroll(0, 2):
        poseidon16_compress_half(
            level_4 + (2 * node) * DIGEST_LEN,
            level_4 + (2 * node + 1) * DIGEST_LEN,
            level_2 + node * DIGEST_LEN,
        )

    poseidon16_compress_half(level_2, level_2 + DIGEST_LEN, dest)
    return


def merkle_root_16_column_into(leaves, dest):
    zero = zero_digest_ret()
    level_8 = Array(8 * DIGEST_LEN)
    for node in unroll(0, 8):
        left_index = 2 * node
        right_index = left_index + 1
        if N <= left_index:
            poseidon16_compress_half(zero, zero, level_8 + node * DIGEST_LEN)
        else:
            if N <= right_index:
                poseidon16_compress_half(
                    leaves + left_index * DIGEST_LEN,
                    zero,
                    level_8 + node * DIGEST_LEN,
                )
            else:
                poseidon16_compress_half(
                    leaves + left_index * DIGEST_LEN,
                    leaves + right_index * DIGEST_LEN,
                    level_8 + node * DIGEST_LEN,
                )

    level_4 = Array(4 * DIGEST_LEN)
    for node in unroll(0, 4):
        poseidon16_compress_half(
            level_8 + (2 * node) * DIGEST_LEN,
            level_8 + (2 * node + 1) * DIGEST_LEN,
            level_4 + node * DIGEST_LEN,
        )

    level_2 = Array(2 * DIGEST_LEN)
    for node in unroll(0, 2):
        poseidon16_compress_half(
            level_4 + (2 * node) * DIGEST_LEN,
            level_4 + (2 * node + 1) * DIGEST_LEN,
            level_2 + node * DIGEST_LEN,
        )

    poseidon16_compress_half(level_2, level_2 + DIGEST_LEN, dest)
    return


# V4-precompile: relay the whole V3-ext commitment through a dedicated table,
# then check all rows with one membership-batch table. Both macro tables call
# existing LeanVM Poseidon/ExtensionOp tables, so the cryptographic relation is unchanged.
def main():
    codewords = Array(N * M_EXT * DIM)
    hint_witness("codewords", codewords)

    public_root = PUBLIC_ROOT_PTR
    check_vector = CHECK_VECTOR_PTR
    commitment_scratch = Array(PQ_DAS_COMMITMENT_SCRATCH_LEN_PLACEHOLDER)
    pq_das_commitment(codewords, public_root, commitment_scratch, N, M_EXT, C_EXT)

    membership_results = Array(N * DIM)
    pq_das_membership_batch(codewords, check_vector, membership_results, N, M_EXT)
    return
