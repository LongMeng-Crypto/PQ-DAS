use std::time::Instant;

use backend::{PrimeCharacteristicRing, PrimeField32};
use clap::{Parser, ValueEnum};
use lean_vm::F;
use pq_das::{
    DIGEST_LEN, ParameterProfile, ProvedRelation, SAMPLING_SOUNDNESS_BITS, commitment_size_bytes, demo_data,
    encode_and_commit, prepare_relation_benchmark, prepare_statement, prove_codewords_with_profiling, query,
    reconstruct, sample_query_indices, transcript_size_bytes, verify_execution_proof, verify_openings,
    verify_relation_benchmark,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProfileName {
    Tiny,
    Medium,
    Large,
    Stress,
    #[value(name = "blob-128k-1")]
    Blob128K1,
    #[value(name = "blob-128k-4")]
    Blob128K4,
    #[value(name = "blob-128k-16")]
    Blob128K16,
    Custom,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RelationName {
    All,
    #[value(name = "row-hashes")]
    RowHashes,
    #[value(name = "column-merkle")]
    ColumnMerkle,
    #[value(name = "rs-membership")]
    RsMembership,
}

impl From<RelationName> for ProvedRelation {
    fn from(value: RelationName) -> Self {
        match value {
            RelationName::All => Self::All,
            RelationName::RowHashes => Self::RowHashes,
            RelationName::ColumnMerkle => Self::ColumnMerkle,
            RelationName::RsMembership => Self::RsMembership,
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Benchmark the parameterized PQ-DAS LeanVM construction")]
struct Cli {
    #[arg(long, value_enum, default_value_t = ProfileName::Tiny)]
    profile: ProfileName,

    #[arg(long)]
    n: Option<usize>,

    #[arg(long)]
    m: Option<usize>,

    #[arg(long)]
    k: Option<usize>,

    #[arg(long)]
    c: Option<usize>,

    #[arg(long, default_value_t = 1)]
    whir_log_inv_rate: usize,

    #[arg(long)]
    skip_reconstruction: bool,

    /// Enable LeanVM's function-level VM profiler.
    #[arg(long)]
    detailed_profiling: bool,

    /// Benchmark one proved relation in isolation; non-all modes are not production proofs.
    #[arg(long, value_enum, default_value_t = RelationName::All)]
    relation: RelationName,
}

impl Cli {
    /// Resolves a named or custom CLI selection into a validated parameter profile.
    fn selected_profile(&self) -> Result<ParameterProfile, Box<dyn std::error::Error>> {
        let mut profile = match self.profile {
            ProfileName::Tiny => ParameterProfile::TINY,
            ProfileName::Medium => ParameterProfile::MEDIUM,
            ProfileName::Large => ParameterProfile::LARGE,
            ProfileName::Stress => ParameterProfile::STRESS,
            ProfileName::Blob128K1 => ParameterProfile::BLOB_128K_1,
            ProfileName::Blob128K4 => ParameterProfile::BLOB_128K_4,
            ProfileName::Blob128K16 => ParameterProfile::BLOB_128K_16,
            ProfileName::Custom => ParameterProfile::custom(
                self.n.ok_or("custom profile requires --n")?,
                self.m.ok_or("custom profile requires --m")?,
                self.k.ok_or("custom profile requires --k")?,
                self.c.ok_or("custom profile requires --c")?,
                self.whir_log_inv_rate,
            )?,
        };
        profile.whir_log_inv_rate = self.whir_log_inv_rate;
        profile.validate()?;
        Ok(profile)
    }
}

/// Runs one complete benchmark for the selected generalized PQ-DAS profile.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::parallel::init();
    let cli = Cli::parse();
    let profile = cli.selected_profile()?;
    let data = demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = encode_and_commit(profile, &data)?;
    let commitment_time = started.elapsed();

    let started = Instant::now();
    let relation = ProvedRelation::from(cli.relation);
    let prepared = if relation == ProvedRelation::All {
        prepare_statement(commitment)?
    } else {
        prepare_relation_benchmark(commitment, relation)?
    };
    let preprocessing_time = started.elapsed();

    let started = Instant::now();
    let proof = prove_codewords_with_profiling(&prepared, &aux.codewords, cli.detailed_profiling)?;
    let proving_time = started.elapsed();

    let sample_count = profile.sampling_count(SAMPLING_SOUNDNESS_BITS);
    let indices = sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], sample_count)?;
    let transcript = query(&aux, &indices)?;

    let started = Instant::now();
    if relation == ProvedRelation::All {
        verify_execution_proof(&prepared.commitment, &proof)?;
    } else {
        verify_relation_benchmark(&prepared.commitment, &proof)?;
    }
    let proof_verification_time = started.elapsed();

    let started = Instant::now();
    let openings_accepted = verify_openings(&prepared.commitment, &transcript);
    let opening_verification_time = started.elapsed();
    let accepted = openings_accepted;
    let (reconstruction, reconstruction_time) = if cli.skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };
    let commitment_bytes = commitment_size_bytes(&prepared.commitment);
    let proof_field_elements = proof.execution.proof.proof_size_fe();
    let proof_bytes = proof_field_elements * size_of::<u32>();
    let sample_bytes = transcript_size_bytes(&transcript);

    println!("PQ-DAS LeanVM demo");
    println!("proved relation: {}", relation.name());
    if relation != ProvedRelation::All {
        println!("benchmark-only reduced relation: true");
    }
    println!(
        "profile: {} (n={}, m={}, k={}, rho={}/{}, c={}, cells={}, threshold={})",
        profile.name,
        profile.n,
        profile.m,
        profile.k,
        profile.k,
        profile.m,
        profile.c,
        profile.n_cells(),
        profile.reconstruction_threshold_cells(),
    );
    println!(
        "root: {:?}",
        prepared.commitment.root.map(|value| value.as_canonical_u32())
    );
    println!("proof accepted: {accepted}");
    println!("bytecode instructions: {}", prepared.bytecode.size());
    println!(
        "read-only public elements: {}",
        prepared.bytecode.read_only_data().len()
    );
    println!("sampling soundness target: {SAMPLING_SOUNDNESS_BITS} bits");
    println!(
        "sampling log2 failure bound: {:.3}",
        profile.sampling_log2_failure(sample_count)
    );
    println!("opened cells: {sample_count}");
    println!("commitment size: {commitment_bytes} bytes");
    println!("proof size: {proof_field_elements} field elements ({proof_bytes} bytes)");
    println!("sample size: {sample_bytes} bytes");
    match reconstruction {
        Some(correct) => println!("reconstruction correct: {correct}"),
        None => println!("reconstruction: skipped"),
    }
    if let Some(elapsed) = reconstruction_time {
        println!("reconstruction time: {:.3}s", elapsed.as_secs_f64());
    }
    println!("encoding + commitment time: {:.3}s", commitment_time.as_secs_f64());
    println!("prover preprocessing time: {:.3}s", preprocessing_time.as_secs_f64());
    println!("LeanVM proving time: {:.3}s", proving_time.as_secs_f64());
    println!(
        "verifier statement rebuild + LeanVM proof verification time: {:.3}s",
        proof_verification_time.as_secs_f64()
    );
    println!(
        "opening verification time: {:.3}s",
        opening_verification_time.as_secs_f64()
    );
    if let Some(metadata) = &proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("VM memory elements: {}", metadata.memory);
        println!("VM public memory elements: {}", metadata.public_memory_size);
        println!("VM runtime memory elements: {}", metadata.runtime_memory);
        println!("VM memory usage: {:.3}%", metadata.memory_usage_percent);
        println!("VM Poseidon16 calls: {}", metadata.n_poseidons);
        println!("VM extension-op calls: {}", metadata.n_extension_ops);
        if cli.detailed_profiling
            && let Some(report) = &metadata.profiling_report
        {
            println!("{report}");
        }
    }
    if let Some(profile) = &proof.execution.prover_profile {
        for table in &profile.tables {
            println!(
                "LeanVM table {}: actual_rows={}, padded_rows={}",
                table.name, table.actual_rows, table.padded_rows
            );
        }
        let timings = &profile.timings;
        for (name, elapsed) in [
            ("bytecode execution", timings.bytecode_execution),
            ("trace generation", timings.trace_generation),
            ("prover setup", timings.prover_setup),
            ("memory access count", timings.memory_access_count),
            ("bytecode access count", timings.bytecode_access_count),
            ("stack and commit", timings.stack_and_commit),
            ("logup", timings.logup),
            ("AIR preparation", timings.air_preparation),
            ("AIR sumcheck", timings.air_sumcheck),
            ("statement finalization", timings.statement_finalization),
            ("WHIR excluding grinding", timings.whir),
            ("grinding", timings.grinding),
        ] {
            println!("LeanVM prover stage {name}: {:.6}s", elapsed.as_secs_f64());
        }
    }
    Ok(())
}
