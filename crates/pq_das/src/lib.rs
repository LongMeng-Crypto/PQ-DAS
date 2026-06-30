mod config;
pub mod dbp_ext;
pub mod encoding;
mod hashing;
mod membership;
pub mod v2_base;
pub mod v2_ext;

use std::fmt::{Display, Formatter};

use lean_prover::prove_execution::ExecutionProof;
use lean_vm::Bytecode;

pub use config::*;
pub use encoding::{Blob, Codeword, Codewords, Data, ErasureDecoder, demo_data, encode, encode_blob};
pub use hashing::Digest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commitment {
    pub profile: ParameterProfile,
    pub row_hashes: Vec<Digest>,
    pub root: Digest,
}

#[derive(Debug, Clone)]
pub struct ProofBundle {
    pub execution: ExecutionProof,
}

#[derive(Debug, Clone)]
pub struct PreparedStatement {
    /// Public profile, row hashes, and column root represented by this statement.
    pub commitment: Commitment,
    /// Public special-barycentric vector generated during statement preparation.
    pub check_vector: membership::CheckVector,
    /// Profile bytecode with the statement values bound in read-only memory.
    pub bytecode: Bytecode,
}

#[derive(Debug)]
pub enum DemoError {
    InvalidDataShape,
    InvalidQuery,
    InvalidOpening,
    InsufficientCells,
    ReconstructionFailed,
    ChallengeOnDomain,
    Profile(ProfileError),
    Prover(lean_prover::ProverError),
    Verification(backend::ProofError),
}

impl Display for DemoError {
    /// Formats each demo failure as a concise user-facing diagnostic.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDataShape => write!(f, "data dimensions do not match the selected profile"),
            Self::InvalidQuery => write!(f, "query contains an invalid or duplicate cell index"),
            Self::InvalidOpening => write!(f, "invalid cell opening"),
            Self::InsufficientCells => write!(f, "not enough distinct cells to reconstruct"),
            Self::ReconstructionFailed => write!(f, "RS reconstruction failed"),
            Self::ChallengeOnDomain => write!(f, "Fiat-Shamir challenge lies on the interpolation domain"),
            Self::Profile(err) => write!(f, "invalid parameter profile: {err}"),
            Self::Prover(err) => write!(f, "LeanVM prover failed: {err}"),
            Self::Verification(err) => write!(f, "LeanVM verification failed: {err}"),
        }
    }
}

impl std::error::Error for DemoError {}

impl From<ProfileError> for DemoError {
    /// Converts profile validation failures into the demo's unified error type.
    fn from(value: ProfileError) -> Self {
        Self::Profile(value)
    }
}

impl From<lean_prover::ProverError> for DemoError {
    /// Converts LeanVM prover failures into the demo's unified error type.
    fn from(value: lean_prover::ProverError) -> Self {
        Self::Prover(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Confirms all V2 base-field profiles satisfy the implementation constraints.
    fn predefined_base_profiles_are_valid() {
        for profile in [
            ParameterProfile::TINY,
            ParameterProfile::MEDIUM,
            ParameterProfile::LARGE,
            ParameterProfile::STRESS,
            ParameterProfile::BLOB_128K_1,
            ParameterProfile::BLOB_128K_4,
            ParameterProfile::BLOB_128K_14,
            ParameterProfile::BLOB_128K_16,
        ] {
            profile.validate().unwrap();
        }
    }

    #[test]
    /// Pins the currently configured V2 base-field benchmark query count.
    fn v2_base_opened_cells_meets_target() {
        for profile in [
            ParameterProfile::BLOB_128K_1,
            ParameterProfile::BLOB_128K_14,
            ParameterProfile::BLOB_128K_16,
        ] {
            assert!(v2_base::subset_log2_failure(profile, v2_base::V2_OPENED_CELLS) <= -40.0);
        }
    }
}
