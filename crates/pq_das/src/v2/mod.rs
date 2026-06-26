use std::{collections::BTreeSet, time::Duration};

use backend::{ArenaVec, PrimeCharacteristicRing, arena_vec, poseidon_hash_slice, poseidon16_compress_pair};
use lean_compiler::{CompilationFlags, ProgramSource, compile_program_with_flags};
use lean_prover::{default_whir_config, prove_execution::prove_execution, verify_execution::verify_execution};
use lean_vm::{Bytecode, ExecutionWitness, F, Hints};

use crate::{
    Commitment, DIGEST_LEN, DemoError, EXT_DEGREE, ParameterProfile, PreparedStatement, ProofBundle,
    encoding::{Codewords, Data, ErasureDecoder, encode},
    hashing::{Digest, merkle_layers, row_hash},
    membership,
};

pub const SUBSET_CLIENTS: usize = 10_000;
pub const SUBSET_EPSILON_NUMERATOR: usize = 1;
pub const SUBSET_EPSILON_DENOMINATOR: usize = 100;
pub const SUBSET_SOUNDNESS_BITS: usize = 40;
pub const V2_OPENED_CELLS: usize = 19;

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
    pub verifier_rebuild_verify: Duration,
    pub verify_openings: Duration,
    pub reconstruct: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct BenchmarkResult {
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
    poseidon_hash_slice(cell)
}

/// Encodes data and constructs V2's row digests and column-root commitment.
pub fn encode_and_commit(profile: ParameterProfile, data: &Data) -> Result<(Commitment, AuxiliaryData), DemoError> {
    profile.validate()?;
    if data.len() != profile.n || data.iter().any(|blob| blob.len() != profile.k) {
        return Err(DemoError::InvalidDataShape);
    }
    let codewords = encode(profile, data);
    let row_hashes = codewords.iter().map(|row| row_hash(profile, row)).collect();
    let n_padded = padded_rows(profile);
    let zero = [F::ZERO; DIGEST_LEN];
    let mut column_roots = Vec::with_capacity(profile.n_cells());

    for cell in 0..profile.n_cells() {
        let start = cell * profile.c;
        let mut leaves = vec![zero; n_padded];
        for row in 0..profile.n {
            leaves[row] = cell_hash(&codewords[row][start..start + profile.c]);
        }
        column_roots.push(merkle_layers(&leaves).last().unwrap()[0]);
    }

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
        .flat_map(|&index| (0..profile.c).map(move |offset| index * profile.c + offset))
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

fn guest_source() -> ProgramSource {
    ProgramSource::Raw(include_str!("../../zkdsl/v2/main.py").to_string())
}

fn compilation_flags(commitment: &Commitment) -> Result<CompilationFlags, DemoError> {
    commitment.profile.validate()?;
    if commitment.row_hashes.len() != commitment.profile.n {
        return Err(DemoError::InvalidDataShape);
    }
    let profile = commitment.profile;
    let mut replacements = std::collections::BTreeMap::new();
    for (name, value) in [
        ("N_PLACEHOLDER", profile.n),
        ("N_PADDED_PLACEHOLDER", padded_rows(profile)),
        ("LOG_N_PADDED_PLACEHOLDER", column_merkle_depth(profile)),
        ("M_PLACEHOLDER", profile.m),
        ("K_PLACEHOLDER", profile.k),
        ("C_PLACEHOLDER", profile.c),
        ("N_CELLS_PLACEHOLDER", profile.n_cells()),
        ("SYSTEMATIC_STRIDE_PLACEHOLDER", profile.systematic_stride()),
        ("ROW_CHUNKS_PLACEHOLDER", profile.k / DIGEST_LEN),
        ("CELL_CHUNKS_PLACEHOLDER", profile.c / DIGEST_LEN),
        ("OUTER_MERKLE_DEPTH_PLACEHOLDER", profile.merkle_depth()),
        ("OUTER_TREE_DIGESTS_PLACEHOLDER", 2 * profile.n_cells() - 1),
        ("PUBLIC_ROW_HASHES_PTR_PLACEHOLDER", DIGEST_LEN),
        ("PUBLIC_ROOT_COL_PTR_PLACEHOLDER", DIGEST_LEN + profile.n * DIGEST_LEN),
        ("CHECK_VECTOR_PTR_PLACEHOLDER", 2 * DIGEST_LEN + profile.n * DIGEST_LEN),
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

fn read_only_data(commitment: &Commitment, check_vector: &membership::CheckVector) -> Vec<F> {
    let mut data =
        Vec::with_capacity(commitment.profile.n * DIGEST_LEN + DIGEST_LEN + commitment.profile.m * EXT_DEGREE);
    data.extend(commitment.row_hashes.iter().flatten().copied());
    data.extend_from_slice(&commitment.root);
    data.extend(check_vector.iter().flatten().copied());
    data
}

/// Recomputes V2 Fiat-Shamir data, generates L, and compiles the V2 guest.
pub fn prepare_statement(commitment: Commitment) -> Result<PreparedStatement, DemoError> {
    let check_vector = membership::check_vector(&commitment).ok_or(DemoError::ChallengeOnDomain)?;
    let bytecode = compile_program_with_flags(&guest_source(), compilation_flags(&commitment)?)
        .with_read_only_data(read_only_data(&commitment, &check_vector));
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

/// Rebuilds the verifier's V2 statement and verifies the LeanVM proof.
pub fn verify_execution_proof(commitment: &Commitment, proof: &ProofBundle) -> Result<(), DemoError> {
    let prepared = prepare_statement(commitment.clone())?;
    verify_execution(
        &prepared.bytecode,
        &leanvm_public_input(),
        proof.execution.proof.clone(),
    )
    .map(|_| ())
    .map_err(DemoError::Verification)
}

/// Rebuilds the public V2 statement, verifies its proof, and checks openings.
pub fn verify(commitment: &Commitment, proof: &ProofBundle, transcript: &Transcript) -> Result<bool, DemoError> {
    verify_execution_proof(commitment, proof)?;
    Ok(verify_openings(commitment, transcript))
}
