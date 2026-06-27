use std::{collections::BTreeSet, time::Duration};

use backend::{ArenaVec, PrimeCharacteristicRing, PrimeField32, arena_vec, poseidon16_compress_pair};
use lean_compiler::{CompilationFlags, ProgramSource, compile_program_with_flags};
use lean_prover::{default_whir_config, prove_execution::prove_execution, verify_execution::verify_execution};
use lean_vm::{Bytecode, ExecutionWitness, F, Hints};

use crate::{
    Commitment, DIGEST_LEN, DemoError, EXT_DEGREE, ParameterProfile, PreparedStatement, ProofBundle,
    encoding::{Codewords, Data, ErasureDecoder, encode},
    hashing::{Digest, merkle_layers},
    membership,
};

pub const SUBSET_CLIENTS: usize = 10_000;
pub const SUBSET_EPSILON_NUMERATOR: usize = 1;
pub const SUBSET_EPSILON_DENOMINATOR: usize = 100;
pub const SUBSET_SOUNDNESS_BITS: usize = 40;
pub const V2_OPENED_CELLS: usize = 19;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Full,
    RowHashOnly,
    CellCommitOnly,
    MembershipOnly,
}

impl Relation {
    /// Returns which V2 relation blocks are enabled in the LeanVM guest.
    pub const fn enabled(self) -> (bool, bool, bool) {
        match self {
            Self::Full => (true, true, true),
            Self::RowHashOnly => (true, false, false),
            Self::CellCommitOnly => (false, true, false),
            Self::MembershipOnly => (false, false, true),
        }
    }

    /// Returns whether the guest reads public row hashes.
    pub const fn needs_row_hashes(self) -> bool {
        matches!(self, Self::Full | Self::RowHashOnly)
    }

    /// Returns whether the guest reads the public column root.
    pub const fn needs_root(self) -> bool {
        matches!(self, Self::Full | Self::CellCommitOnly)
    }

    /// Returns whether the guest reads the public RS check vector.
    pub const fn needs_check_vector(self) -> bool {
        matches!(self, Self::Full | Self::MembershipOnly)
    }

    /// Returns a stable benchmark label for this relation mode.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::RowHashOnly => "row-hash-only",
            Self::CellCommitOnly => "cell-commit-only",
            Self::MembershipOnly => "membership-only",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuxiliaryData {
    pub profile: ParameterProfile,
    pub codewords: Codewords,
    pub column_roots: Vec<Digest>,
    pub outer_merkle_layers: Vec<Vec<Digest>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellOpening {
    pub index: usize,
    pub cells: Vec<Vec<F>>,
    pub outer_authentication_path: Vec<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transcript {
    pub openings: Vec<CellOpening>,
}

#[derive(Clone, Debug)]
pub struct BenchmarkTimings {
    pub encode_commit: Duration,
    pub prover_preprocess: Duration,
    pub prove: Duration,
    pub opening_generation: Duration,
    pub verifier_rebuild: Duration,
    pub proof_verify: Duration,
    pub verify_openings: Duration,
    pub reconstruct: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    pub relation: Relation,
    pub profile: ParameterProfile,
    pub commitment: Commitment,
    pub prepared: PreparedStatement,
    pub proof: ProofBundle,
    pub transcript: Transcript,
    pub opened_cells: usize,
    pub reconstruction: Option<bool>,
    pub timings: BenchmarkTimings,
    pub accepted: bool,
}

/// Returns the power-of-two row count used inside every column Merkle tree.
pub fn padded_rows(profile: ParameterProfile) -> usize {
    profile.n.next_power_of_two()
}

/// Returns the binary depth of each inner column Merkle tree.
pub fn column_merkle_depth(profile: ParameterProfile) -> usize {
    padded_rows(profile).ilog2() as usize
}

/// Hashes one base-field cell into a Poseidon digest.
pub fn cell_hash(cell: &[F]) -> Digest {
    fixed_compression_hash(cell)
}

/// Maps a V2 physical even-first codeword index back to the logical FFT-domain index.
fn physical_to_logical(profile: ParameterProfile, index: usize) -> usize {
    debug_assert!(index < profile.m);
    if index < profile.k {
        2 * index
    } else {
        2 * (index - profile.k) + 1
    }
}

/// Reorders a logical codeword as all even-domain symbols followed by all odd-domain symbols.
fn logical_to_physical_codeword(profile: ParameterProfile, row: &[F]) -> Vec<F> {
    debug_assert_eq!(row.len(), profile.m);
    (0..profile.m)
        .map(|index| row[physical_to_logical(profile, index)])
        .collect()
}

fn physical_codewords(profile: ParameterProfile, codewords: Codewords) -> Codewords {
    codewords
        .iter()
        .map(|row| logical_to_physical_codeword(profile, row))
        .collect()
}

/// Hashes the systematic cell digests of one row as specified by V2.
fn row_hash_from_cell_digests(
    profile: ParameterProfile,
    n_padded: usize,
    cell_digests: &[Digest],
    row: usize,
) -> Digest {
    let mut chunks = (0..profile.reconstruction_threshold_cells()).map(|cell| cell_digests[cell * n_padded + row]);
    compression_chain_from_chunks(&mut chunks)
}

/// Hashes field data as a fixed-length chain of Poseidon16 compression calls.
fn fixed_compression_hash(data: &[F]) -> Digest {
    debug_assert!(!data.is_empty());
    debug_assert!(data.len().is_multiple_of(DIGEST_LEN));
    let mut chunks = data.chunks_exact(DIGEST_LEN).map(|chunk| chunk.try_into().unwrap());
    compression_chain_from_chunks(&mut chunks)
}

fn compression_chain_from_chunks(chunks: &mut impl Iterator<Item = Digest>) -> Digest {
    let zero = [F::ZERO; DIGEST_LEN];
    let first = chunks
        .next()
        .expect("fixed-compression hash requires at least one chunk");
    let Some(second) = chunks.next() else {
        return poseidon16_compress_pair(&zero, &first);
    };
    chunks.fold(poseidon16_compress_pair(&first, &second), |state, chunk| {
        poseidon16_compress_pair(&state, &chunk)
    })
}

/// Encodes data and constructs V2's row digests and column-root commitment.
pub fn encode_and_commit(profile: ParameterProfile, data: &Data) -> Result<(Commitment, AuxiliaryData), DemoError> {
    profile.validate()?;
    if data.len() != profile.n || data.iter().any(|blob| blob.len() != profile.k) {
        return Err(DemoError::InvalidDataShape);
    }
    let codewords = physical_codewords(profile, encode(profile, data));
    let n_padded = padded_rows(profile);
    let zero = [F::ZERO; DIGEST_LEN];
    let mut cell_digests = vec![zero; profile.n_cells() * n_padded];

    for cell in 0..profile.n_cells() {
        let start = cell * profile.c;
        for row in 0..profile.n {
            cell_digests[cell * n_padded + row] = cell_hash(&codewords[row][start..start + profile.c]);
        }
    }

    let row_hashes = (0..profile.n)
        .map(|row| row_hash_from_cell_digests(profile, n_padded, &cell_digests, row))
        .collect();
    let column_roots = (0..profile.n_cells())
        .map(|cell| {
            merkle_layers(&cell_digests[cell * n_padded..(cell + 1) * n_padded])
                .last()
                .unwrap()[0]
        })
        .collect::<Vec<_>>();

    let outer_merkle_layers = merkle_layers(&column_roots);
    let commitment = Commitment {
        profile,
        row_hashes,
        root: outer_merkle_layers.last().unwrap()[0],
    };
    Ok((
        commitment,
        AuxiliaryData {
            profile,
            codewords,
            column_roots,
            outer_merkle_layers,
        },
    ))
}

/// Samples distinct cell-column indices from the public commitment root and external randomness.
pub fn sample_query_indices(
    commitment: &Commitment,
    randomness: &[F; DIGEST_LEN],
    count: usize,
) -> Result<Vec<usize>, DemoError> {
    if count > commitment.profile.n_cells() {
        return Err(DemoError::InvalidQuery);
    }
    let mut indices = Vec::with_capacity(count);
    let mut seen = BTreeSet::new();
    let mut counter = 0u32;
    while indices.len() < count {
        let mut block = *randomness;
        block[0] += F::from_u32(counter);
        let digest = poseidon16_compress_pair(&commitment.root, &block);
        for word in digest {
            let index = word.as_canonical_u32() as usize % commitment.profile.n_cells();
            if seen.insert(index) {
                indices.push(index);
                if indices.len() == count {
                    break;
                }
            }
        }
        counter = counter.wrapping_add(1);
    }
    Ok(indices)
}

/// Returns the canonical byte size of the public V2-base commitment.
pub fn commitment_size_bytes(commitment: &Commitment) -> usize {
    (commitment.row_hashes.len() * DIGEST_LEN + DIGEST_LEN) * size_of::<u32>()
}

/// Opens requested cell columns and attaches only the outer column-root Merkle paths.
pub fn query(aux: &AuxiliaryData, indices: &[usize]) -> Result<Transcript, DemoError> {
    let profile = aux.profile;
    let mut seen = BTreeSet::new();
    let mut openings = Vec::with_capacity(indices.len());
    for &index in indices {
        if index >= profile.n_cells() || !seen.insert(index) {
            return Err(DemoError::InvalidQuery);
        }
        let start = index * profile.c;
        let cells = aux
            .codewords
            .iter()
            .map(|row| row[start..start + profile.c].to_vec())
            .collect();
        let mut node = index;
        let mut outer_authentication_path = Vec::with_capacity(profile.merkle_depth());
        for layer in aux.outer_merkle_layers.iter().take(profile.merkle_depth()) {
            outer_authentication_path.push(layer[node ^ 1]);
            node /= 2;
        }
        openings.push(CellOpening {
            index,
            cells,
            outer_authentication_path,
        });
    }
    Ok(Transcript { openings })
}

/// Verifies opened cells by recomputing the inner column root and its outer path.
pub fn verify_openings(commitment: &Commitment, transcript: &Transcript) -> bool {
    let profile = commitment.profile;
    let n_padded = padded_rows(profile);
    let zero = [F::ZERO; DIGEST_LEN];
    let mut seen = BTreeSet::new();
    transcript.openings.iter().all(|opening| {
        if opening.index >= profile.n_cells()
            || !seen.insert(opening.index)
            || opening.cells.len() != profile.n
            || opening.cells.iter().any(|cell| cell.len() != profile.c)
            || opening.outer_authentication_path.len() != profile.merkle_depth()
        {
            return false;
        }

        let mut leaves = vec![zero; n_padded];
        for (row, cell) in opening.cells.iter().enumerate() {
            leaves[row] = cell_hash(cell);
        }
        let mut digest = merkle_layers(&leaves).last().unwrap()[0];
        let mut node = opening.index;
        for sibling in &opening.outer_authentication_path {
            digest = if node.is_multiple_of(2) {
                poseidon16_compress_pair(&digest, sibling)
            } else {
                poseidon16_compress_pair(sibling, &digest)
            };
            node /= 2;
        }
        digest == commitment.root
    })
}

/// Returns the canonical byte size of queried indices, cells, and outer Merkle paths.
pub fn transcript_size_bytes(transcript: &Transcript) -> usize {
    transcript
        .openings
        .iter()
        .map(|opening| {
            size_of::<u32>()
                + opening.cells.iter().map(Vec::len).sum::<usize>() * size_of::<u32>()
                + opening.outer_authentication_path.len() * DIGEST_LEN * size_of::<u32>()
        })
        .sum()
}

/// Reconstructs the original blobs after verifying enough distinct V2 cell columns.
pub fn reconstruct(commitment: &Commitment, transcripts: &[Transcript]) -> Result<Data, DemoError> {
    let profile = commitment.profile;
    let mut openings = std::collections::HashMap::new();
    for transcript in transcripts {
        if !verify_openings(commitment, transcript) {
            return Err(DemoError::InvalidOpening);
        }
        for opening in &transcript.openings {
            openings.entry(opening.index).or_insert_with(|| opening.clone());
        }
    }
    if openings.len() < profile.reconstruction_threshold_cells() {
        return Err(DemoError::InsufficientCells);
    }

    let mut indices: Vec<_> = openings.keys().copied().collect();
    indices.sort_unstable();
    let symbol_indices: Vec<_> = indices
        .iter()
        .flat_map(|&index| (0..profile.c).map(move |offset| physical_to_logical(profile, index * profile.c + offset)))
        .collect();
    let decoder = ErasureDecoder::new(profile, &symbol_indices).ok_or(DemoError::ReconstructionFailed)?;
    (0..profile.n)
        .map(|row| {
            let values: Vec<_> = indices
                .iter()
                .flat_map(|&index| {
                    let opening = &openings[&index];
                    (0..profile.c).map(move |offset| opening.cells[row][offset])
                })
                .collect();
            decoder.reconstruct_blob(&values).ok_or(DemoError::ReconstructionFailed)
        })
        .collect()
}

/// Computes the V2 subset-soundness log2 failure bound without replacement.
pub fn subset_log2_failure(profile: ParameterProfile, opened_cells: usize) -> f64 {
    let ell = profile.n_cells();
    let delta = profile.reconstruction_threshold_cells() - 1;
    let l_sub = SUBSET_CLIENTS * SUBSET_EPSILON_NUMERATOR / SUBSET_EPSILON_DENOMINATOR;
    if opened_cells > delta {
        return f64::NEG_INFINITY;
    }
    log2_binomial(ell, delta)
        + log2_binomial(SUBSET_CLIENTS, l_sub)
        + (l_sub as f64) * (log2_binomial(delta, opened_cells) - log2_binomial(ell, opened_cells))
}

fn log2_binomial(n: usize, k: usize) -> f64 {
    if k > n {
        return f64::NEG_INFINITY;
    }
    let k = k.min(n - k);
    (0..k).map(|i| ((n - i) as f64).log2() - ((i + 1) as f64).log2()).sum()
}

fn guest_source(relation: Relation) -> ProgramSource {
    let source = match relation {
        Relation::Full => include_str!("../../zkdsl/v2_base/full.py"),
        Relation::RowHashOnly | Relation::CellCommitOnly | Relation::MembershipOnly => {
            include_str!("../../zkdsl/v2_base/main.py")
        }
    };
    ProgramSource::Raw(source.to_string())
}

fn compilation_flags(commitment: &Commitment, relation: Relation) -> Result<CompilationFlags, DemoError> {
    commitment.profile.validate()?;
    if commitment.row_hashes.len() != commitment.profile.n {
        return Err(DemoError::InvalidDataShape);
    }
    let profile = commitment.profile;
    let (row_hash_enabled, cell_commit_enabled, membership_enabled) = relation.enabled();
    let mut read_only_cursor = DIGEST_LEN;
    let row_hashes_ptr = read_only_cursor;
    if relation.needs_row_hashes() {
        read_only_cursor += profile.n * DIGEST_LEN;
    }
    let root_ptr = read_only_cursor;
    if relation.needs_root() {
        read_only_cursor += DIGEST_LEN;
    }
    let check_vector_ptr = read_only_cursor;

    let mut replacements = std::collections::BTreeMap::new();
    for (name, value) in [
        ("N_PLACEHOLDER", profile.n),
        ("N_PADDED_PLACEHOLDER", padded_rows(profile)),
        ("LOG_N_PADDED_PLACEHOLDER", column_merkle_depth(profile)),
        ("M_PLACEHOLDER", profile.m),
        ("K_PLACEHOLDER", profile.k),
        ("C_PLACEHOLDER", profile.c),
        ("N_CELLS_PLACEHOLDER", profile.n_cells()),
        ("SYSTEMATIC_CELLS_PLACEHOLDER", profile.reconstruction_threshold_cells()),
        ("SYSTEMATIC_STRIDE_PLACEHOLDER", profile.systematic_stride()),
        ("ROW_CHUNKS_PLACEHOLDER", profile.k / DIGEST_LEN),
        ("CELL_CHUNKS_PLACEHOLDER", profile.c / DIGEST_LEN),
        ("OUTER_MERKLE_DEPTH_PLACEHOLDER", profile.merkle_depth()),
        ("OUTER_TREE_DIGESTS_PLACEHOLDER", 2 * profile.n_cells() - 1),
        ("ROW_HASH_ENABLED_PLACEHOLDER", usize::from(row_hash_enabled)),
        ("CELL_COMMIT_ENABLED_PLACEHOLDER", usize::from(cell_commit_enabled)),
        ("MEMBERSHIP_ENABLED_PLACEHOLDER", usize::from(membership_enabled)),
        ("PUBLIC_ROW_HASHES_PTR_PLACEHOLDER", row_hashes_ptr),
        ("PUBLIC_ROOT_COL_PTR_PLACEHOLDER", root_ptr),
        ("CHECK_VECTOR_PTR_PLACEHOLDER", check_vector_ptr),
    ] {
        replacements.insert(name.to_string(), value.to_string());
    }

    let mut sizes = Vec::with_capacity(profile.merkle_depth() + 1);
    let mut offsets = Vec::with_capacity(profile.merkle_depth() + 1);
    let mut size = profile.n_cells();
    let mut offset = 0;
    loop {
        sizes.push(size);
        offsets.push(offset);
        if size == 1 {
            break;
        }
        offset += size;
        size /= 2;
    }
    replacements.insert(
        "OUTER_LEVEL_SIZES_PLACEHOLDER".to_string(),
        format!("[{}]", sizes.iter().map(usize::to_string).collect::<Vec<_>>().join(",")),
    );
    replacements.insert(
        "OUTER_LEVEL_OFFSETS_PLACEHOLDER".to_string(),
        format!(
            "[{}]",
            offsets.iter().map(usize::to_string).collect::<Vec<_>>().join(",")
        ),
    );

    Ok(CompilationFlags { replacements })
}

fn leanvm_public_input() -> [F; DIGEST_LEN] {
    [F::ZERO; DIGEST_LEN]
}

fn read_only_data(commitment: &Commitment, check_vector: &membership::CheckVector, relation: Relation) -> Vec<F> {
    let mut capacity = 0;
    if relation.needs_row_hashes() {
        capacity += commitment.profile.n * DIGEST_LEN;
    }
    if relation.needs_root() {
        capacity += DIGEST_LEN;
    }
    if relation.needs_check_vector() {
        capacity += commitment.profile.m * EXT_DEGREE;
    }

    let mut data = Vec::with_capacity(capacity);
    if relation.needs_row_hashes() {
        data.extend(commitment.row_hashes.iter().flatten().copied());
    }
    if relation.needs_root() {
        data.extend_from_slice(&commitment.root);
    }
    if relation.needs_check_vector() {
        data.extend(check_vector.iter().flatten().copied());
    }
    data
}

/// Recomputes V2 Fiat-Shamir data, generates physical-order L, and compiles the full V2 guest.
pub fn prepare_statement(commitment: Commitment) -> Result<PreparedStatement, DemoError> {
    prepare_statement_with_relation(commitment, Relation::Full)
}

/// Recomputes V2 Fiat-Shamir data and compiles the selected relation benchmark guest.
pub fn prepare_statement_with_relation(
    commitment: Commitment,
    relation: Relation,
) -> Result<PreparedStatement, DemoError> {
    let check_vector = if relation.needs_check_vector() {
        membership::physical_check_vector(&commitment).ok_or(DemoError::ChallengeOnDomain)?
    } else {
        Vec::new()
    };
    let bytecode = compile_program_with_flags(&guest_source(relation), compilation_flags(&commitment, relation)?)
        .with_read_only_data(read_only_data(&commitment, &check_vector, relation));
    Ok(PreparedStatement {
        commitment,
        check_vector,
        bytecode,
    })
}

fn witness(bytecode: &Bytecode, codewords: &Codewords) -> ExecutionWitness {
    let flattened: Vec<_> = codewords.iter().flat_map(|row| row.iter().copied()).collect();
    let mut hints = Hints::default();
    hints.insert(bytecode, "codewords", arena_vec![ArenaVec::from_slice(&flattened)]);
    ExecutionWitness {
        hints,
        ..Default::default()
    }
}

/// Proves the V2 cell-first commitment and RS dot-product statement.
pub fn prove_codewords(prepared: &PreparedStatement, codewords: &Codewords) -> Result<ProofBundle, DemoError> {
    let profile = prepared.commitment.profile;
    if codewords.len() != profile.n || codewords.iter().any(|row| row.len() != profile.m) {
        return Err(DemoError::InvalidDataShape);
    }
    let execution = prove_execution(
        &prepared.bytecode,
        &leanvm_public_input(),
        &witness(&prepared.bytecode, codewords),
        &default_whir_config(profile.whir_log_inv_rate),
        false,
    )?;
    Ok(ProofBundle { execution })
}

/// Verifies a LeanVM proof against an already rebuilt V2 statement.
pub fn verify_prepared_execution_proof(prepared: &PreparedStatement, proof: &ProofBundle) -> Result<(), DemoError> {
    verify_execution(
        &prepared.bytecode,
        &leanvm_public_input(),
        proof.execution.proof.clone(),
    )
    .map(|_| ())
    .map_err(DemoError::Verification)
}

/// Rebuilds the verifier's full V2 statement and verifies the LeanVM proof.
pub fn verify_execution_proof(commitment: &Commitment, proof: &ProofBundle) -> Result<(), DemoError> {
    verify_execution_proof_with_relation(commitment, proof, Relation::Full)
}

/// Rebuilds the verifier's selected V2 relation statement and verifies the LeanVM proof.
pub fn verify_execution_proof_with_relation(
    commitment: &Commitment,
    proof: &ProofBundle,
    relation: Relation,
) -> Result<(), DemoError> {
    let prepared = prepare_statement_with_relation(commitment.clone(), relation)?;
    verify_prepared_execution_proof(&prepared, proof)
}

/// Rebuilds the public V2 statement, verifies its proof, and checks openings.
pub fn verify(commitment: &Commitment, proof: &ProofBundle, transcript: &Transcript) -> Result<bool, DemoError> {
    verify_execution_proof(commitment, proof)?;
    Ok(verify_openings(commitment, transcript))
}
