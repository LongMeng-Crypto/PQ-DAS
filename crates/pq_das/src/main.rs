use std::time::{Duration, Instant};

use backend::{PrimeCharacteristicRing, PrimeField32};
use clap::{Parser, ValueEnum};
use lean_vm::F;
use pq_das::{
    DIGEST_LEN, ParameterProfile, SAMPLING_SOUNDNESS_BITS, SAMPLING_TRANSCRIPTS, commitment_size_bytes, demo_data,
    encode_and_commit, prepare_statement, prove_codewords, query, reconstruct, sample_query_indices,
    transcript_size_bytes, v2, v2_ext, verify_execution_proof, verify_openings,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum VersionName {
    V1,
    V2,
    #[value(name = "v2-ext")]
    V2Ext,
}

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
    #[value(name = "blob-128k-14")]
    Blob128K14,
    #[value(name = "blob-128k-16")]
    Blob128K16,
    #[value(name = "blob-ext-1")]
    BlobExt1,
    #[value(name = "blob-ext-14")]
    BlobExt14,
    #[value(name = "blob-ext-16")]
    BlobExt16,
    Custom,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum V2RelationName {
    Full,
    #[value(name = "row-hash-only")]
    RowHashOnly,
    #[value(name = "cell-commit-only")]
    CellCommitOnly,
    #[value(name = "membership-only")]
    MembershipOnly,
}

impl From<V2RelationName> for v2::Relation {
    fn from(value: V2RelationName) -> Self {
        match value {
            V2RelationName::Full => Self::Full,
            V2RelationName::RowHashOnly => Self::RowHashOnly,
            V2RelationName::CellCommitOnly => Self::CellCommitOnly,
            V2RelationName::MembershipOnly => Self::MembershipOnly,
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Benchmark the parameterized PQ-DAS LeanVM construction")]
struct Cli {
    #[arg(long, value_enum, default_value_t = VersionName::V1)]
    version: VersionName,

    #[arg(long, value_enum, default_value_t = ProfileName::Tiny)]
    profile: ProfileName,

    #[arg(long, help = "Run blob-128k-1, blob-128k-14, and blob-128k-16 under V2")]
    all_v2_benchmarks: bool,

    #[arg(long, help = "Run blob-ext-1, blob-ext-14, and blob-ext-16 under V2-ext")]
    all_v2_ext_benchmarks: bool,

    #[arg(long, value_enum, default_value_t = V2RelationName::Full)]
    v2_relation: V2RelationName,

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
            ProfileName::Blob128K14 => ParameterProfile::BLOB_128K_14,
            ProfileName::Blob128K16 => ParameterProfile::BLOB_128K_16,
            ProfileName::BlobExt1 | ProfileName::BlobExt14 | ProfileName::BlobExt16 => {
                return Err("extension profiles require --version v2-ext".into());
            }
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

    /// Resolves a named extension-field profile.
    fn selected_ext_profile(&self) -> Result<v2_ext::ExtProfile, Box<dyn std::error::Error>> {
        let mut profile = match self.profile {
            ProfileName::BlobExt1 => v2_ext::ExtProfile::BLOB_EXT_1,
            ProfileName::BlobExt14 => v2_ext::ExtProfile::BLOB_EXT_14,
            ProfileName::BlobExt16 => v2_ext::ExtProfile::BLOB_EXT_16,
            _ => return Err("v2-ext requires --profile blob-ext-1, blob-ext-14, or blob-ext-16".into()),
        };
        profile.whir_log_inv_rate = self.whir_log_inv_rate;
        profile.validate()?;
        Ok(profile)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::parallel::init();
    let cli = Cli::parse();
    if cli.all_v2_benchmarks {
        run_all_v2_benchmarks(cli.skip_reconstruction, cli.v2_relation.into())?;
        return Ok(());
    }
    if cli.all_v2_ext_benchmarks {
        run_all_v2_ext_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }

    match cli.version {
        VersionName::V1 => run_v1_single(cli.selected_profile()?, cli.skip_reconstruction)?,
        VersionName::V2 => run_v2_single(cli.selected_profile()?, cli.skip_reconstruction, cli.v2_relation.into())?,
        VersionName::V2Ext => run_v2_ext_single(cli.selected_ext_profile()?, cli.skip_reconstruction)?,
    }
    Ok(())
}

/// Runs the original V1 benchmark path.
fn run_v1_single(profile: ParameterProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let data = demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = encode_and_commit(profile, &data)?;
    let commitment_time = started.elapsed();

    let started = Instant::now();
    let prepared = prepare_statement(commitment)?;
    let preprocessing_time = started.elapsed();

    let started = Instant::now();
    let proof = prove_codewords(&prepared, &aux.codewords)?;
    let proving_time = started.elapsed();

    let sample_count = profile.sampling_count(SAMPLING_SOUNDNESS_BITS);
    let indices = sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], sample_count)?;
    let transcript = query(&aux, &indices)?;

    let started = Instant::now();
    verify_execution_proof(&prepared.commitment, &proof)?;
    let proof_verification_time = started.elapsed();

    let started = Instant::now();
    let openings_accepted = verify_openings(&prepared.commitment, &transcript);
    let opening_verification_time = started.elapsed();
    let (reconstruction, reconstruction_time) = if skip_reconstruction {
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
    print_v1_report(
        profile,
        &prepared,
        &proof,
        &transcript,
        openings_accepted,
        reconstruction,
        reconstruction_time,
        commitment_time,
        preprocessing_time,
        proving_time,
        proof_verification_time,
        opening_verification_time,
    );
    Ok(())
}

/// Runs one V2 benchmark and prints the same detailed report plus VM counters.
fn run_v2_single(
    profile: ParameterProfile,
    skip_reconstruction: bool,
    relation: v2::Relation,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v2_benchmark(profile, skip_reconstruction, relation)?;
    print_v2_report(&result);
    Ok(())
}

/// Runs and prints the requested V2 benchmark table.
fn run_all_v2_benchmarks(skip_reconstruction: bool, relation: v2::Relation) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        ParameterProfile::BLOB_128K_1,
        ParameterProfile::BLOB_128K_14,
        ParameterProfile::BLOB_128K_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v2_benchmark(profile, skip_reconstruction, relation)?);
    }
    print_v2_table(&results);
    Ok(())
}

/// Runs and prints the requested V2-ext benchmark table.
fn run_all_v2_ext_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v2_ext::ExtProfile::BLOB_EXT_1,
        v2_ext::ExtProfile::BLOB_EXT_14,
        v2_ext::ExtProfile::BLOB_EXT_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v2_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v2_ext_table(&results);
    Ok(())
}

/// Runs one V2-ext benchmark and prints the same detailed report plus VM counters.
fn run_v2_ext_single(profile: v2_ext::ExtProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v2_ext_benchmark(profile, skip_reconstruction)?;
    print_v2_ext_report(&result);
    Ok(())
}

fn run_v2_benchmark(
    profile: ParameterProfile,
    skip_reconstruction: bool,
    relation: v2::Relation,
) -> Result<v2::BenchmarkResult, Box<dyn std::error::Error>> {
    let data = demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v2::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v2::prepare_statement_with_relation(commitment, relation)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v2::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v2::V2_OPENED_CELLS.min(profile.n_cells());
    let started = Instant::now();
    let indices = sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v2::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v2::prepare_statement_with_relation(prepared.commitment.clone(), relation)?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v2::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v2::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v2::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v2::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };
    Ok(v2::BenchmarkResult {
        relation,
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v2::BenchmarkTimings {
            encode_commit,
            prover_preprocess,
            prove,
            opening_generation,
            verifier_rebuild,
            proof_verify,
            verify_openings,
            reconstruct: reconstruct_time,
        },
        accepted: opening_accepted,
    })
}

fn run_v2_ext_benchmark(
    profile: v2_ext::ExtProfile,
    skip_reconstruction: bool,
) -> Result<v2_ext::ExtBenchmarkResult, Box<dyn std::error::Error>> {
    let data = v2_ext::demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v2_ext::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v2_ext::prepare_statement(commitment)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v2_ext::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v2_ext::opened_cells(profile).min(profile.n_cells());
    let started = Instant::now();
    let indices = v2_ext::sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v2_ext::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v2_ext::prepare_statement(prepared.commitment.clone())?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v2_ext::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v2_ext::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = v2_ext::sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v2_ext::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v2_ext::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };

    Ok(v2_ext::ExtBenchmarkResult {
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v2_ext::ExtBenchmarkTimings {
            encode_commit,
            prover_preprocess,
            prove,
            opening_generation,
            verifier_rebuild,
            proof_verify,
            verify_openings,
            reconstruct: reconstruct_time,
        },
        accepted: opening_accepted,
    })
}

#[allow(clippy::too_many_arguments)]
fn print_v1_report(
    profile: ParameterProfile,
    prepared: &pq_das::PreparedStatement,
    proof: &pq_das::ProofBundle,
    transcript: &pq_das::Transcript,
    accepted: bool,
    reconstruction: Option<bool>,
    reconstruction_time: Option<Duration>,
    commitment_time: Duration,
    preprocessing_time: Duration,
    proving_time: Duration,
    proof_verification_time: Duration,
    opening_verification_time: Duration,
) {
    let commitment_bytes = commitment_size_bytes(&prepared.commitment);
    let proof_field_elements = proof.execution.proof.proof_size_fe();
    let proof_bytes = proof_field_elements * size_of::<u32>();
    let sample_bytes = transcript_size_bytes(transcript);
    println!("PQ-DAS V1 LeanVM demo");
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
    println!("sampling transcripts assumed: {SAMPLING_TRANSCRIPTS}");
    println!(
        "sampling log2 failure bound: {:.3}",
        profile.sampling_log2_failure(profile.sampling_count(SAMPLING_SOUNDNESS_BITS))
    );
    println!("opened cells: {}", profile.sampling_count(SAMPLING_SOUNDNESS_BITS));
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
}

fn print_v2_report(result: &v2::BenchmarkResult) {
    println!("PQ-DAS V2 LeanVM demo");
    println!("{}", v2_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v2_table(results: &[v2::BenchmarkResult]) {
    println!("PQ-DAS V2 LeanVM benchmark table");
    println!(
        "| Profile | Relation | Bytecode instructions | Read-only elements | Opened cells | $\\log_2\\nu_{{\\mathrm{{wor}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Result |"
    );
    println!(
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v2_row(result));
    }
}

fn print_v2_ext_report(result: &v2_ext::ExtBenchmarkResult) {
    println!("PQ-DAS V2-ext LeanVM demo");
    println!("{}", v2_ext_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v2_ext_table(results: &[v2_ext::ExtBenchmarkResult]) {
    println!("PQ-DAS V2-ext LeanVM benchmark table");
    println!(
        "| Profile | Bytecode instructions | Read-only elements | Opened cells | $\\log_2\\nu_{{\\mathrm{{rep}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v2_ext_row(result));
    }
}

fn v2_row(result: &v2::BenchmarkResult) -> String {
    let profile = result.profile;
    let proof_field_elements = result.proof.execution.proof.proof_size_fe();
    let proof_bytes = proof_field_elements * size_of::<u32>();
    let metadata = result.proof.execution.metadata.as_ref();
    let reconstruction = match result.reconstruction {
        Some(true) => format_duration(result.timings.reconstruct),
        Some(false) => "failed".to_string(),
        None => "skipped".to_string(),
    };
    let ok = result.accepted && result.reconstruction.unwrap_or(true);
    format!(
        "| {} | {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {} |",
        profile.name,
        result.relation.label(),
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v2::subset_log2_failure(profile, result.opened_cells),
        kb(commitment_size_bytes(&result.commitment)),
        kb(proof_bytes),
        kb(v2::transcript_size_bytes(&result.transcript)),
        result.timings.encode_commit.as_secs_f64(),
        result.timings.prover_preprocess.as_secs_f64(),
        result.timings.prove.as_secs_f64(),
        result.timings.opening_generation.as_secs_f64(),
        result.timings.verifier_rebuild.as_secs_f64(),
        result.timings.proof_verify.as_secs_f64(),
        result.timings.verify_openings.as_secs_f64(),
        reconstruction,
        metadata.map(|m| m.cycles).unwrap_or_default(),
        metadata.map(|m| m.n_poseidons).unwrap_or_default(),
        metadata.map(|m| m.n_extension_ops).unwrap_or_default(),
        if ok { "accepted" } else { "failed" },
    )
}

fn v2_ext_row(result: &v2_ext::ExtBenchmarkResult) -> String {
    let profile = result.profile;
    let proof_field_elements = result.proof.execution.proof.proof_size_fe();
    let proof_bytes = proof_field_elements * size_of::<u32>();
    let metadata = result.proof.execution.metadata.as_ref();
    let reconstruction = match result.reconstruction {
        Some(true) => format_duration(result.timings.reconstruct),
        Some(false) => "failed".to_string(),
        None => "pending".to_string(),
    };
    let ok = result.accepted && result.reconstruction.unwrap_or(true);
    format!(
        "| {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {} |",
        profile.name,
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v2_ext::subset_log2_failure_with_replacement(profile, result.opened_cells),
        kb(v2_ext::commitment_size_bytes(&result.commitment)),
        kb(proof_bytes),
        kb(v2_ext::transcript_size_bytes(&result.transcript)),
        result.timings.encode_commit.as_secs_f64(),
        result.timings.prover_preprocess.as_secs_f64(),
        result.timings.prove.as_secs_f64(),
        result.timings.opening_generation.as_secs_f64(),
        result.timings.verifier_rebuild.as_secs_f64(),
        result.timings.proof_verify.as_secs_f64(),
        result.timings.verify_openings.as_secs_f64(),
        reconstruction,
        metadata.map(|m| m.cycles).unwrap_or_default(),
        metadata.map(|m| m.n_poseidons).unwrap_or_default(),
        metadata.map(|m| m.n_extension_ops).unwrap_or_default(),
        if ok { "accepted" } else { "failed" },
    )
}

fn format_duration(value: Option<Duration>) -> String {
    value
        .map(|duration| format!("{:.3}s", duration.as_secs_f64()))
        .unwrap_or_else(|| "n/a".to_string())
}

fn kb(bytes: usize) -> String {
    format!("{:.2}", bytes as f64 / 1024.0)
}
