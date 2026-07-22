use std::{collections::BTreeSet, fmt::Display, time::Duration};

use backend::{
    Algebra, ArenaVec, BasedVectorSpace, Field, PrimeCharacteristicRing, PrimeField32, TwoAdicField, arena_vec,
    parallel, poseidon16_compress_pair,
};
use lean_compiler::{CompilationFlags, ProgramSource, compile_program_with_flags};
use lean_prover::{default_whir_config, prove_execution::prove_execution, verify_execution::verify_execution};
use lean_vm::{Bytecode, EF, ExecutionWitness, F, Hints};
use sha2::{Digest as ShaDigest, Sha256};

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
    pub const BLOB_EXT_2X_1: Self = Self {
        name: "blob-ext-2x-1",
        n: 1,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_2: Self = Self {
        name: "blob-ext-2x-2",
        n: 2,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_4: Self = Self {
        name: "blob-ext-2x-4",
        n: 4,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_6: Self = Self {
        name: "blob-ext-2x-6",
        n: 6,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_8: Self = Self {
        name: "blob-ext-2x-8",
        n: 8,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_10: Self = Self {
        name: "blob-ext-2x-10",
        n: 10,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_12: Self = Self {
        name: "blob-ext-2x-12",
        n: 12,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_14: Self = Self {
        name: "blob-ext-2x-14",
        n: 14,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_15: Self = Self {
        name: "blob-ext-2x-15",
        n: 15,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_16: Self = Self {
        name: "blob-ext-2x-16",
        n: 16,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_18: Self = Self {
        name: "blob-ext-2x-18",
        n: 18,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_20: Self = Self {
        name: "blob-ext-2x-20",
        n: 20,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_22: Self = Self {
        name: "blob-ext-2x-22",
        n: 22,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_24: Self = Self {
        name: "blob-ext-2x-24",
        n: 24,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_26: Self = Self {
        name: "blob-ext-2x-26",
        n: 26,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_28: Self = Self {
        name: "blob-ext-2x-28",
        n: 28,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_30: Self = Self {
        name: "blob-ext-2x-30",
        n: 30,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_32: Self = Self {
        name: "blob-ext-2x-32",
        n: 32,
        m: 32768,
        k: 16384,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C16_14: Self = Self {
        name: "blob-ext-2x-c16-14",
        n: 14,
        m: 32768,
        k: 16384,
        c: 16,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C64_1: Self = Self {
        name: "blob-ext-2x-c64-1",
        n: 1,
        m: 32768,
        k: 16384,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C64_14: Self = Self {
        name: "blob-ext-2x-c64-14",
        n: 14,
        m: 32768,
        k: 16384,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C64_16: Self = Self {
        name: "blob-ext-2x-c64-16",
        n: 16,
        m: 32768,
        k: 16384,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_1: Self = Self {
        name: "blob-ext-2x-c128-1",
        n: 1,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_14: Self = Self {
        name: "blob-ext-2x-c128-14",
        n: 14,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_15: Self = Self {
        name: "blob-ext-2x-c128-15",
        n: 15,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_16: Self = Self {
        name: "blob-ext-2x-c128-16",
        n: 16,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_24: Self = Self {
        name: "blob-ext-2x-c128-24",
        n: 24,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_26: Self = Self {
        name: "blob-ext-2x-c128-26",
        n: 26,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_28: Self = Self {
        name: "blob-ext-2x-c128-28",
        n: 28,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_2X_C128_30: Self = Self {
        name: "blob-ext-2x-c128-30",
        n: 30,
        m: 32768,
        k: 16384,
        c: 128,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_1: Self = Self {
        name: "blob-ext-4x-1",
        n: 1,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_2: Self = Self {
        name: "blob-ext-4x-2",
        n: 2,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_4: Self = Self {
        name: "blob-ext-4x-4",
        n: 4,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_6: Self = Self {
        name: "blob-ext-4x-6",
        n: 6,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_8: Self = Self {
        name: "blob-ext-4x-8",
        n: 8,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_10: Self = Self {
        name: "blob-ext-4x-10",
        n: 10,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_12: Self = Self {
        name: "blob-ext-4x-12",
        n: 12,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_14: Self = Self {
        name: "blob-ext-4x-14",
        n: 14,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_16: Self = Self {
        name: "blob-ext-4x-16",
        n: 16,
        m: 65536,
        k: 32768,
        c: 64,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C16_14: Self = Self {
        name: "blob-ext-4x-c16-14",
        n: 14,
        m: 65536,
        k: 32768,
        c: 16,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_1: Self = Self {
        name: "blob-ext-4x-c32-1",
        n: 1,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_2: Self = Self {
        name: "blob-ext-4x-c32-2",
        n: 2,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_4: Self = Self {
        name: "blob-ext-4x-c32-4",
        n: 4,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_6: Self = Self {
        name: "blob-ext-4x-c32-6",
        n: 6,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_8: Self = Self {
        name: "blob-ext-4x-c32-8",
        n: 8,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_10: Self = Self {
        name: "blob-ext-4x-c32-10",
        n: 10,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_12: Self = Self {
        name: "blob-ext-4x-c32-12",
        n: 12,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_14: Self = Self {
        name: "blob-ext-4x-c32-14",
        n: 14,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_16: Self = Self {
        name: "blob-ext-4x-c32-16",
        n: 16,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_18: Self = Self {
        name: "blob-ext-4x-c32-18",
        n: 18,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_20: Self = Self {
        name: "blob-ext-4x-c32-20",
        n: 20,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_22: Self = Self {
        name: "blob-ext-4x-c32-22",
        n: 22,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_24: Self = Self {
        name: "blob-ext-4x-c32-24",
        n: 24,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_26: Self = Self {
        name: "blob-ext-4x-c32-26",
        n: 26,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_28: Self = Self {
        name: "blob-ext-4x-c32-28",
        n: 28,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_30: Self = Self {
        name: "blob-ext-4x-c32-30",
        n: 30,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C32_32: Self = Self {
        name: "blob-ext-4x-c32-32",
        n: 32,
        m: 65536,
        k: 32768,
        c: 32,
        whir_log_inv_rate: 1,
    };
    pub const BLOB_EXT_4X_C128_14: Self = Self {
        name: "blob-ext-4x-c128-14",
        n: 14,
        m: 65536,
        k: 32768,
        c: 128,
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
    pub row_hashes: Vec<Digest>,
    pub root_col: Digest,
    /// Cached final root derived from the public row hashes and column root.
    pub root: Digest,
}

impl ExtCommitment {
    fn recompute_row_root(&self) -> Option<Digest> {
        if self.row_hashes.len() != self.profile.n {
            return None;
        }
        let zero = [F::ZERO; DIGEST_LEN];
        let mut row_leaves = vec![zero; padded_rows(self.profile)];
        row_leaves[..self.profile.n].copy_from_slice(&self.row_hashes);
        Some(merkle_root_no_layers(&row_leaves))
    }

    fn recompute_root(&self) -> Option<Digest> {
        Some(poseidon16_compress_pair(&self.recompute_row_root()?, &self.root_col))
    }

    fn normalize(mut self) -> Result<Self, DemoError> {
        self.root = self.recompute_root().ok_or(DemoError::InvalidDataShape)?;
        Ok(self)
    }
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
    pub column_roots: Vec<Digest>,
    pub outer_merkle_layers: Vec<Vec<Digest>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtCellOpening {
    pub index: usize,
    pub cells: Vec<Vec<F>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtTranscript {
    pub openings: Vec<ExtCellOpening>,
    pub outer_multiproof: Vec<Digest>,
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
    let root = commitment.recompute_root().ok_or(DemoError::InvalidDataShape)?;
    let mut state = poseidon16_compress_pair(&root, randomness);
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

fn logical_to_physical(profile: ExtProfile, index: usize) -> usize {
    if index.is_multiple_of(2) {
        index / 2
    } else {
        profile.k + index / 2
    }
}

fn physical_to_logical(profile: ExtProfile, index: usize) -> usize {
    if index < profile.k {
        2 * index
    } else {
        2 * (index - profile.k) + 1
    }
}

fn logical_to_physical_in_place(profile: ExtProfile, codeword: &mut [EF]) {
    debug_assert_eq!(codeword.len(), profile.m);
    let mut visited = vec![false; profile.m];
    for start in 0..profile.m {
        if visited[start] {
            continue;
        }
        let mut current = start;
        let mut carry = codeword[current];
        loop {
            visited[current] = true;
            let next = logical_to_physical(profile, current);
            carry = std::mem::replace(&mut codeword[next], carry);
            current = next;
            if current == start {
                break;
            }
        }
    }
}

/// Encodes extension-field coefficients and leaves the codeword in physical cell order.
pub fn encode_blob(profile: ExtProfile, blob: &[EF]) -> ExtCodeword {
    assert_eq!(blob.len(), profile.k);
    let mut codeword = vec![EF::ZERO; profile.m];
    codeword[..profile.k].copy_from_slice(blob);
    fft(&mut codeword);
    logical_to_physical_in_place(profile, &mut codeword);
    codeword
}

/// RS-encodes all extension-field blobs. Rows are independent, so the host preparation runs them in parallel.
pub fn encode(profile: ExtProfile, data: &ExtData) -> ExtCodewords {
    assert_eq!(data.len(), profile.n);
    parallel::par_map_collect(data.len(), |row| encode_blob(profile, &data[row]))
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

/// Hashes one extension-field cell by streaming its KoalaBear coordinates without allocating.
pub fn cell_hash(cell: &[EF]) -> Digest {
    debug_assert!(!cell.is_empty());
    debug_assert!((cell.len() * EXT_DEGREE).is_multiple_of(DIGEST_LEN));
    let zero = [F::ZERO; DIGEST_LEN];
    let mut first = [F::ZERO; DIGEST_LEN];
    let mut chunk = [F::ZERO; DIGEST_LEN];
    let mut chunk_pos = 0usize;
    let mut chunk_count = 0usize;
    let mut state = [F::ZERO; DIGEST_LEN];

    for value in cell {
        for &coord in value.as_basis_coefficients_slice() {
            chunk[chunk_pos] = coord;
            chunk_pos += 1;
            if chunk_pos == DIGEST_LEN {
                if chunk_count == 0 {
                    first = chunk;
                } else if chunk_count == 1 {
                    state = poseidon16_compress_pair(&first, &chunk);
                } else {
                    state = poseidon16_compress_pair(&state, &chunk);
                }
                chunk = [F::ZERO; DIGEST_LEN];
                chunk_pos = 0;
                chunk_count += 1;
            }
        }
    }

    debug_assert_eq!(chunk_pos, 0);
    match chunk_count {
        0 => unreachable!("non-empty cell must produce at least one chunk"),
        1 => poseidon16_compress_pair(&zero, &first),
        _ => state,
    }
}

fn merkle_root_16(leaves: &[Digest]) -> Digest {
    debug_assert_eq!(leaves.len(), 16);
    let mut level_8 = [[F::ZERO; DIGEST_LEN]; 8];
    for node in 0..8 {
        level_8[node] = poseidon16_compress_pair(&leaves[2 * node], &leaves[2 * node + 1]);
    }
    let mut level_4 = [[F::ZERO; DIGEST_LEN]; 4];
    for node in 0..4 {
        level_4[node] = poseidon16_compress_pair(&level_8[2 * node], &level_8[2 * node + 1]);
    }
    let mut level_2 = [[F::ZERO; DIGEST_LEN]; 2];
    for node in 0..2 {
        level_2[node] = poseidon16_compress_pair(&level_4[2 * node], &level_4[2 * node + 1]);
    }
    poseidon16_compress_pair(&level_2[0], &level_2[1])
}

fn merkle_root_no_layers(leaves: &[Digest]) -> Digest {
    assert!(leaves.len().is_power_of_two());
    if leaves.len() == 16 {
        return merkle_root_16(leaves);
    }
    let mut layer = leaves.to_vec();
    let mut len = layer.len();
    while len > 1 {
        for node in 0..len / 2 {
            layer[node] = poseidon16_compress_pair(&layer[2 * node], &layer[2 * node + 1]);
        }
        len /= 2;
    }
    layer[0]
}

fn outer_merkle_multiproof(layers: &[Vec<Digest>], indices: &[usize]) -> Vec<Digest> {
    let mut nodes: BTreeSet<usize> = indices.iter().copied().collect();
    let mut proof = Vec::new();
    for layer in layers.iter().take(layers.len() - 1) {
        for &node in &nodes {
            let sibling = node ^ 1;
            if !nodes.contains(&sibling) {
                proof.push(layer[sibling]);
            }
        }
        nodes = nodes.into_iter().map(|node| node / 2).collect();
    }
    proof
}

fn verify_outer_merkle_multiproof(leaf_count: usize, leaves: &[(usize, Digest)], proof: &[Digest]) -> Option<Digest> {
    if !leaf_count.is_power_of_two() || leaves.is_empty() {
        return None;
    }
    let mut nodes = std::collections::BTreeMap::new();
    for &(index, digest) in leaves {
        if index >= leaf_count || nodes.insert(index, digest).is_some() {
            return None;
        }
    }
    let mut proof_pos = 0usize;
    let mut width = leaf_count;
    while width > 1 {
        let current = std::mem::take(&mut nodes);
        let mut next = std::collections::BTreeMap::new();
        for (&node, &digest) in &current {
            if node % 2 == 1 && current.contains_key(&(node ^ 1)) {
                continue;
            }
            let sibling_node = node ^ 1;
            let sibling = if let Some(&sibling) = current.get(&sibling_node) {
                sibling
            } else {
                let sibling = *proof.get(proof_pos)?;
                proof_pos += 1;
                sibling
            };
            let parent = if node.is_multiple_of(2) {
                poseidon16_compress_pair(&digest, &sibling)
            } else {
                poseidon16_compress_pair(&sibling, &digest)
            };
            if next.insert(node / 2, parent).is_some() {
                return None;
            }
        }
        nodes = next;
        width /= 2;
    }
    if proof_pos == proof.len() {
        nodes.remove(&0)
    } else {
        None
    }
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
    let codewords = encode(profile, data);
    let n_padded = padded_rows(profile);
    let zero = [F::ZERO; DIGEST_LEN];
    let mut cell_digests = vec![zero; profile.n_cells() * n_padded];
    parallel::par_chunks_mut(&mut cell_digests, n_padded, |cell, leaves| {
        let start = cell * profile.c;
        for row in 0..profile.n {
            leaves[row] = cell_hash(&codewords[row][start..start + profile.c]);
        }
    });
    let row_hashes = parallel::par_map_collect(profile.n, |row| {
        row_hash_from_cell_digests(profile, n_padded, &cell_digests, row)
    });
    let mut row_leaves = vec![zero; n_padded];
    row_leaves[..profile.n].copy_from_slice(&row_hashes);
    let row_root = merkle_root_no_layers(&row_leaves);
    let column_roots = parallel::par_map_collect(profile.n_cells(), |cell| {
        merkle_root_no_layers(&cell_digests[cell * n_padded..(cell + 1) * n_padded])
    });
    let outer_merkle_layers = merkle_layers(&column_roots);
    let root_col = outer_merkle_layers.last().unwrap()[0];
    let commitment = ExtCommitment {
        profile,
        row_hashes,
        root_col,
        root: poseidon16_compress_pair(&row_root, &root_col),
    };
    Ok((
        commitment,
        ExtAuxiliaryData {
            profile,
            codewords,
            column_roots,
            outer_merkle_layers,
        },
    ))
}

const FS_SHA256_DOMAIN: &[u8] = b"PQ-DAS-SHA256-RS-CHECK-v1";

fn coeffs(value: EF) -> [F; EXT_DEGREE] {
    value.as_basis_coefficients_slice().try_into().unwrap()
}

fn push_fs_field(bytes: &mut Vec<u8>, value: F) {
    bytes.extend_from_slice(&value.as_canonical_u32().to_le_bytes());
}

fn push_fs_block(bytes: &mut Vec<u8>, block: &[F; DIGEST_LEN]) {
    for &value in block {
        push_fs_field(bytes, value);
    }
}

fn fiat_shamir_transcript(commitment: &ExtCommitment) -> Vec<u8> {
    let mut bytes =
        Vec::with_capacity(FS_SHA256_DOMAIN.len() + (2 * DIGEST_LEN + commitment.root.len()) * size_of::<u32>());
    bytes.extend_from_slice(FS_SHA256_DOMAIN);
    push_fs_block(&mut bytes, &fs_block());
    push_fs_block(&mut bytes, &commitment.profile.profile_block());
    push_fs_block(&mut bytes, &commitment.root);
    bytes
}

fn sha256_field_elements(transcript: &[u8], count: usize) -> Vec<F> {
    let mut out = Vec::with_capacity(count);
    let mut counter = 0u32;
    while out.len() < count {
        let mut hasher = Sha256::new();
        hasher.update(FS_SHA256_DOMAIN);
        hasher.update(b":expand");
        hasher.update(counter.to_le_bytes());
        hasher.update(transcript);
        let digest = hasher.finalize();
        for chunk in digest.chunks_exact(size_of::<u32>()) {
            let candidate = u32::from_le_bytes(chunk.try_into().unwrap());
            if candidate < F::ORDER_U32 {
                out.push(F::from_u32(candidate));
                if out.len() == count {
                    break;
                }
            }
        }
        counter = counter.checked_add(1).expect("SHA-256 Fiat-Shamir counter overflow");
    }
    out
}

fn challenge(commitment: &ExtCommitment) -> EF {
    let transcript = fiat_shamir_transcript(commitment);
    let coeffs: [F; EXT_DEGREE] = sha256_field_elements(&transcript, EXT_DEGREE).try_into().unwrap();
    EF::from_basis_coefficients_slice(&coeffs).unwrap()
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
    Some(parallel::par_map_collect(profile.m, |index| {
        if index < profile.k {
            let x = EF::from(xs[index]);
            coeffs(common_p * x * denominators[2 * index])
        } else {
            let r = index - profile.k;
            let x = EF::from(xs[r]);
            coeffs(-(common_q * x * denominators[2 * r + 1]))
        }
    }))
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
        ("PUBLIC_ROOT_PTR_PLACEHOLDER", root_ptr),
        ("CHECK_VECTOR_PTR_PLACEHOLDER", check_vector_ptr),
    ] {
        replacements.insert(name.to_string(), value.to_string());
    }
    Ok(CompilationFlags { replacements })
}

fn leanvm_public_input() -> [F; DIGEST_LEN] {
    [F::ZERO; DIGEST_LEN]
}

fn read_only_data(commitment: &ExtCommitment, check_vector: &ExtCheckVector) -> Vec<F> {
    let mut data = vec![F::ZERO; DIGEST_LEN + commitment.profile.m * EXT_DEGREE];
    data[..DIGEST_LEN].copy_from_slice(&commitment.root);
    parallel::par_chunks_mut(&mut data[DIGEST_LEN..], EXT_DEGREE, |index, slot| {
        slot.copy_from_slice(&check_vector[index]);
    });
    data
}

/// Recomputes Fiat-Shamir, generates physical-order L, and compiles the V3-ext guest.
pub fn prepare_statement(commitment: ExtCommitment) -> Result<ExtPreparedStatement, DemoError> {
    let commitment = commitment.normalize()?;
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

/// Opens requested extension-field cell columns and attaches one shared outer Merkle multiproof.
pub fn query(aux: &ExtAuxiliaryData, indices: &[usize]) -> Result<ExtTranscript, DemoError> {
    let profile = aux.profile;
    let mut seen = BTreeSet::new();
    for &index in indices {
        if index >= profile.n_cells() || !seen.insert(index) {
            return Err(DemoError::InvalidQuery);
        }
    }
    let openings = parallel::par_map_collect(indices.len(), |i| {
        let index = indices[i];
        let start = index * profile.c;
        let cells = aux
            .codewords
            .iter()
            .map(|row| ext_slice_to_base(&row[start..start + profile.c]))
            .collect();
        ExtCellOpening { index, cells }
    });
    let outer_multiproof = outer_merkle_multiproof(&aux.outer_merkle_layers, indices);
    Ok(ExtTranscript {
        openings,
        outer_multiproof,
    })
}

/// Verifies extension-field opened cells by recomputing inner column roots and one outer multiproof.
pub fn verify_openings(commitment: &ExtCommitment, transcript: &ExtTranscript) -> bool {
    let profile = commitment.profile;
    let n_padded = padded_rows(profile);
    let zero = [F::ZERO; DIGEST_LEN];
    let expected_cell_len = profile.c * EXT_DEGREE;
    let mut seen = BTreeSet::new();
    for opening in &transcript.openings {
        if opening.index >= profile.n_cells()
            || !seen.insert(opening.index)
            || opening.cells.len() != profile.n
            || opening.cells.iter().any(|cell| cell.len() != expected_cell_len)
        {
            return false;
        }
    }
    let column_roots = parallel::par_map_collect(transcript.openings.len(), |i| {
        let opening = &transcript.openings[i];
        let mut leaves = vec![zero; n_padded];
        for (row, cell) in opening.cells.iter().enumerate() {
            leaves[row] = fixed_compression_hash(cell);
        }
        (opening.index, merkle_root_no_layers(&leaves))
    });
    let Some(root_col) = verify_outer_merkle_multiproof(profile.n_cells(), &column_roots, &transcript.outer_multiproof)
    else {
        return false;
    };
    root_col == commitment.root_col && commitment.recompute_root().is_some_and(|root| root == commitment.root)
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

    parallel::par_map_collect(profile.n, |row| {
        let mut values = Vec::with_capacity(symbol_indices.len());
        for &index in &indices {
            let opening = &openings[&index];
            values.extend(ext_slice_from_base(&opening.cells[row]).ok_or(DemoError::ReconstructionFailed)?);
        }
        decoder.reconstruct_blob(&values).ok_or(DemoError::ReconstructionFailed)
    })
    .into_iter()
    .collect()
}

/// Returns canonical bytes for queried indices, serialized cells, and shared outer multiproof.
pub fn transcript_size_bytes(transcript: &ExtTranscript) -> usize {
    transcript
        .openings
        .iter()
        .map(|opening| size_of::<u32>() + opening.cells.iter().map(Vec::len).sum::<usize>() * size_of::<u32>())
        .sum::<usize>()
        + transcript.outer_multiproof.len() * DIGEST_LEN * size_of::<u32>()
}

/// Returns the public commitment size: all row hashes plus the column root.
pub fn commitment_size_bytes(commitment: &ExtCommitment) -> usize {
    (commitment.row_hashes.len() * DIGEST_LEN + DIGEST_LEN) * size_of::<u32>()
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
        let codewords = encode(profile, &data);
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
