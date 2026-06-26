use std::time::{Duration, Instant};

use backend::{PrimeCharacteristicRing, PrimeField32};
use clap::{Parser, ValueEnum};
use lean_vm::F;
use pq_das::{
    DIGEST_LEN, ParameterProfile, SAMPLING_SOUNDNESS_BITS, SAMPLING_TRANSCRIPTS, commitment_size_bytes, demo_data,
    encode_and_commit, prepare_statement, prove_codewords, query, reconstruct, sample_query_indices,
    transcript_size_bytes, v2, verify_execution_proof, verify_openings,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum VersionName {
    V1,
    V2,
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
    Custom,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::parallel::init();
    let cli = Cli::parse();
    if cli.all_v2_benchmarks {
        run_all_v2_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }

    let profile = cli.selected_profile()?;
    match cli.version {
        VersionName::V1 => run_v1_single(profile, cli.skip_reconstruction)?,
        VersionName::V2 => run_v2_single(profile, cli.skip_reconstruction)?,
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
fn run_v2_single(profile: ParameterProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v2_benchmark(profile, skip_reconstruction)?;
    print_v2_report(&result);
    Ok(())
}

/// Runs and prints the requested V2 benchmark table.
fn run_all_v2_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        ParameterProfile::BLOB_128K_1,
        ParameterProfile::BLOB_128K_14,
        ParameterProfile::BLOB_128K_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v2_benchmark(profile, skip_reconstruction)?);
    }
    print_v2_table(&results);
    Ok(())
}

fn run_v2_benchmark(
    profile: ParameterProfile,
    skip_reconstruction: bool,
) -> Result<v2::BenchmarkResult, Box<dyn std::error::Error>> {
    let data = demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v2::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v2::prepare_statement(commitment)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v2::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v2::V2_OPENED_CELLS.min(profile.n_cells());
    let indices = sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v2::query(&aux, &indices)?;

    let started = Instant::now();
    v2::verify_execution_proof(&prepared.commitment, &proof)?;
    let verifier_rebuild_verify = started.elapsed();

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
            verifier_rebuild_verify,
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
        "| Profile | Bytecode instructions | Read-only elements | Opened cells | $\\log_2\\nu_{{\\mathrm{{wor}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Verifier rebuild + LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v2_row(result));
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
        "| {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {} |",
        profile.name,
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
        result.timings.verifier_rebuild_verify.as_secs_f64(),
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
