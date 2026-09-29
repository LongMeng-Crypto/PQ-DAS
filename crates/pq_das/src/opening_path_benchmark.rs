//! Supplementary host measurements; this module never executes a LeanVM proof.

use std::{
    fs::{self, OpenOptions},
    hint::black_box,
    io::{self, BufWriter, Write},
    path::PathBuf,
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use backend::{PrimeCharacteristicRing, PrimeField32, poseidon16_compress_pair};
use clap::Args;
use lean_vm::F;
use serde::Serialize;

use crate::{
    DIGEST_LEN,
    hashing::{Digest, merkle_layers},
};

const COLUMN_COUNTS: [usize; 5] = [256, 512, 1024, 2048, 4096];
const DIGEST_BYTES: usize = DIGEST_LEN * size_of::<u32>();
const UPLOAD_BYTES_PER_SECOND: u64 = 50_000_000 / 8;
const WARMUP_REPETITIONS: usize = 3;

#[derive(Debug, Args)]
pub struct Options {
    /// Measure all-column path assembly only, covering both supplementary documents.
    #[arg(long)]
    pub all_opening_paths_only: bool,

    /// Repetitions of the lightweight path assembly, without encoding or proving.
    #[arg(
        long,
        default_value_t = 100,
        value_parser = clap::value_parser!(u32).range(1..=10000),
        requires = "all_opening_paths_only"
    )]
    pub path_benchmark_repetitions: u32,

    /// Save raw nanosecond timings and exact byte counts to a new JSON file.
    #[arg(long, requires = "all_opening_paths_only")]
    pub path_benchmark_output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum PathFormat {
    ColumnRoot,
    FinalRoot,
}

impl PathFormat {
    fn name(self) -> &'static str {
        match self {
            Self::ColumnRoot => "column-root",
            Self::FinalRoot => "final-root",
        }
    }

    fn final_sibling(self, row_root: &Digest) -> Option<&Digest> {
        match self {
            Self::ColumnRoot => None,
            Self::FinalRoot => Some(row_root),
        }
    }
}

#[derive(Debug, Serialize)]
struct Measurement {
    format: PathFormat,
    columns: usize,
    digests_per_path: usize,
    sibling_bytes: usize,
    index_bytes: usize,
    all_path_bytes: usize,
    elapsed_ns: Vec<u64>,
    mean_seconds: f64,
    median_seconds: f64,
    min_seconds: f64,
    max_seconds: f64,
    upload_seconds: f64,
    verified: bool,
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    unix_timestamp_seconds: u64,
    git_revision: Option<String>,
    os: &'static str,
    architecture: &'static str,
    cpu_model: Option<String>,
    rayon_num_threads_env: Option<String>,
    assembly_threads: usize,
    warmup_repetitions: usize,
    repetitions: u32,
    upload_bytes_per_second: u64,
    methodology: &'static str,
    wire_format: &'static str,
    measurements: Vec<Measurement>,
}

fn write_digest(bytes: &mut Vec<u8>, digest: &Digest) {
    for element in digest {
        bytes.extend_from_slice(&element.as_canonical_u32().to_le_bytes());
    }
}

/// Each packet is [u32 column index, bottom-up outer siblings, optional row root].
/// The profile fixes its length; cells and public commitment bytes are separate.
fn assemble_all_paths(layers: &[Vec<Digest>], row_root: Option<&Digest>) -> Vec<u8> {
    let columns = layers[0].len();
    let depth = layers.len() - 1;
    let packet_bytes = size_of::<u32>() + (depth + usize::from(row_root.is_some())) * DIGEST_BYTES;
    let mut bytes = Vec::with_capacity(columns * packet_bytes);
    for index in 0..columns {
        bytes.extend_from_slice(&u32::try_from(index).unwrap().to_le_bytes());
        let mut node = index;
        for layer in layers.iter().take(depth) {
            write_digest(&mut bytes, &layer[node ^ 1]);
            node /= 2;
        }
        if let Some(row_root) = row_root {
            write_digest(&mut bytes, row_root);
        }
    }
    bytes
}

fn read_digest(bytes: &[u8]) -> Option<Digest> {
    if bytes.len() != DIGEST_BYTES {
        return None;
    }
    let mut digest = [F::ZERO; DIGEST_LEN];
    for (element, bytes) in digest.iter_mut().zip(bytes.chunks_exact(size_of::<u32>())) {
        let value = u32::from_le_bytes(bytes.try_into().ok()?);
        if value >= F::ORDER_U32 {
            return None;
        }
        *element = F::from_u32(value);
    }
    Some(digest)
}

fn verify_all_paths(bytes: &[u8], leaves: &[Digest], expected_root: Digest, final_root_format: bool) -> bool {
    if !leaves.len().is_power_of_two() {
        return false;
    }
    let depth = leaves.len().ilog2() as usize;
    let packet_bytes = size_of::<u32>() + (depth + usize::from(final_root_format)) * DIGEST_BYTES;
    if bytes.len() != leaves.len() * packet_bytes {
        return false;
    }
    for (index, packet) in bytes.chunks_exact(packet_bytes).enumerate() {
        if u32::from_le_bytes(packet[..4].try_into().unwrap()) as usize != index {
            return false;
        }
        let mut node = index;
        let mut root = leaves[index];
        let mut siblings = packet[4..].chunks_exact(DIGEST_BYTES);
        for bytes in siblings.by_ref().take(depth) {
            let Some(sibling) = read_digest(bytes) else {
                return false;
            };
            root = if node % 2 == 0 {
                poseidon16_compress_pair(&root, &sibling)
            } else {
                poseidon16_compress_pair(&sibling, &root)
            };
            node /= 2;
        }
        if final_root_format {
            let Some(row_root) = siblings.next().and_then(read_digest) else {
                return false;
            };
            root = poseidon16_compress_pair(&row_root, &root);
        }
        if root != expected_root {
            return false;
        }
    }
    true
}

fn fixture(columns: usize) -> (Vec<Vec<Digest>>, Digest) {
    // Copying paths depends on tree dimensions, not the encoded payload values.
    let leaves: Vec<Digest> = (0..columns)
        .map(|column| std::array::from_fn(|coord| F::from_usize(1 + column * DIGEST_LEN + coord)))
        .collect();
    let row_root = std::array::from_fn(|coord| F::from_usize(1 + columns * DIGEST_LEN + coord));
    (merkle_layers(&leaves), row_root)
}

fn measure(layers: &[Vec<Digest>], row_root: &Digest, format: PathFormat, repetitions: u32) -> Measurement {
    let final_sibling = format.final_sibling(row_root);
    for _ in 0..WARMUP_REPETITIONS {
        black_box(assemble_all_paths(black_box(layers), black_box(final_sibling)));
    }
    let mut elapsed_ns = Vec::with_capacity(repetitions as usize);
    let mut last_bytes = Vec::new();
    for _ in 0..repetitions {
        let started = Instant::now();
        let bytes = assemble_all_paths(black_box(layers), black_box(final_sibling));
        black_box(&bytes);
        let elapsed = started.elapsed();
        elapsed_ns.push(u64::try_from(elapsed.as_nanos()).unwrap());
        // Release the previous buffer outside the measured allocation/assembly/serialization.
        last_bytes = bytes;
    }
    let columns = layers[0].len();
    let digests_per_path = layers.len() - 1 + usize::from(final_sibling.is_some());
    let root_col = layers.last().unwrap()[0];
    let expected_root = final_sibling.map_or(root_col, |root_row| poseidon16_compress_pair(root_row, &root_col));
    let verified = verify_all_paths(&last_bytes, &layers[0], expected_root, final_sibling.is_some());
    let mut sorted = elapsed_ns.clone();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    let median_ns = (sorted[(sorted.len() - 1) / 2] as f64 + sorted[mid] as f64) / 2.0;
    let mean_seconds = elapsed_ns.iter().map(|&ns| ns as f64).sum::<f64>() / repetitions as f64 / 1e9;
    Measurement {
        format,
        columns,
        digests_per_path,
        sibling_bytes: columns * digests_per_path * DIGEST_BYTES,
        index_bytes: columns * size_of::<u32>(),
        all_path_bytes: last_bytes.len(),
        elapsed_ns,
        mean_seconds,
        median_seconds: median_ns / 1e9,
        min_seconds: sorted[0] as f64 / 1e9,
        max_seconds: sorted[sorted.len() - 1] as f64 / 1e9,
        upload_seconds: last_bytes.len() as f64 / UPLOAD_BYTES_PER_SECOND as f64,
        verified,
    }
}

pub fn run(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    if options.path_benchmark_repetitions == 0 {
        return Err("path benchmark repetitions must be positive".into());
    }
    let output = options
        .path_benchmark_output
        .as_ref()
        .map(|path| OpenOptions::new().write(true).create_new(true).open(path))
        .transpose()?;
    let git_revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().to_owned());
    let cpu_model = fs::read_to_string("/proc/cpuinfo").ok().and_then(|text| {
        text.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            (key.trim() == "model name").then(|| value.trim().to_owned())
        })
    });
    let mut report = Report {
        schema_version: 1,
        unix_timestamp_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        git_revision,
        os: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        cpu_model,
        rayon_num_threads_env: std::env::var("RAYON_NUM_THREADS").ok(),
        assembly_threads: 1,
        warmup_repetitions: WARMUP_REPETITIONS,
        repetitions: options.path_benchmark_repetitions,
        upload_bytes_per_second: UPLOAD_BYTES_PER_SECOND,
        methodology: "Synthetic column digests; existing Poseidon tree construction before timing. Time allocation, extraction of every independent path, and canonical serialization. Verification and buffer destruction are outside timing. No encoding, cell copying, LeanVM proof, or reconstruction.",
        wire_format: "For every column in increasing order: u32 little-endian index, bottom-up siblings (8 canonical u32 little-endian coordinates per digest), and a row-root sibling only for final-root format. Profile fixes packet lengths; codeword cells and public commitments are excluded.",
        measurements: Vec::new(),
    };
    println!("PQ-DAS all-column opening paths supplement");
    println!(
        "{} assembly repetitions per case; 3 warmups; one assembly thread; tree construction and verification excluded.",
        options.path_benchmark_repetitions
    );
    println!(
        "Same (ell, format) measurement applies across row counts, blob sizes, WHIR rates, and precompile variants."
    );
    println!("All path size includes one u32 column index per packet. No existing throughput is recalculated.");
    println!(
        "| Path format | Columns ell | Digests/path | All path size (KiB) | Open all paths mean (ms) | Path upload at 50 Mbps (s) | Result |"
    );
    println!("| --- | ---: | ---: | ---: | ---: | ---: | --- |");
    for columns in COLUMN_COUNTS {
        let (layers, row_root) = fixture(columns);
        for format in [PathFormat::ColumnRoot, PathFormat::FinalRoot] {
            let result = measure(&layers, &row_root, format, options.path_benchmark_repetitions);
            if !result.verified {
                return Err(io::Error::other("all-column path verification failed").into());
            }
            println!(
                "| {} | {} | {} | {:.2} | {:.6} | {:.9} | verified |",
                format.name(),
                columns,
                result.digests_per_path,
                result.all_path_bytes as f64 / 1024.0,
                result.mean_seconds * 1000.0,
                result.upload_seconds,
            );
            report.measurements.push(result);
        }
    }
    if let Some(output) = output {
        let mut writer = BufWriter::new(output);
        serde_json::to_writer_pretty(&mut writer, &report)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        println!(
            "Raw measurements saved to {}",
            options.path_benchmark_output.as_ref().unwrap().display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_paths_match_existing_column_openings() {
        let profile = crate::v3_ext::ExtProfile {
            name: "path-test",
            n: 3,
            m: 64,
            k: 32,
            c: 8,
            whir_log_inv_rate: 1,
        };
        let data = crate::v3_ext::demo_data(profile);
        let (commitment, aux) = crate::v3_ext::encode_and_commit(profile, &data).unwrap();
        let bytes = assemble_all_paths(&aux.outer_merkle_layers, None);
        let packet_bytes = 4 + profile.n_cells().ilog2() as usize * DIGEST_BYTES;
        for (column, packet) in bytes.chunks_exact(packet_bytes).enumerate() {
            let transcript = crate::v3_ext::query(&aux, &[column]).unwrap();
            let siblings: Vec<_> = packet[4..]
                .chunks_exact(DIGEST_BYTES)
                .map(|b| read_digest(b).unwrap())
                .collect();
            assert_eq!(siblings, transcript.outer_multiproof);
        }
        assert!(verify_all_paths(&bytes, &aux.column_roots, commitment.root_col, false));
    }

    #[test]
    fn serialized_paths_authenticate_every_column_in_both_formats() {
        for columns in [1, 2, 16, 256] {
            let (layers, row_root) = fixture(columns);
            for format in [PathFormat::ColumnRoot, PathFormat::FinalRoot] {
                let sibling = format.final_sibling(&row_root);
                let bytes = assemble_all_paths(&layers, sibling);
                let root_col = layers.last().unwrap()[0];
                let expected_root = sibling.map_or(root_col, |row| poseidon16_compress_pair(row, &root_col));
                assert_eq!(
                    bytes.len(),
                    columns * (4 + (columns.ilog2() as usize + usize::from(sibling.is_some())) * 32)
                );
                assert!(verify_all_paths(&bytes, &layers[0], expected_root, sibling.is_some()));
            }
        }
    }

    #[test]
    fn rejects_corrupt_indices_siblings_truncated_and_extra_packets() {
        let (layers, row_root) = fixture(16);
        let expected_root = poseidon16_compress_pair(&row_root, &layers.last().unwrap()[0]);
        let bytes = assemble_all_paths(&layers, Some(&row_root));
        for offset in [0, 4, 4 + 4 * DIGEST_BYTES] {
            let mut corrupt = bytes.clone();
            corrupt[offset] ^= 1;
            assert!(!verify_all_paths(&corrupt, &layers[0], expected_root, true));
        }
        assert!(!verify_all_paths(
            &bytes[..bytes.len() - 1],
            &layers[0],
            expected_root,
            true
        ));
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(!verify_all_paths(&extra, &layers[0], expected_root, true));
        let mut noncanonical = bytes;
        noncanonical[4..8].copy_from_slice(&F::ORDER_U32.to_le_bytes());
        assert!(!verify_all_paths(&noncanonical, &layers[0], expected_root, true));
    }

    #[test]
    fn measured_sizes_include_independent_paths_and_indices() {
        let (layers, row_root) = fixture(1024);
        for (format, expected_bytes) in [(PathFormat::ColumnRoot, 331_776), (PathFormat::FinalRoot, 364_544)] {
            let measurement = measure(&layers, &row_root, format, 1);
            assert!(measurement.verified);
            assert_eq!(measurement.all_path_bytes, expected_bytes);
            assert_eq!(
                measurement.all_path_bytes,
                measurement.sibling_bytes + measurement.index_bytes
            );
            assert_eq!(measurement.elapsed_ns.len(), 1);
            assert_eq!(measurement.upload_seconds, expected_bytes as f64 / 6_250_000.0);
        }
    }
}
