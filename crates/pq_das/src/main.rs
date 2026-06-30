use std::time::{Duration, Instant};

use backend::PrimeCharacteristicRing;
use clap::{Parser, ValueEnum};
use lean_vm::F;
use pq_das::{DIGEST_LEN, ParameterProfile, demo_data, v2_base, v2_ext, v3_base, v3_ext};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum VersionName {
    #[value(name = "v2_base", alias = "v2-base", alias = "v2")]
    V2Base,
    #[value(name = "v2_ext", alias = "v2-ext")]
    V2Ext,
    #[value(name = "v3_base", alias = "v3-base")]
    V3Base,
    #[value(name = "v3_ext", alias = "v3-ext")]
    V3Ext,
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

impl From<V2RelationName> for v2_base::Relation {
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
#[command(about = "Benchmark the parameterized PQ-DAS V2 LeanVM demos")]
struct Cli {
    #[arg(long, value_enum, default_value_t = VersionName::V2Base)]
    version: VersionName,

    #[arg(long, value_enum, default_value_t = ProfileName::Blob128K1)]
    profile: ProfileName,

    #[arg(
        long = "all-v2-base-benchmarks",
        alias = "all-v2-benchmarks",
        help = "Run blob-128k-1, blob-128k-14, and blob-128k-16 under V2-base"
    )]
    all_v2_base_benchmarks: bool,

    #[arg(
        long = "all-v2-ext-benchmarks",
        help = "Run blob-ext-1, blob-ext-14, and blob-ext-16 under V2-ext"
    )]
    all_v2_ext_benchmarks: bool,

    #[arg(
        long = "all-v3-base-benchmarks",
        help = "Run blob-128k-1, blob-128k-14, and blob-128k-16 under V3-base"
    )]
    all_v3_base_benchmarks: bool,

    #[arg(
        long = "all-v3-ext-benchmarks",
        help = "Run blob-ext-1, blob-ext-14, and blob-ext-16 under V3-ext"
    )]
    all_v3_ext_benchmarks: bool,

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
    /// Resolves a named or custom CLI selection into a validated V2-base profile.
    fn selected_base_profile(&self) -> Result<ParameterProfile, Box<dyn std::error::Error>> {
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
                return Err("extension profiles require --version v2_ext".into());
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
            _ => return Err("v2_ext requires --profile blob-ext-1, blob-ext-14, or blob-ext-16".into()),
        };
        profile.whir_log_inv_rate = self.whir_log_inv_rate;
        profile.validate()?;
        Ok(profile)
    }

    fn selected_v3_ext_profile(&self) -> Result<v3_ext::ExtProfile, Box<dyn std::error::Error>> {
        let mut profile = match self.profile {
            ProfileName::BlobExt1 => v3_ext::ExtProfile::BLOB_EXT_1,
            ProfileName::BlobExt14 => v3_ext::ExtProfile::BLOB_EXT_14,
            ProfileName::BlobExt16 => v3_ext::ExtProfile::BLOB_EXT_16,
            _ => return Err("v3_ext requires --profile blob-ext-1, blob-ext-14, or blob-ext-16".into()),
        };
        profile.whir_log_inv_rate = self.whir_log_inv_rate;
        profile.validate()?;
        Ok(profile)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::parallel::init();
    let cli = Cli::parse();
    if cli.all_v2_base_benchmarks {
        run_all_v2_base_benchmarks(cli.skip_reconstruction, cli.v2_relation.into())?;
        return Ok(());
    }
    if cli.all_v2_ext_benchmarks {
        run_all_v2_ext_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.all_v3_base_benchmarks {
        run_all_v3_base_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.all_v3_ext_benchmarks {
        run_all_v3_ext_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }

    match cli.version {
        VersionName::V2Base => run_v2_base_single(
            cli.selected_base_profile()?,
            cli.skip_reconstruction,
            cli.v2_relation.into(),
        )?,
        VersionName::V2Ext => run_v2_ext_single(cli.selected_ext_profile()?, cli.skip_reconstruction)?,
        VersionName::V3Base => run_v3_base_single(cli.selected_base_profile()?, cli.skip_reconstruction)?,
        VersionName::V3Ext => run_v3_ext_single(cli.selected_v3_ext_profile()?, cli.skip_reconstruction)?,
    }
    Ok(())
}

/// Runs one V2-base benchmark and prints the detailed report plus VM counters.
fn run_v2_base_single(
    profile: ParameterProfile,
    skip_reconstruction: bool,
    relation: v2_base::Relation,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v2_base_benchmark(profile, skip_reconstruction, relation)?;
    print_v2_base_report(&result);
    Ok(())
}

/// Runs and prints the requested V2-base benchmark table.
fn run_all_v2_base_benchmarks(
    skip_reconstruction: bool,
    relation: v2_base::Relation,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        ParameterProfile::BLOB_128K_1,
        ParameterProfile::BLOB_128K_14,
        ParameterProfile::BLOB_128K_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v2_base_benchmark(profile, skip_reconstruction, relation)?);
    }
    print_v2_base_table(&results);
    Ok(())
}

/// Runs one V2-ext benchmark and prints the detailed report plus VM counters.
fn run_v2_ext_single(profile: v2_ext::ExtProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v2_ext_benchmark(profile, skip_reconstruction)?;
    print_v2_ext_report(&result);
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

/// Runs one V3-base benchmark and prints the detailed report plus VM counters.
fn run_v3_base_single(profile: ParameterProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v3_base_benchmark(profile, skip_reconstruction)?;
    print_v3_base_report(&result);
    Ok(())
}

/// Runs and prints the V3-base benchmark table.
fn run_all_v3_base_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        ParameterProfile::BLOB_128K_1,
        ParameterProfile::BLOB_128K_14,
        ParameterProfile::BLOB_128K_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_base_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_base_table(&results);
    Ok(())
}

/// Runs one V3-ext benchmark and prints the detailed report plus VM counters.
fn run_v3_ext_single(profile: v3_ext::ExtProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v3_ext_benchmark(profile, skip_reconstruction)?;
    print_v3_ext_report(&result);
    Ok(())
}

/// Runs and prints the V3-ext benchmark table.
fn run_all_v3_ext_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_1,
        v3_ext::ExtProfile::BLOB_EXT_14,
        v3_ext::ExtProfile::BLOB_EXT_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

fn run_v2_base_benchmark(
    profile: ParameterProfile,
    skip_reconstruction: bool,
    relation: v2_base::Relation,
) -> Result<v2_base::BenchmarkResult, Box<dyn std::error::Error>> {
    let data = demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v2_base::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v2_base::prepare_statement_with_relation(commitment, relation)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v2_base::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v2_base::V2_OPENED_CELLS.min(profile.n_cells());
    let started = Instant::now();
    let indices = v2_base::sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v2_base::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v2_base::prepare_statement_with_relation(prepared.commitment.clone(), relation)?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v2_base::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v2_base::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = v2_base::sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v2_base::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v2_base::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };

    Ok(v2_base::BenchmarkResult {
        relation,
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v2_base::BenchmarkTimings {
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

fn run_v3_base_benchmark(
    profile: ParameterProfile,
    skip_reconstruction: bool,
) -> Result<v3_base::BenchmarkResult, Box<dyn std::error::Error>> {
    let data = demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v3_base::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v3_base::prepare_statement(commitment)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v3_base::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v3_base::V3_OPENED_CELLS.min(profile.n_cells());
    let started = Instant::now();
    let indices = v3_base::sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v3_base::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v3_base::prepare_statement(prepared.commitment.clone())?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v3_base::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v3_base::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = v3_base::sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v3_base::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v3_base::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };

    Ok(v3_base::BenchmarkResult {
        relation: v3_base::Relation::Full,
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v3_base::BenchmarkTimings {
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

fn run_v3_ext_benchmark(
    profile: v3_ext::ExtProfile,
    skip_reconstruction: bool,
) -> Result<v3_ext::ExtBenchmarkResult, Box<dyn std::error::Error>> {
    let data = v3_ext::demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v3_ext::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v3_ext::prepare_statement(commitment)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v3_ext::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v3_ext::opened_cells(profile).min(profile.n_cells());
    let started = Instant::now();
    let indices = v3_ext::sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v3_ext::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v3_ext::prepare_statement(prepared.commitment.clone())?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v3_ext::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v3_ext::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = v3_ext::sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v3_ext::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v3_ext::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };

    Ok(v3_ext::ExtBenchmarkResult {
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v3_ext::ExtBenchmarkTimings {
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

fn print_v2_base_report(result: &v2_base::BenchmarkResult) {
    println!("PQ-DAS V2-base LeanVM demo");
    println!("{}", v2_base_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v2_base_table(results: &[v2_base::BenchmarkResult]) {
    println!("PQ-DAS V2-base LeanVM benchmark table");
    println!(
        "| Profile | Relation | Bytecode instructions | Read-only elements | Opened cells | $\\log_2\\nu_{{\\mathrm{{wor}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Result |"
    );
    println!(
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v2_base_row(result));
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

fn print_v3_base_report(result: &v3_base::BenchmarkResult) {
    println!("PQ-DAS V3-base LeanVM demo");
    println!("{}", v3_base_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v3_base_table(results: &[v3_base::BenchmarkResult]) {
    println!("PQ-DAS V3-base LeanVM benchmark table");
    println!(
        "| Profile | Bytecode instructions | Read-only elements | Opened cells | $\\log_2\\nu_{{\\mathrm{{wor}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v3_base_row(result));
    }
}

fn print_v3_ext_report(result: &v3_ext::ExtBenchmarkResult) {
    println!("PQ-DAS V3-ext LeanVM demo");
    println!("{}", v3_ext_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v3_ext_table(results: &[v3_ext::ExtBenchmarkResult]) {
    println!("PQ-DAS V3-ext LeanVM benchmark table");
    println!(
        "| Profile | Bytecode instructions | Read-only elements | Opened cells | $\\log_2\\nu_{{\\mathrm{{rep}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v3_ext_row(result));
    }
}

fn v2_base_row(result: &v2_base::BenchmarkResult) -> String {
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
        v2_base::subset_log2_failure(profile, result.opened_cells),
        kb(v2_base::commitment_size_bytes(&result.commitment)),
        kb(proof_bytes),
        kb(v2_base::transcript_size_bytes(&result.transcript)),
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
        None => "skipped".to_string(),
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

fn v3_base_row(result: &v3_base::BenchmarkResult) -> String {
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
        "| {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {} |",
        profile.name,
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v3_base::subset_log2_failure(profile, result.opened_cells),
        kb(v3_base::commitment_size_bytes(&result.commitment)),
        kb(proof_bytes),
        kb(v3_base::transcript_size_bytes(&result.transcript)),
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

fn v3_ext_row(result: &v3_ext::ExtBenchmarkResult) -> String {
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
        "| {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {} |",
        profile.name,
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v3_ext::subset_log2_failure_with_replacement(profile, result.opened_cells),
        kb(v3_ext::commitment_size_bytes(&result.commitment)),
        kb(proof_bytes),
        kb(v3_ext::transcript_size_bytes(&result.transcript)),
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
