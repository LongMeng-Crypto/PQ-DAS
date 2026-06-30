use std::{collections::BTreeSet, fmt::Display, time::Duration};

use backend::{
    Algebra, ArenaVec, BasedVectorSpace, Field, PrimeCharacteristicRing, PrimeField32, TwoAdicField, arena_vec,
    poseidon_hash_slice, poseidon16_compress_pair,
};
use lean_compiler::{CompilationFlags, ProgramSource, compile_program_with_flags};
use lean_prover::{default_whir_config, prove_execution::prove_execution, verify_execution::verify_execution};
use lean_vm::{Bytecode, EF, ExecutionWitness, F, Hints};

use crate::{
    DIGEST_LEN, DemoError, EXT_DEGREE, ProofBundle, fs_block,
    hashing::{Digest, merkle_layers},
};

pub const SUBSET_CLIENTS: usize = 10_000;
pub const SUBSET_EPSILON_NUMERATOR: usize = 1;
pub const SUBSET_EPSILON_DENOMINATOR: usize = 100;
pub const SUBSET_SOUNDNESS_BITS: usize = 40;

pub type ExtBlob = Vec<EF>;
pub type ExtCodeword = Vec<EF>;
pub type ExtData = Vec<ExtBlob>;
pub type ExtCodewords = Vec<ExtCodeword>;
pub type ExtCheckVector = Vec<[F; EXT_DEGREE]>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtProfile {
    pub name: &'static str,
    pub n: usize,
    pub m: usize,
    pub k: usize,
    pub c: usize,
    pub whir_log_inv_rate: usize,
}

impl ExtProfile {
    pub const BLOB_EXT_1: Self = Self {
        name: "blob-ext-1",
        n: 1,
        m: 16384,
        k: 8192,
        c: 16,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_14: Self = Self {
        name: "blob-ext-14",
        n: 14,
        m: 16384,
        k: 8192,
        c: 16,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_16: Self = Self {
        name: "blob-ext-16",
        n: 16,
        m: 16384,
        k: 8192,
        c: 16,
        whir_log_inv_rate: 1,
    };

    /// Returns the number of cell columns in one extension-field codeword.
    pub const fn n_cells(self) -> usize {
        self.m / self.c
    }

    /// Returns the number of cell columns needed for RS reconstruction.
    pub const fn reconstruction_threshold_cells(self) -> usize {
        self.k.div_ceil(self.c)
    }

    /// Returns the systematic spacing in the logical FFT domain.
    pub const fn systematic_stride(self) -> usize {
        self.m / self.k
    }

    /// Returns the outer column-root Merkle depth.
    pub const fn merkle_depth(self) -> usize {
        self.n_cells().ilog2() as usize
    }

    /// Checks the extension-field demo constraints.
    pub fn validate(self) -> Result<(), DemoError> {
        if self.n == 0 || self.k == 0 || self.m == 0 || self.c == 0 {
            return Err(DemoError::InvalidDataShape);
        }
        if !self.m.is_power_of_two() || !self.k.is_power_of_two() || self.m != 2 * self.k {
            return Err(DemoError::InvalidDataShape);
        }
        if !self.m.is_multiple_of(self.c) || !self.n_cells().is_power_of_two() {
            return Err(DemoError::InvalidDataShape);
        }
        if !(self.c * EXT_DEGREE).is_multiple_of(DIGEST_LEN) {
            return Err(DemoError::InvalidDataShape);
        }
        if !(1..=4).contains(&self.whir_log_inv_rate) || self.m.ilog2() > 24 {
            return Err(DemoError::InvalidDataShape);
        }
        Ok(())
    }

    /// Encodes the extension-field profile into one Fiat-Shamir block.
    pub fn profile_block(self) -> [F; DIGEST_LEN] {
        [
            F::from_u32(0x5051_4458),
            F::TWO,
            F::from_usize(self.n),
            F::from_usize(self.m),
            F::from_usize(self.k),
            F::from_usize(self.c),
            F::from_usize(self.n_cells()),
            F::from_usize(self.whir_log_inv_rate),
        ]
    }
}

impl Display for ExtProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtCommitment {
    pub profile: ExtProfile,
    pub root: Digest,
}

#[derive(Debug, Clone)]
pub struct ExtPreparedStatement {
    pub commitment: ExtCommitment,
    pub check_vector: ExtCheckVector,
    pub bytecode: Bytecode,
}

#[derive(Clone, Debug)]
pub struct ExtAuxiliaryData {
    pub profile: ExtProfile,
    pub codewords: ExtCodewords,
    pub row_root: Digest,
    pub column_roots: Vec<Digest>,
    pub outer_merkle_layers: Vec<Vec<Digest>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtCellOpening {
    pub index: usize,
    pub cells: Vec<Vec<F>>,
    pub outer_authentication_path: Vec<Digest>,
    pub row_root: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtTranscript {
    pub openings: Vec<ExtCellOpening>,
}

#[derive(Clone, Debug)]
pub struct ExtBenchmarkTimings {
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
pub struct ExtBenchmarkResult {
    pub profile: ExtProfile,
    pub commitment: ExtCommitment,
    pub prepared: ExtPreparedStatement,
    pub proof: ProofBundle,
    pub transcript: ExtTranscript,
    pub opened_cells: usize,
    pub reconstruction: Option<bool>,
    pub timings: ExtBenchmarkTimings,
    pub accepted: bool,
}

/// Returns the power-of-two row count used inside every inner column Merkle tree.
pub fn padded_rows(profile: ExtProfile) -> usize {
    profile.n.next_power_of_two()
}

/// Returns the binary depth of each inner column Merkle tree.
pub fn column_merkle_depth(profile: ExtProfile) -> usize {
    padded_rows(profile).ilog2() as usize
}

/// Returns one verifier's sampled cell count from the with-replacement subset-soundness formula.
pub fn opened_cells(profile: ExtProfile) -> usize {
    for q in 1..=profile.n_cells() {
        if subset_log2_failure_with_replacement(profile, q) <= -(SUBSET_SOUNDNESS_BITS as f64) {
            return q;
        }
    }
    profile.n_cells()
}

/// Derives distinct sampled cell indices from the extension commitment root and public randomness.
pub fn sample_query_indices(
    commitment: &ExtCommitment,
    randomness: &[F; DIGEST_LEN],
    count: usize,
) -> Result<Vec<usize>, DemoError> {
    let n_cells = commitment.profile.n_cells();
    if count == 0 || count > n_cells {
        return Err(DemoError::InvalidQuery);
    }
    let mut state = poseidon16_compress_pair(&commitment.root, randomness);
    let mut counter = 0;
    let mut seen = BTreeSet::new();
    let mut indices = Vec::with_capacity(count);
    while indices.len() < count {
        for value in state {
            let canonical = value.as_canonical_u32();
            let modulus = F::ORDER_U32;
            let unbiased_limit = modulus - modulus % n_cells as u32;
            if canonical >= unbiased_limit {
                continue;
            }
            let index = canonical as usize % n_cells;
            if seen.insert(index) {
                indices.push(index);
                if indices.len() == count {
                    break;
                }
            }
        }
        counter += 1;
        let mut counter_block = [F::ZERO; DIGEST_LEN];
        counter_block[0] = F::from_usize(counter);
        state = poseidon16_compress_pair(&state, &counter_block);
    }
    Ok(indices)
}

/// Computes log2 of the V3 subset-soundness bound for sampling with replacement.
pub fn subset_log2_failure_with_replacement(profile: ExtProfile, opened: usize) -> f64 {
    let ell = profile.n_cells();
    let delta = profile.reconstruction_threshold_cells() - 1;
    let l_sub = SUBSET_CLIENTS * SUBSET_EPSILON_NUMERATOR / SUBSET_EPSILON_DENOMINATOR;
    log2_binomial(ell, delta)
        + log2_binomial(SUBSET_CLIENTS, l_sub)
        + (l_sub * opened) as f64 * ((delta as f64) / (ell as f64)).log2()
}

fn log2_binomial(n: usize, k: usize) -> f64 {
    if k > n {
        return f64::NEG_INFINITY;
    }
    let k = k.min(n - k);
    (0..k).map(|i| ((n - i) as f64).log2() - ((i + 1) as f64).log2()).sum()
}

fn fft_with_root<A: Algebra<F> + Copy>(values: &mut [A], root: F) {
    let n = values.len();
    assert!(n.is_power_of_two());
    let shift = usize::BITS - n.ilog2();
    for i in 0..n {
        let j = i.reverse_bits() >> shift;
        if i < j {
            values.swap(i, j);
        }
    }
    let mut size = 2;
    while size <= n {
        let half = size / 2;
        let step = root.exp_u64((n / size) as u64);
        for chunk_start in (0..n).step_by(size) {
            let mut twiddle = F::ONE;
            for i in 0..half {
                let even = values[chunk_start + i];
                let odd = values[chunk_start + i + half] * twiddle;
                values[chunk_start + i] = even + odd;
                values[chunk_start + i + half] = even - odd;
                twiddle *= step;
            }
        }
        size *= 2;
    }
}

fn fft<A: Algebra<F> + Copy>(values: &mut [A]) {
    fft_with_root(values, F::two_adic_generator(values.len().ilog2() as usize));
}

fn ifft<A: Algebra<F> + Copy>(values: &mut [A]) {
    let root = F::two_adic_generator(values.len().ilog2() as usize);
    fft_with_root(values, root.inverse());
    let n_inv = F::from_usize(values.len()).inverse();
    for value in values {
        *value *= n_inv;
    }
}

fn multiply_polynomials_base(left: &[F], right: &[F]) -> Vec<F> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let output_len = left.len() + right.len() - 1;
    if left.len().min(right.len()) <= 16 {
        let mut output = vec![F::ZERO; output_len];
        for (i, &a) in left.iter().enumerate() {
            for (j, &b) in right.iter().enumerate() {
                output[i + j] += a * b;
            }
        }
        return output;
    }

    let fft_len = output_len.next_power_of_two();
    let mut left_evals = vec![F::ZERO; fft_len];
    let mut right_evals = vec![F::ZERO; fft_len];
    left_evals[..left.len()].copy_from_slice(left);
    right_evals[..right.len()].copy_from_slice(right);
    fft(&mut left_evals);
    fft(&mut right_evals);
    for (left, right) in left_evals.iter_mut().zip(right_evals) {
        *left *= right;
    }
    ifft(&mut left_evals);
    left_evals.truncate(output_len);
    left_evals
}

fn multiply_polynomials_ext_by_base(left: &[EF], right: &[F]) -> Vec<EF> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let output_len = left.len() + right.len() - 1;
    if left.len().min(right.len()) <= 16 {
        let mut output = vec![EF::ZERO; output_len];
        for (i, &a) in left.iter().enumerate() {
            for (j, &b) in right.iter().enumerate() {
                output[i + j] += a * b;
            }
        }
        return output;
    }

    let fft_len = output_len.next_power_of_two();
    let mut left_evals = vec![EF::ZERO; fft_len];
    let mut right_evals = vec![F::ZERO; fft_len];
    left_evals[..left.len()].copy_from_slice(left);
    right_evals[..right.len()].copy_from_slice(right);
    fft(&mut left_evals);
    fft(&mut right_evals);
    for (left, right) in left_evals.iter_mut().zip(right_evals) {
        *left *= right;
    }
    ifft(&mut left_evals);
    left_evals.truncate(output_len);
    left_evals
}

fn root_polynomial(roots: &[F]) -> Vec<F> {
    const CHUNK_ROOTS: usize = 16;

    let mut level: Vec<Vec<F>> = roots
        .chunks(CHUNK_ROOTS)
        .map(|chunk| {
            let mut polynomial = vec![F::ONE];
            for &root in chunk {
                let mut next = vec![F::ZERO; polynomial.len() + 1];
                for (degree, &coefficient) in polynomial.iter().enumerate() {
                    next[degree] -= coefficient * root;
                    next[degree + 1] += coefficient;
                }
                polynomial = next;
            }
            polynomial
        })
        .collect();
    if level.is_empty() {
        return vec![F::ONE];
    }

    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        let mut pairs = level.chunks_exact(2);
        for pair in &mut pairs {
            next.push(multiply_polynomials_base(&pair[0], &pair[1]));
        }
        if let Some(last) = pairs.remainder().first() {
            next.push(last.clone());
        }
        level = next;
    }
    level.pop().unwrap()
}

fn invert_series(polynomial: &[F], target_len: usize) -> Vec<F> {
    debug_assert!(!polynomial.is_empty() && polynomial[0] != F::ZERO);
    let mut inverse = vec![polynomial[0].inverse()];
    while inverse.len() < target_len {
        let next_len = (2 * inverse.len()).min(target_len);
        let product = multiply_polynomials_base(&polynomial[..polynomial.len().min(next_len)], &inverse);
        let mut correction = vec![F::ZERO; next_len];
        correction[0] = F::TWO;
        for (output, value) in correction.iter_mut().zip(product) {
            *output -= value;
        }
        inverse = multiply_polynomials_base(&inverse, &correction);
        inverse.truncate(next_len);
    }
    inverse
}

#[derive(Debug)]
pub struct ExtErasureDecoder {
    profile: ExtProfile,
    known_indices: Vec<usize>,
    locator_evaluations: Vec<F>,
    reversed_locator_inverse: Vec<F>,
    numerator_max_degree: usize,
}

impl ExtErasureDecoder {
    /// Precomputes the arbitrary-erasure locator over the base roots-of-unity domain.
    pub fn new(profile: ExtProfile, known_indices: &[usize]) -> Option<Self> {
        if known_indices.len() < profile.k {
            return None;
        }
        let mut known = vec![false; profile.m];
        for &index in known_indices {
            if index >= profile.m || std::mem::replace(&mut known[index], true) {
                return None;
            }
        }

        let omega = F::two_adic_generator(profile.m.ilog2() as usize);
        let mut point = F::ONE;
        let mut erased_points = Vec::with_capacity(profile.m - known_indices.len());
        for is_known in &known {
            if !is_known {
                erased_points.push(point);
            }
            point *= omega;
        }

        let locator = root_polynomial(&erased_points);
        let mut locator_evaluations = vec![F::ZERO; profile.m];
        locator_evaluations[..locator.len()].copy_from_slice(&locator);
        fft(&mut locator_evaluations);

        let reversed_locator: Vec<_> = locator.iter().rev().copied().collect();
        let reversed_locator_inverse = invert_series(&reversed_locator, profile.k);
        Some(Self {
            profile,
            known_indices: known_indices.to_vec(),
            locator_evaluations,
            reversed_locator_inverse,
            numerator_max_degree: profile.k + erased_points.len() - 1,
        })
    }

    /// Recovers the original extension-field coefficient blob from arbitrary codeword evaluations.
    pub fn reconstruct_blob(&self, values: &[EF]) -> Option<ExtBlob> {
        if values.len() != self.known_indices.len() {
            return None;
        }

        let mut numerator = vec![EF::ZERO; self.profile.m];
        for (&index, &value) in self.known_indices.iter().zip(values) {
            numerator[index] = value * self.locator_evaluations[index];
        }
        ifft(&mut numerator);

        let reversed_numerator: Vec<_> = (0..self.profile.k)
            .map(|offset| numerator[self.numerator_max_degree - offset])
            .collect();
        let mut reversed_coefficients =
            multiply_polynomials_ext_by_base(&reversed_numerator, &self.reversed_locator_inverse);
        reversed_coefficients.truncate(self.profile.k);
        reversed_coefficients.reverse();
        Some(reversed_coefficients)
    }
}

/// Encodes extension-field coefficients by zero-padding to half rate and applying an NTT.
pub fn encode_blob(profile: ExtProfile, blob: &[EF]) -> ExtCodeword {
    assert_eq!(blob.len(), profile.k);
    let mut codeword = vec![EF::ZERO; profile.m];
    codeword[..profile.k].copy_from_slice(blob);
    fft(&mut codeword);
    codeword
}

/// RS-encodes all extension-field blobs.
pub fn encode(profile: ExtProfile, data: &ExtData) -> ExtCodewords {
    assert_eq!(data.len(), profile.n);
    data.iter().map(|blob| encode_blob(profile, blob)).collect()
}

fn physical_to_logical(profile: ExtProfile, index: usize) -> usize {
    if index < profile.k {
        2 * index
    } else {
        2 * (index - profile.k) + 1
    }
}

fn logical_to_physical_codeword(profile: ExtProfile, row: &[EF]) -> ExtCodeword {
    (0..profile.m)
        .map(|index| row[physical_to_logical(profile, index)])
        .collect()
}

fn physical_codewords(profile: ExtProfile, codewords: ExtCodewords) -> ExtCodewords {
    codewords
        .iter()
        .map(|row| logical_to_physical_codeword(profile, row))
        .collect()
}

fn push_ext(out: &mut Vec<F>, value: EF) {
    out.extend_from_slice(value.as_basis_coefficients_slice());
}

fn ext_slice_to_base(values: &[EF]) -> Vec<F> {
    let mut out = Vec::with_capacity(values.len() * EXT_DEGREE);
    for &value in values {
        push_ext(&mut out, value);
    }
    out
}

fn ext_slice_from_base(values: &[F]) -> Option<Vec<EF>> {
    if !values.len().is_multiple_of(EXT_DEGREE) {
        return None;
    }
    values
        .chunks_exact(EXT_DEGREE)
        .map(EF::from_basis_coefficients_slice)
        .collect()
}

fn fixed_compression_hash(data: &[F]) -> Digest {
    debug_assert!(!data.is_empty());
    debug_assert!(data.len().is_multiple_of(DIGEST_LEN));
    let mut chunks = data.chunks_exact(DIGEST_LEN).map(|chunk| chunk.try_into().unwrap());
    compression_chain_from_chunks(&mut chunks)
}

fn compression_chain_from_chunks(chunks: &mut impl Iterator<Item = Digest>) -> Digest {
    let zero = [F::ZERO; DIGEST_LEN];
    let first = chunks.next().expect("hash requires at least one chunk");
    let Some(second) = chunks.next() else {
        return poseidon16_compress_pair(&zero, &first);
    };
    chunks.fold(poseidon16_compress_pair(&first, &second), |state, chunk| {
        poseidon16_compress_pair(&state, &chunk)
    })
}

/// Hashes one extension-field cell after serializing it to KoalaBear coordinates.
pub fn cell_hash(cell: &[EF]) -> Digest {
    fixed_compression_hash(&ext_slice_to_base(cell))
}

fn row_hash_from_cell_digests(profile: ExtProfile, n_padded: usize, cell_digests: &[Digest], row: usize) -> Digest {
    let mut chunks = (0..profile.reconstruction_threshold_cells()).map(|cell| cell_digests[cell * n_padded + row]);
    compression_chain_from_chunks(&mut chunks)
}

/// Encodes extension-field data and constructs V3's row digests and column-root commitment.
pub fn encode_and_commit(profile: ExtProfile, data: &ExtData) -> Result<(ExtCommitment, ExtAuxiliaryData), DemoError> {
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
        .collect::<Vec<_>>();
    let mut row_leaves = vec![zero; n_padded];
    row_leaves[..profile.n].copy_from_slice(&row_hashes);
    let row_root = merkle_layers(&row_leaves).last().unwrap()[0];
    let column_roots = (0..profile.n_cells())
        .map(|cell| {
            merkle_layers(&cell_digests[cell * n_padded..(cell + 1) * n_padded])
                .last()
                .unwrap()[0]
        })
        .collect::<Vec<_>>();
    let outer_merkle_layers = merkle_layers(&column_roots);
    let root_col = outer_merkle_layers.last().unwrap()[0];
    let commitment = ExtCommitment {
        profile,
        root: poseidon16_compress_pair(&row_root, &root_col),
    };
    Ok((
        commitment,
        ExtAuxiliaryData {
            profile,
            codewords,
            row_root,
            column_roots,
            outer_merkle_layers,
        },
    ))
}

fn ext_from_digest(digest: &Digest) -> EF {
    EF::from_basis_coefficients_slice(&digest[..EXT_DEGREE]).unwrap()
}

fn coeffs(value: EF) -> [F; EXT_DEGREE] {
    value.as_basis_coefficients_slice().try_into().unwrap()
}

fn fiat_shamir_digest(commitment: &ExtCommitment) -> Digest {
    let mut values = Vec::with_capacity(3 * DIGEST_LEN);
    values.extend_from_slice(&fs_block());
    values.extend_from_slice(&commitment.profile.profile_block());
    values.extend_from_slice(&commitment.root);
    poseidon_hash_slice(&values)
}

fn challenge(commitment: &ExtCommitment) -> EF {
    ext_from_digest(&fiat_shamir_digest(commitment))
}

fn batch_invert(values: &mut [EF]) {
    let mut accumulator = EF::ONE;
    let mut prefixes = Vec::with_capacity(values.len());
    for &value in values.iter() {
        prefixes.push(accumulator);
        accumulator *= value;
    }
    let mut inverse = accumulator.inverse();
    for (value, prefix) in values.iter_mut().zip(prefixes).rev() {
        let original = *value;
        *value = inverse * prefix;
        inverse *= original;
    }
}

/// Computes the physical-order public extension check vector used by dot_product_ee.
pub fn check_vector(commitment: &ExtCommitment) -> Option<ExtCheckVector> {
    let profile = commitment.profile;
    let omega = F::two_adic_generator(profile.m.ilog2() as usize);
    let omega_sq = omega.square();
    let p = challenge(commitment);
    let q = p / EF::from(omega);
    let h_inv = F::from_usize(profile.k).inverse();
    let common_p = (p.exp_u64(profile.k as u64) - EF::ONE) * EF::from(h_inv);
    let common_q = (q.exp_u64(profile.k as u64) - EF::ONE) * EF::from(h_inv);

    let mut xs = Vec::with_capacity(profile.k);
    let mut denominators = Vec::with_capacity(profile.m);
    let mut x = F::ONE;
    for _ in 0..profile.k {
        if p == EF::from(x) || q == EF::from(x) {
            return None;
        }
        xs.push(x);
        denominators.push(p - EF::from(x));
        denominators.push(q - EF::from(x));
        x *= omega_sq;
    }
    batch_invert(&mut denominators);
    let mut vector = vec![[F::ZERO; EXT_DEGREE]; profile.m];
    for (r, x) in xs.into_iter().enumerate() {
        vector[r] = coeffs(common_p * EF::from(x) * denominators[2 * r]);
        vector[profile.k + r] = coeffs(-(common_q * EF::from(x) * denominators[2 * r + 1]));
    }
    Some(vector)
}

fn guest_source() -> ProgramSource {
    ProgramSource::Raw(include_str!("../../zkdsl/v3_ext/full.py").to_string())
}

fn compilation_flags(commitment: &ExtCommitment) -> Result<CompilationFlags, DemoError> {
    commitment.profile.validate()?;
    let profile = commitment.profile;
    let root_ptr = DIGEST_LEN;
    let check_vector_ptr = root_ptr + DIGEST_LEN;
    let cell_base_len = profile.c * EXT_DEGREE;
    let mut replacements = std::collections::BTreeMap::new();
    for (name, value) in [
        ("N_PLACEHOLDER", profile.n),
        ("N_PADDED_PLACEHOLDER", padded_rows(profile)),
        ("LOG_N_PADDED_PLACEHOLDER", column_merkle_depth(profile)),
        ("M_EXT_PLACEHOLDER", profile.m),
        ("K_EXT_PLACEHOLDER", profile.k),
        ("C_EXT_PLACEHOLDER", profile.c),
        ("CELL_BASE_LEN_PLACEHOLDER", cell_base_len),
        ("N_CELLS_PLACEHOLDER", profile.n_cells()),
        ("SYSTEMATIC_CELLS_PLACEHOLDER", profile.reconstruction_threshold_cells()),
        ("CELL_CHUNKS_PLACEHOLDER", cell_base_len / DIGEST_LEN),
        ("OUTER_MERKLE_DEPTH_PLACEHOLDER", profile.merkle_depth()),
        ("OUTER_TREE_DIGESTS_PLACEHOLDER", 2 * profile.n_cells() - 1),
        ("PUBLIC_ROOT_PTR_PLACEHOLDER", root_ptr),
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

fn read_only_data(commitment: &ExtCommitment, check_vector: &ExtCheckVector) -> Vec<F> {
    let mut data = Vec::with_capacity(DIGEST_LEN + commitment.profile.m * EXT_DEGREE);
    data.extend_from_slice(&commitment.root);
    data.extend(check_vector.iter().flatten().copied());
    data
}

/// Recomputes Fiat-Shamir, generates physical-order L, and compiles the V3-ext guest.
pub fn prepare_statement(commitment: ExtCommitment) -> Result<ExtPreparedStatement, DemoError> {
    let check_vector = check_vector(&commitment).ok_or(DemoError::ChallengeOnDomain)?;
    let bytecode = compile_program_with_flags(&guest_source(), compilation_flags(&commitment)?)
        .with_read_only_data(read_only_data(&commitment, &check_vector));
    Ok(ExtPreparedStatement {
        commitment,
        check_vector,
        bytecode,
    })
}

fn witness(bytecode: &Bytecode, codewords: &ExtCodewords) -> ExecutionWitness {
    let flattened: Vec<_> = codewords
        .iter()
        .flat_map(|row| {
            row.iter()
                .flat_map(|value| value.as_basis_coefficients_slice().iter().copied())
        })
        .collect();
    let mut hints = Hints::default();
    hints.insert(bytecode, "codewords", arena_vec![ArenaVec::from_slice(&flattened)]);
    ExecutionWitness {
        hints,
        ..Default::default()
    }
}

/// Proves V3-ext's cell-first commitment and extension-field RS dot-product statement.
pub fn prove_codewords(prepared: &ExtPreparedStatement, codewords: &ExtCodewords) -> Result<ProofBundle, DemoError> {
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

/// Verifies a LeanVM proof against an already rebuilt V3-ext statement.
pub fn verify_prepared_execution_proof(prepared: &ExtPreparedStatement, proof: &ProofBundle) -> Result<(), DemoError> {
    verify_execution(
        &prepared.bytecode,
        &leanvm_public_input(),
        proof.execution.proof.clone(),
    )
    .map(|_| ())
    .map_err(DemoError::Verification)
}

/// Opens requested extension-field cell columns and attaches outer column-root paths.
pub fn query(aux: &ExtAuxiliaryData, indices: &[usize]) -> Result<ExtTranscript, DemoError> {
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
            .map(|row| ext_slice_to_base(&row[start..start + profile.c]))
            .collect();
        let mut node = index;
        let mut outer_authentication_path = Vec::with_capacity(profile.merkle_depth());
        for layer in aux.outer_merkle_layers.iter().take(profile.merkle_depth()) {
            outer_authentication_path.push(layer[node ^ 1]);
            node /= 2;
        }
        openings.push(ExtCellOpening {
            index,
            cells,
            outer_authentication_path,
            row_root: aux.row_root,
        });
    }
    Ok(ExtTranscript { openings })
}

/// Verifies extension-field opened cells by recomputing the inner column root and outer path.
pub fn verify_openings(commitment: &ExtCommitment, transcript: &ExtTranscript) -> bool {
    let profile = commitment.profile;
    let n_padded = padded_rows(profile);
    let zero = [F::ZERO; DIGEST_LEN];
    let expected_cell_len = profile.c * EXT_DEGREE;
    let mut seen = BTreeSet::new();
    transcript.openings.iter().all(|opening| {
        if opening.index >= profile.n_cells()
            || !seen.insert(opening.index)
            || opening.cells.len() != profile.n
            || opening.cells.iter().any(|cell| cell.len() != expected_cell_len)
            || opening.outer_authentication_path.len() != profile.merkle_depth()
        {
            return false;
        }
        let mut leaves = vec![zero; n_padded];
        for (row, cell) in opening.cells.iter().enumerate() {
            leaves[row] = fixed_compression_hash(cell);
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
        poseidon16_compress_pair(&opening.row_root, &digest) == commitment.root
    })
}

/// Reconstructs extension-field blobs after verifying enough distinct V3-ext cell columns.
pub fn reconstruct(commitment: &ExtCommitment, transcripts: &[ExtTranscript]) -> Result<ExtData, DemoError> {
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
    let decoder = ExtErasureDecoder::new(profile, &symbol_indices).ok_or(DemoError::ReconstructionFailed)?;

    (0..profile.n)
        .map(|row| {
            let mut values = Vec::with_capacity(symbol_indices.len());
            for &index in &indices {
                let opening = &openings[&index];
                values.extend(ext_slice_from_base(&opening.cells[row]).ok_or(DemoError::ReconstructionFailed)?);
            }
            decoder.reconstruct_blob(&values).ok_or(DemoError::ReconstructionFailed)
        })
        .collect()
}

/// Returns the byte size of queried indices, serialized extension cells, and outer paths.
pub fn transcript_size_bytes(transcript: &ExtTranscript) -> usize {
    transcript
        .openings
        .iter()
        .map(|opening| {
            size_of::<u32>()
                + opening.cells.iter().map(Vec::len).sum::<usize>() * size_of::<u32>()
                + opening.outer_authentication_path.len() * DIGEST_LEN * size_of::<u32>()
                + DIGEST_LEN * size_of::<u32>()
        })
        .sum()
}

/// Returns the public commitment size in canonical KoalaBear bytes.
pub fn commitment_size_bytes(_commitment: &ExtCommitment) -> usize {
    DIGEST_LEN * size_of::<u32>()
}

/// Deterministically generates extension-field input blobs for benchmarking.
pub fn demo_data(profile: ExtProfile) -> ExtData {
    (0..profile.n)
        .map(|row| {
            (0..profile.k)
                .map(|col| {
                    let base = 1 + row * profile.k + col;
                    let coords = [
                        F::from_usize(base),
                        F::from_usize(base + 17),
                        F::from_usize(base + 31),
                        F::from_usize(base + 47),
                        F::from_usize(base + 61),
                    ];
                    EF::from_basis_coefficients_slice(&coords).unwrap()
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_erasure_reconstruction_recovers_coefficients() {
        let profile = ExtProfile {
            name: "test-ext",
            n: 1,
            m: 16,
            k: 8,
            c: 8,
            whir_log_inv_rate: 1,
        };
        let data = demo_data(profile);
        let codewords = physical_codewords(profile, encode(profile, &data));
        let cell = 1;
        let physical_indices: Vec<_> = (0..profile.c).map(|offset| cell * profile.c + offset).collect();
        let logical_indices: Vec<_> = physical_indices
            .iter()
            .map(|&index| physical_to_logical(profile, index))
            .collect();
        let values: Vec<_> = physical_indices.iter().map(|&index| codewords[0][index]).collect();
        let decoder = ExtErasureDecoder::new(profile, &logical_indices).unwrap();
        assert_eq!(decoder.reconstruct_blob(&values).unwrap(), data[0]);
    }
}
