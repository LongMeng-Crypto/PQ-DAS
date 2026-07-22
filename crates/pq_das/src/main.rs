use std::time::{Duration, Instant};

use backend::PrimeCharacteristicRing;
use clap::{Parser, ValueEnum};
use lean_vm::F;
use pq_das::{DIGEST_LEN, ParameterProfile, demo_data, v2_base, v2_ext, v3_base, v3_ext, v4_ext, v4_precompile};

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
    #[value(name = "v4_ext", alias = "v4-ext")]
    V4Ext,
    #[value(name = "v4_precompile", alias = "v4-precompile")]
    V4Precompile,
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
    #[value(name = "blob-256k-1")]
    Blob256K1,
    #[value(name = "blob-256k-14")]
    Blob256K14,
    #[value(name = "blob-256k-16")]
    Blob256K16,
    #[value(name = "blob-ext-1")]
    BlobExt1,
    #[value(name = "blob-ext-14")]
    BlobExt14,
    #[value(name = "blob-ext-16")]
    BlobExt16,
    #[value(name = "blob-ext-2x-1")]
    BlobExt2x1,
    #[value(name = "blob-ext-2x-2")]
    BlobExt2x2,
    #[value(name = "blob-ext-2x-4")]
    BlobExt2x4,
    #[value(name = "blob-ext-2x-6")]
    BlobExt2x6,
    #[value(name = "blob-ext-2x-8")]
    BlobExt2x8,
    #[value(name = "blob-ext-2x-10")]
    BlobExt2x10,
    #[value(name = "blob-ext-2x-12")]
    BlobExt2x12,
    #[value(name = "blob-ext-2x-14")]
    BlobExt2x14,
    #[value(name = "blob-ext-2x-15")]
    BlobExt2x15,
    #[value(name = "blob-ext-2x-16")]
    BlobExt2x16,
    #[value(name = "blob-ext-2x-18")]
    BlobExt2x18,
    #[value(name = "blob-ext-2x-20")]
    BlobExt2x20,
    #[value(name = "blob-ext-2x-22")]
    BlobExt2x22,
    #[value(name = "blob-ext-2x-24")]
    BlobExt2x24,
    #[value(name = "blob-ext-2x-26")]
    BlobExt2x26,
    #[value(name = "blob-ext-2x-28")]
    BlobExt2x28,
    #[value(name = "blob-ext-2x-30")]
    BlobExt2x30,
    #[value(name = "blob-ext-2x-32")]
    BlobExt2x32,
    #[value(name = "blob-ext-2x-c16-14")]
    BlobExt2xC16_14,
    #[value(name = "blob-ext-2x-c64-1")]
    BlobExt2xC64_1,
    #[value(name = "blob-ext-2x-c64-14")]
    BlobExt2xC64_14,
    #[value(name = "blob-ext-2x-c64-16")]
    BlobExt2xC64_16,
    #[value(name = "blob-ext-2x-c128-1")]
    BlobExt2xC128_1,
    #[value(name = "blob-ext-2x-c128-14")]
    BlobExt2xC128_14,
    #[value(name = "blob-ext-2x-c128-16")]
    BlobExt2xC128_16,
    #[value(name = "blob-ext-2x-c128-28")]
    BlobExt2xC128_28,
    #[value(name = "blob-ext-2x-c128-30")]
    BlobExt2xC128_30,
    #[value(name = "blob-ext-4x-1")]
    BlobExt4x1,
    #[value(name = "blob-ext-4x-2")]
    BlobExt4x2,
    #[value(name = "blob-ext-4x-4")]
    BlobExt4x4,
    #[value(name = "blob-ext-4x-6")]
    BlobExt4x6,
    #[value(name = "blob-ext-4x-8")]
    BlobExt4x8,
    #[value(name = "blob-ext-4x-10")]
    BlobExt4x10,
    #[value(name = "blob-ext-4x-12")]
    BlobExt4x12,
    #[value(name = "blob-ext-4x-14")]
    BlobExt4x14,
    #[value(name = "blob-ext-4x-16")]
    BlobExt4x16,
    #[value(name = "blob-ext-4x-c16-14")]
    BlobExt4xC16_14,
    #[value(name = "blob-ext-4x-c32-1")]
    BlobExt4xC32_1,
    #[value(name = "blob-ext-4x-c32-2")]
    BlobExt4xC32_2,
    #[value(name = "blob-ext-4x-c32-4")]
    BlobExt4xC32_4,
    #[value(name = "blob-ext-4x-c32-6")]
    BlobExt4xC32_6,
    #[value(name = "blob-ext-4x-c32-8")]
    BlobExt4xC32_8,
    #[value(name = "blob-ext-4x-c32-10")]
    BlobExt4xC32_10,
    #[value(name = "blob-ext-4x-c32-12")]
    BlobExt4xC32_12,
    #[value(name = "blob-ext-4x-c32-14")]
    BlobExt4xC32_14,
    #[value(name = "blob-ext-4x-c32-16")]
    BlobExt4xC32_16,
    #[value(name = "blob-ext-4x-c32-18")]
    BlobExt4xC32_18,
    #[value(name = "blob-ext-4x-c32-20")]
    BlobExt4xC32_20,
    #[value(name = "blob-ext-4x-c32-22")]
    BlobExt4xC32_22,
    #[value(name = "blob-ext-4x-c32-24")]
    BlobExt4xC32_24,
    #[value(name = "blob-ext-4x-c32-26")]
    BlobExt4xC32_26,
    #[value(name = "blob-ext-4x-c32-28")]
    BlobExt4xC32_28,
    #[value(name = "blob-ext-4x-c32-30")]
    BlobExt4xC32_30,
    #[value(name = "blob-ext-4x-c32-32")]
    BlobExt4xC32_32,
    #[value(name = "blob-ext-4x-c128-14")]
    BlobExt4xC128_14,
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
        help = "Run the 1x and 2x blob profiles under V3-base"
    )]
    all_v3_base_benchmarks: bool,

    #[arg(
        long = "all-v3-ext-benchmarks",
        help = "Run the 1x, 2x, and 4x extension-blob profiles under V3-ext"
    )]
    all_v3_ext_benchmarks: bool,

    #[arg(
        long = "all-v4-ext-benchmarks",
        help = "Run the V4-ext column-root-only benchmark profile"
    )]
    all_v4_ext_benchmarks: bool,

    #[arg(
        long = "all-v4-precompile-benchmarks",
        help = "Run the V4-precompile macro-expanded V3-ext benchmark profile"
    )]
    all_v4_precompile_benchmarks: bool,

    #[arg(
        long = "v4-ext-row-count-sweep",
        help = "Run V4-ext 2x, c=32 profiles for n=1,2,4,6,...,32"
    )]
    v4_ext_row_count_sweep: bool,

    #[arg(
        long = "v3-ext-blob-size-sweep",
        help = "Run V3-ext n=14, ell=1024 profiles for blob sizes 1x, 2x, and 4x"
    )]
    v3_ext_blob_size_sweep: bool,

    #[arg(
        long = "v3-ext-cell-size-sweep",
        help = "Run V3-ext 2x, n=14 profiles for cell sizes c=16, c=32, c=64, and c=128"
    )]
    v3_ext_cell_size_sweep: bool,

    #[arg(
        long = "v3-ext-row-count-sweep",
        help = "Run V3-ext 2x, c=32 profiles for n=1,2,4,6,...,14,15,16,...,32"
    )]
    v3_ext_row_count_sweep: bool,

    #[arg(
        long = "v3-ext-whir-rate-sweep",
        help = "Run V3-ext blob-ext-2x-14 and blob-ext-4x-14 with supported WHIR log inverse rates 1 and 2"
    )]
    v3_ext_whir_rate_sweep: bool,

    #[arg(
        long = "all-v3-ext-cellsize-benchmarks",
        help = "Run V3-ext 2x profiles with cell sizes c=32, c=64, and c=128"
    )]
    all_v3_ext_cellsize_benchmarks: bool,

    #[arg(
        long = "v3-ext-4x-cell-size-sweep",
        help = "Run V3-ext 4x, n=14 profiles for cell sizes c=16, c=32, c=64, and c=128"
    )]
    v3_ext_4x_cell_size_sweep: bool,

    #[arg(
        long = "all-v3-ext-4x-cellsize-benchmarks",
        help = "Run V3-ext 4x profiles with cell sizes c=16, c=32, c=64, and c=128"
    )]
    all_v3_ext_4x_cellsize_benchmarks: bool,

    #[arg(
        long = "all-v3-ext-2x-row-benchmarks",
        help = "Run V3-ext 2x profiles for n=1,2,4,6,...,14,15,16,...,32"
    )]
    all_v3_ext_2x_row_benchmarks: bool,

    #[arg(
        long = "v3-ext-4x-row-count-sweep",
        help = "Run V3-ext 4x, c=32 profiles for n=1,2,4,6,...,16"
    )]
    v3_ext_4x_row_count_sweep: bool,

    #[arg(
        long = "all-v3-ext-4x-row-benchmarks",
        help = "Run V3-ext 4x-c32 profiles for n=1,2,4,6,...,16"
    )]
    all_v3_ext_4x_row_benchmarks: bool,

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
            ProfileName::Blob256K1 => ParameterProfile::BLOB_256K_1,
            ProfileName::Blob256K14 => ParameterProfile::BLOB_256K_14,
            ProfileName::Blob256K16 => ParameterProfile::BLOB_256K_16,
            ProfileName::BlobExt1
            | ProfileName::BlobExt14
            | ProfileName::BlobExt16
            | ProfileName::BlobExt2x1
            | ProfileName::BlobExt2x2
            | ProfileName::BlobExt2x4
            | ProfileName::BlobExt2x6
            | ProfileName::BlobExt2x8
            | ProfileName::BlobExt2x10
            | ProfileName::BlobExt2x12
            | ProfileName::BlobExt2x14
            | ProfileName::BlobExt2x15
            | ProfileName::BlobExt2x16
            | ProfileName::BlobExt2x18
            | ProfileName::BlobExt2x20
            | ProfileName::BlobExt2x22
            | ProfileName::BlobExt2x24
            | ProfileName::BlobExt2x26
            | ProfileName::BlobExt2x28
            | ProfileName::BlobExt2x30
            | ProfileName::BlobExt2x32
            | ProfileName::BlobExt2xC16_14
            | ProfileName::BlobExt2xC64_1
            | ProfileName::BlobExt2xC64_14
            | ProfileName::BlobExt2xC64_16
            | ProfileName::BlobExt2xC128_1
            | ProfileName::BlobExt2xC128_14
            | ProfileName::BlobExt2xC128_16
            | ProfileName::BlobExt2xC128_28
            | ProfileName::BlobExt2xC128_30
            | ProfileName::BlobExt4x1
            | ProfileName::BlobExt4x2
            | ProfileName::BlobExt4x4
            | ProfileName::BlobExt4x6
            | ProfileName::BlobExt4x8
            | ProfileName::BlobExt4x10
            | ProfileName::BlobExt4x12
            | ProfileName::BlobExt4x14
            | ProfileName::BlobExt4x16
            | ProfileName::BlobExt4xC16_14
            | ProfileName::BlobExt4xC32_1
            | ProfileName::BlobExt4xC32_2
            | ProfileName::BlobExt4xC32_4
            | ProfileName::BlobExt4xC32_6
            | ProfileName::BlobExt4xC32_8
            | ProfileName::BlobExt4xC32_10
            | ProfileName::BlobExt4xC32_12
            | ProfileName::BlobExt4xC32_14
            | ProfileName::BlobExt4xC32_16
            | ProfileName::BlobExt4xC32_18
            | ProfileName::BlobExt4xC32_20
            | ProfileName::BlobExt4xC32_22
            | ProfileName::BlobExt4xC32_24
            | ProfileName::BlobExt4xC32_26
            | ProfileName::BlobExt4xC32_28
            | ProfileName::BlobExt4xC32_30
            | ProfileName::BlobExt4xC32_32
            | ProfileName::BlobExt4xC128_14 => {
                return Err(
                    "extension profiles require --version v2_ext, --version v3_ext, or --version v4_ext".into(),
                );
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
            ProfileName::BlobExt2x1
            | ProfileName::BlobExt2x2
            | ProfileName::BlobExt2x4
            | ProfileName::BlobExt2x6
            | ProfileName::BlobExt2x8
            | ProfileName::BlobExt2x10
            | ProfileName::BlobExt2x12
            | ProfileName::BlobExt2x14
            | ProfileName::BlobExt2x15
            | ProfileName::BlobExt2x16
            | ProfileName::BlobExt2x18
            | ProfileName::BlobExt2x20
            | ProfileName::BlobExt2x22
            | ProfileName::BlobExt2x24
            | ProfileName::BlobExt2x26
            | ProfileName::BlobExt2x28
            | ProfileName::BlobExt2x30
            | ProfileName::BlobExt2x32
            | ProfileName::BlobExt2xC16_14
            | ProfileName::BlobExt2xC64_1
            | ProfileName::BlobExt2xC64_14
            | ProfileName::BlobExt2xC64_16
            | ProfileName::BlobExt2xC128_1
            | ProfileName::BlobExt2xC128_14
            | ProfileName::BlobExt2xC128_16
            | ProfileName::BlobExt2xC128_28
            | ProfileName::BlobExt2xC128_30
            | ProfileName::BlobExt4x1
            | ProfileName::BlobExt4x14
            | ProfileName::BlobExt4x16
            | ProfileName::BlobExt4xC32_1
            | ProfileName::BlobExt4xC32_2
            | ProfileName::BlobExt4xC32_4
            | ProfileName::BlobExt4xC32_6
            | ProfileName::BlobExt4xC32_8
            | ProfileName::BlobExt4xC32_10
            | ProfileName::BlobExt4xC32_12
            | ProfileName::BlobExt4xC32_14
            | ProfileName::BlobExt4xC32_16 => {
                return Err("larger extension profiles require --version v3_ext".into());
            }
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
            ProfileName::BlobExt2x1 => v3_ext::ExtProfile::BLOB_EXT_2X_1,
            ProfileName::BlobExt2x2 => v3_ext::ExtProfile::BLOB_EXT_2X_2,
            ProfileName::BlobExt2x4 => v3_ext::ExtProfile::BLOB_EXT_2X_4,
            ProfileName::BlobExt2x6 => v3_ext::ExtProfile::BLOB_EXT_2X_6,
            ProfileName::BlobExt2x8 => v3_ext::ExtProfile::BLOB_EXT_2X_8,
            ProfileName::BlobExt2x10 => v3_ext::ExtProfile::BLOB_EXT_2X_10,
            ProfileName::BlobExt2x12 => v3_ext::ExtProfile::BLOB_EXT_2X_12,
            ProfileName::BlobExt2x14 => v3_ext::ExtProfile::BLOB_EXT_2X_14,
            ProfileName::BlobExt2x15 => v3_ext::ExtProfile::BLOB_EXT_2X_15,
            ProfileName::BlobExt2x16 => v3_ext::ExtProfile::BLOB_EXT_2X_16,
            ProfileName::BlobExt2x18 => v3_ext::ExtProfile::BLOB_EXT_2X_18,
            ProfileName::BlobExt2x20 => v3_ext::ExtProfile::BLOB_EXT_2X_20,
            ProfileName::BlobExt2x22 => v3_ext::ExtProfile::BLOB_EXT_2X_22,
            ProfileName::BlobExt2x24 => v3_ext::ExtProfile::BLOB_EXT_2X_24,
            ProfileName::BlobExt2x26 => v3_ext::ExtProfile::BLOB_EXT_2X_26,
            ProfileName::BlobExt2x28 => v3_ext::ExtProfile::BLOB_EXT_2X_28,
            ProfileName::BlobExt2x30 => v3_ext::ExtProfile::BLOB_EXT_2X_30,
            ProfileName::BlobExt2x32 => v3_ext::ExtProfile::BLOB_EXT_2X_32,
            ProfileName::BlobExt2xC16_14 => v3_ext::ExtProfile::BLOB_EXT_2X_C16_14,
            ProfileName::BlobExt2xC64_1 => v3_ext::ExtProfile::BLOB_EXT_2X_C64_1,
            ProfileName::BlobExt2xC64_14 => v3_ext::ExtProfile::BLOB_EXT_2X_C64_14,
            ProfileName::BlobExt2xC64_16 => v3_ext::ExtProfile::BLOB_EXT_2X_C64_16,
            ProfileName::BlobExt2xC128_1 => v3_ext::ExtProfile::BLOB_EXT_2X_C128_1,
            ProfileName::BlobExt2xC128_14 => v3_ext::ExtProfile::BLOB_EXT_2X_C128_14,
            ProfileName::BlobExt2xC128_16 => v3_ext::ExtProfile::BLOB_EXT_2X_C128_16,
            ProfileName::BlobExt2xC128_28 => v3_ext::ExtProfile::BLOB_EXT_2X_C128_28,
            ProfileName::BlobExt2xC128_30 => v3_ext::ExtProfile::BLOB_EXT_2X_C128_30,
            ProfileName::BlobExt4x1 => v3_ext::ExtProfile::BLOB_EXT_4X_1,
            ProfileName::BlobExt4x2 => v3_ext::ExtProfile::BLOB_EXT_4X_2,
            ProfileName::BlobExt4x4 => v3_ext::ExtProfile::BLOB_EXT_4X_4,
            ProfileName::BlobExt4x6 => v3_ext::ExtProfile::BLOB_EXT_4X_6,
            ProfileName::BlobExt4x8 => v3_ext::ExtProfile::BLOB_EXT_4X_8,
            ProfileName::BlobExt4x10 => v3_ext::ExtProfile::BLOB_EXT_4X_10,
            ProfileName::BlobExt4x12 => v3_ext::ExtProfile::BLOB_EXT_4X_12,
            ProfileName::BlobExt4x14 => v3_ext::ExtProfile::BLOB_EXT_4X_14,
            ProfileName::BlobExt4x16 => v3_ext::ExtProfile::BLOB_EXT_4X_16,
            ProfileName::BlobExt4xC16_14 => v3_ext::ExtProfile::BLOB_EXT_4X_C16_14,
            ProfileName::BlobExt4xC32_1 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_1,
            ProfileName::BlobExt4xC32_2 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_2,
            ProfileName::BlobExt4xC32_4 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_4,
            ProfileName::BlobExt4xC32_6 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_6,
            ProfileName::BlobExt4xC32_8 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_8,
            ProfileName::BlobExt4xC32_10 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_10,
            ProfileName::BlobExt4xC32_12 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_12,
            ProfileName::BlobExt4xC32_14 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_14,
            ProfileName::BlobExt4xC32_16 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_16,
            ProfileName::BlobExt4xC32_18 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_18,
            ProfileName::BlobExt4xC32_20 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_20,
            ProfileName::BlobExt4xC32_22 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_22,
            ProfileName::BlobExt4xC32_24 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_24,
            ProfileName::BlobExt4xC32_26 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_26,
            ProfileName::BlobExt4xC32_28 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_28,
            ProfileName::BlobExt4xC32_30 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_30,
            ProfileName::BlobExt4xC32_32 => v3_ext::ExtProfile::BLOB_EXT_4X_C32_32,
            ProfileName::BlobExt4xC128_14 => v3_ext::ExtProfile::BLOB_EXT_4X_C128_14,
            _ => return Err("v3_ext requires an extension profile".into()),
        };
        profile.whir_log_inv_rate = self.whir_log_inv_rate;
        profile.validate()?;
        Ok(profile)
    }

    fn selected_v4_precompile_profile(&self) -> Result<v4_precompile::ExtProfile, Box<dyn std::error::Error>> {
        match self.profile {
            ProfileName::BlobExt2x15 => Ok(v4_precompile::ExtProfile::BLOB_EXT_2X_15),
            _ => Err("v4_precompile currently supports --profile blob-ext-2x-15".into()),
        }
    }

    fn selected_v4_ext_profile(&self) -> Result<v4_ext::ExtProfile, Box<dyn std::error::Error>> {
        let mut profile = match self.profile {
            ProfileName::BlobExt2x1 => v4_ext::ExtProfile::BLOB_EXT_2X_1,
            ProfileName::BlobExt2x2 => v4_ext::ExtProfile::BLOB_EXT_2X_2,
            ProfileName::BlobExt2x4 => v4_ext::ExtProfile::BLOB_EXT_2X_4,
            ProfileName::BlobExt2x6 => v4_ext::ExtProfile::BLOB_EXT_2X_6,
            ProfileName::BlobExt2x8 => v4_ext::ExtProfile::BLOB_EXT_2X_8,
            ProfileName::BlobExt2x10 => v4_ext::ExtProfile::BLOB_EXT_2X_10,
            ProfileName::BlobExt2x12 => v4_ext::ExtProfile::BLOB_EXT_2X_12,
            ProfileName::BlobExt2x14 => v4_ext::ExtProfile::BLOB_EXT_2X_14,
            ProfileName::BlobExt2x16 => v4_ext::ExtProfile::BLOB_EXT_2X_16,
            ProfileName::BlobExt2x18 => v4_ext::ExtProfile::BLOB_EXT_2X_18,
            ProfileName::BlobExt2x20 => v4_ext::ExtProfile::BLOB_EXT_2X_20,
            ProfileName::BlobExt2x22 => v4_ext::ExtProfile::BLOB_EXT_2X_22,
            ProfileName::BlobExt2x24 => v4_ext::ExtProfile::BLOB_EXT_2X_24,
            ProfileName::BlobExt2x26 => v4_ext::ExtProfile::BLOB_EXT_2X_26,
            ProfileName::BlobExt2x28 => v4_ext::ExtProfile::BLOB_EXT_2X_28,
            ProfileName::BlobExt2x30 => v4_ext::ExtProfile::BLOB_EXT_2X_30,
            ProfileName::BlobExt2x32 => v4_ext::ExtProfile::BLOB_EXT_2X_32,
            _ => return Err("v4_ext currently supports 2x extension row-count profiles".into()),
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
    if cli.all_v4_ext_benchmarks {
        run_all_v4_ext_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }

    if cli.all_v4_precompile_benchmarks {
        run_all_v4_precompile_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v4_ext_row_count_sweep {
        run_v4_ext_row_count_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v3_ext_blob_size_sweep {
        run_v3_ext_blob_size_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v3_ext_cell_size_sweep {
        run_v3_ext_cell_size_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v3_ext_row_count_sweep {
        run_v3_ext_row_count_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v3_ext_whir_rate_sweep {
        run_v3_ext_whir_rate_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.all_v3_ext_cellsize_benchmarks {
        run_all_v3_ext_cellsize_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v3_ext_4x_cell_size_sweep {
        run_v3_ext_4x_cell_size_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.all_v3_ext_4x_cellsize_benchmarks {
        run_all_v3_ext_4x_cellsize_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.all_v3_ext_2x_row_benchmarks {
        run_all_v3_ext_2x_row_benchmarks(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.v3_ext_4x_row_count_sweep {
        run_v3_ext_4x_row_count_sweep(cli.skip_reconstruction)?;
        return Ok(());
    }
    if cli.all_v3_ext_4x_row_benchmarks {
        run_all_v3_ext_4x_row_benchmarks(cli.skip_reconstruction)?;
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
        VersionName::V4Ext => run_v4_ext_single(cli.selected_v4_ext_profile()?, cli.skip_reconstruction)?,
        VersionName::V4Precompile => {
            run_v4_precompile_single(cli.selected_v4_precompile_profile()?, cli.skip_reconstruction)?
        }
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
        ParameterProfile::BLOB_256K_1,
        ParameterProfile::BLOB_256K_14,
        ParameterProfile::BLOB_256K_16,
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
        v3_ext::ExtProfile::BLOB_EXT_2X_1,
        v3_ext::ExtProfile::BLOB_EXT_2X_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_16,
        v3_ext::ExtProfile::BLOB_EXT_4X_1,
        v3_ext::ExtProfile::BLOB_EXT_4X_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs one V4-ext benchmark and prints the detailed report plus VM counters.
fn run_v4_precompile_single(
    profile: v4_precompile::ExtProfile,
    skip_reconstruction: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v4_precompile_benchmark(profile, skip_reconstruction)?;
    print_v4_precompile_report(&result);
    Ok(())
}

fn run_all_v4_precompile_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [v4_precompile::ExtProfile::BLOB_EXT_2X_15];
    let mut results = Vec::new();
    for profile in profiles {
        results.push(run_v4_precompile_benchmark(profile, skip_reconstruction)?);
    }
    print_v4_precompile_table(&results);
    Ok(())
}

fn run_v4_ext_single(profile: v4_ext::ExtProfile, skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = run_v4_ext_benchmark(profile, skip_reconstruction)?;
    print_v4_ext_report(&result);
    Ok(())
}

/// Runs and prints the V4-ext benchmark table.
fn run_all_v4_ext_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [v4_ext::ExtProfile::BLOB_EXT_2X_14];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v4_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v4_ext_table(&results);
    Ok(())
}

/// Runs the V4-ext 2x row-count sweep for n=1,2,4,6,...,32.
fn run_v4_ext_row_count_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v4_ext::ExtProfile::BLOB_EXT_2X_1,
        v4_ext::ExtProfile::BLOB_EXT_2X_2,
        v4_ext::ExtProfile::BLOB_EXT_2X_4,
        v4_ext::ExtProfile::BLOB_EXT_2X_6,
        v4_ext::ExtProfile::BLOB_EXT_2X_8,
        v4_ext::ExtProfile::BLOB_EXT_2X_10,
        v4_ext::ExtProfile::BLOB_EXT_2X_12,
        v4_ext::ExtProfile::BLOB_EXT_2X_14,
        v4_ext::ExtProfile::BLOB_EXT_2X_16,
        v4_ext::ExtProfile::BLOB_EXT_2X_18,
        v4_ext::ExtProfile::BLOB_EXT_2X_20,
        v4_ext::ExtProfile::BLOB_EXT_2X_22,
        v4_ext::ExtProfile::BLOB_EXT_2X_24,
        v4_ext::ExtProfile::BLOB_EXT_2X_26,
        v4_ext::ExtProfile::BLOB_EXT_2X_28,
        v4_ext::ExtProfile::BLOB_EXT_2X_30,
        v4_ext::ExtProfile::BLOB_EXT_2X_32,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v4_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v4_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext blob-size sweep with ell=1024 and n=14 fixed.
fn run_v3_ext_blob_size_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_14,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext cell-size sweep with 2x blob size and n=14 fixed.
fn run_v3_ext_cell_size_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_2X_C16_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_C64_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_C128_14,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext row-count sweep with 2x blob size and c=32 fixed.
fn run_v3_ext_row_count_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_2X_1,
        v3_ext::ExtProfile::BLOB_EXT_2X_2,
        v3_ext::ExtProfile::BLOB_EXT_2X_4,
        v3_ext::ExtProfile::BLOB_EXT_2X_6,
        v3_ext::ExtProfile::BLOB_EXT_2X_8,
        v3_ext::ExtProfile::BLOB_EXT_2X_10,
        v3_ext::ExtProfile::BLOB_EXT_2X_12,
        v3_ext::ExtProfile::BLOB_EXT_2X_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_15,
        v3_ext::ExtProfile::BLOB_EXT_2X_16,
        v3_ext::ExtProfile::BLOB_EXT_2X_18,
        v3_ext::ExtProfile::BLOB_EXT_2X_20,
        v3_ext::ExtProfile::BLOB_EXT_2X_22,
        v3_ext::ExtProfile::BLOB_EXT_2X_24,
        v3_ext::ExtProfile::BLOB_EXT_2X_26,
        v3_ext::ExtProfile::BLOB_EXT_2X_28,
        v3_ext::ExtProfile::BLOB_EXT_2X_30,
        v3_ext::ExtProfile::BLOB_EXT_2X_32,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext WHIR-rate sweep that is valid under the default WHIR folding factors.
fn run_v3_ext_whir_rate_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let base_profiles = [v3_ext::ExtProfile::BLOB_EXT_2X_14, v3_ext::ExtProfile::BLOB_EXT_4X_14];
    let whir_log_inv_rates = [1, 2];
    let mut results = Vec::with_capacity(base_profiles.len() * whir_log_inv_rates.len());
    for base_profile in base_profiles {
        for whir_log_inv_rate in whir_log_inv_rates {
            let mut profile = base_profile;
            profile.whir_log_inv_rate = whir_log_inv_rate;
            results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
        }
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext double-blob cell-size comparison for c=32, c=64, and c=128.
fn run_all_v3_ext_cellsize_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_2X_1,
        v3_ext::ExtProfile::BLOB_EXT_2X_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_16,
        v3_ext::ExtProfile::BLOB_EXT_2X_C64_1,
        v3_ext::ExtProfile::BLOB_EXT_2X_C64_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_C64_16,
        v3_ext::ExtProfile::BLOB_EXT_2X_C128_1,
        v3_ext::ExtProfile::BLOB_EXT_2X_C128_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_C128_16,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext 4x cell-size comparison for c=32 and c=64.
fn run_v3_ext_4x_cell_size_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_4X_C16_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_C128_14,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext 4x cell-size comparison for c=16, c=32, c=64, and c=128.
fn run_all_v3_ext_4x_cellsize_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_4X_C16_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_1,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_16,
        v3_ext::ExtProfile::BLOB_EXT_4X_1,
        v3_ext::ExtProfile::BLOB_EXT_4X_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_16,
        v3_ext::ExtProfile::BLOB_EXT_4X_C128_14,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Runs the V3-ext 2x row-count sweep for n=1,2,4,6,...,32.
fn run_all_v3_ext_2x_row_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = [
        v3_ext::ExtProfile::BLOB_EXT_2X_1,
        v3_ext::ExtProfile::BLOB_EXT_2X_2,
        v3_ext::ExtProfile::BLOB_EXT_2X_4,
        v3_ext::ExtProfile::BLOB_EXT_2X_6,
        v3_ext::ExtProfile::BLOB_EXT_2X_8,
        v3_ext::ExtProfile::BLOB_EXT_2X_10,
        v3_ext::ExtProfile::BLOB_EXT_2X_12,
        v3_ext::ExtProfile::BLOB_EXT_2X_14,
        v3_ext::ExtProfile::BLOB_EXT_2X_15,
        v3_ext::ExtProfile::BLOB_EXT_2X_16,
        v3_ext::ExtProfile::BLOB_EXT_2X_18,
        v3_ext::ExtProfile::BLOB_EXT_2X_20,
        v3_ext::ExtProfile::BLOB_EXT_2X_22,
        v3_ext::ExtProfile::BLOB_EXT_2X_24,
        v3_ext::ExtProfile::BLOB_EXT_2X_26,
        v3_ext::ExtProfile::BLOB_EXT_2X_28,
        v3_ext::ExtProfile::BLOB_EXT_2X_30,
        v3_ext::ExtProfile::BLOB_EXT_2X_32,
    ];
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Returns the local-safe 4x, c=32 row-count sweep profiles.
fn v3_ext_4x_c32_row_profiles() -> [v3_ext::ExtProfile; 9] {
    [
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_1,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_2,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_4,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_6,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_8,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_10,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_12,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_14,
        v3_ext::ExtProfile::BLOB_EXT_4X_C32_16,
    ]
}

/// Runs the V3-ext 4x-c32 row-count sweep for n=1,2,4,6,8,10,12,14,16.
fn run_v3_ext_4x_row_count_sweep(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = v3_ext_4x_c32_row_profiles();
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        results.push(run_v3_ext_benchmark(profile, skip_reconstruction)?);
    }
    print_v3_ext_table(&results);
    Ok(())
}

/// Compatibility alias for the V3-ext 4x-c32 row-count sweep.
fn run_all_v3_ext_4x_row_benchmarks(skip_reconstruction: bool) -> Result<(), Box<dyn std::error::Error>> {
    run_v3_ext_4x_row_count_sweep(skip_reconstruction)
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

fn run_v4_precompile_benchmark(
    profile: v4_precompile::ExtProfile,
    skip_reconstruction: bool,
) -> Result<v4_precompile::ExtBenchmarkResult, Box<dyn std::error::Error>> {
    let data = v4_precompile::demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v4_precompile::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v4_precompile::prepare_statement(commitment)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v4_precompile::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v4_precompile::opened_cells(profile).min(profile.n_cells());
    let started = Instant::now();
    let indices =
        v4_precompile::sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v4_precompile::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v4_precompile::prepare_statement(prepared.commitment.clone())?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v4_precompile::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v4_precompile::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = v4_precompile::sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v4_precompile::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v4_precompile::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };

    Ok(v4_precompile::ExtBenchmarkResult {
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v4_precompile::ExtBenchmarkTimings {
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

fn run_v4_ext_benchmark(
    profile: v4_ext::ExtProfile,
    skip_reconstruction: bool,
) -> Result<v4_ext::ExtBenchmarkResult, Box<dyn std::error::Error>> {
    let data = v4_ext::demo_data(profile);

    let started = Instant::now();
    let (commitment, aux) = v4_ext::encode_and_commit(profile, &data)?;
    let encode_commit = started.elapsed();

    let started = Instant::now();
    let prepared = v4_ext::prepare_statement(commitment)?;
    let prover_preprocess = started.elapsed();

    let started = Instant::now();
    let proof = v4_ext::prove_codewords(&prepared, &aux.codewords)?;
    let prove = started.elapsed();

    let opened_cells = v4_ext::opened_cells(profile).min(profile.n_cells());
    let started = Instant::now();
    let indices = v4_ext::sample_query_indices(&prepared.commitment, &[F::from_u32(42); DIGEST_LEN], opened_cells)?;
    let transcript = v4_ext::query(&aux, &indices)?;
    let opening_generation = started.elapsed();

    let started = Instant::now();
    let verifier_prepared = v4_ext::prepare_statement(prepared.commitment.clone())?;
    let verifier_rebuild = started.elapsed();

    let started = Instant::now();
    v4_ext::verify_prepared_execution_proof(&verifier_prepared, &proof)?;
    let proof_verify = started.elapsed();

    let started = Instant::now();
    let opening_accepted = v4_ext::verify_openings(&prepared.commitment, &transcript);
    let verify_openings = started.elapsed();

    let (reconstruction, reconstruct_time) = if skip_reconstruction {
        (None, None)
    } else {
        let reconstruction_indices = v4_ext::sample_query_indices(
            &prepared.commitment,
            &[F::from_u32(84); DIGEST_LEN],
            profile.reconstruction_threshold_cells(),
        )?;
        let reconstruction_transcript = v4_ext::query(&aux, &reconstruction_indices)?;
        let started = Instant::now();
        let correct = v4_ext::reconstruct(&prepared.commitment, &[reconstruction_transcript])? == data;
        (Some(correct), Some(started.elapsed()))
    };

    Ok(v4_ext::ExtBenchmarkResult {
        profile,
        commitment: prepared.commitment.clone(),
        prepared,
        proof,
        transcript,
        opened_cells,
        reconstruction,
        timings: v4_ext::ExtBenchmarkTimings {
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
        r"| Profile | WHIR log inv rate | Bytecode instructions | Read-only elements | Opened cells | $\log_2\nu_{{\mathrm{{rep}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v3_ext_row(result));
    }
}

fn print_v4_precompile_report(result: &v4_precompile::ExtBenchmarkResult) {
    println!("PQ-DAS V4-precompile LeanVM demo");
    println!("{}", v4_precompile_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v4_precompile_table(results: &[v4_precompile::ExtBenchmarkResult]) {
    println!("PQ-DAS V4-precompile LeanVM benchmark table");
    println!(
        r"| Profile | WHIR log inv rate | Bytecode instructions | Read-only elements | Opened cells | $\log_2\nu_{{\mathrm{{rep}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v4_precompile_row(result));
    }
}

fn print_v4_ext_report(result: &v4_ext::ExtBenchmarkResult) {
    println!("PQ-DAS V4-ext LeanVM demo");
    println!("{}", v4_ext_row(result));
    if let Some(metadata) = &result.proof.execution.metadata {
        println!("VM cycles: {}", metadata.cycles);
        println!("Poseidon16 calls: {}", metadata.n_poseidons);
        println!("ExtensionOp calls: {}", metadata.n_extension_ops);
    }
}

fn print_v4_ext_table(results: &[v4_ext::ExtBenchmarkResult]) {
    println!("PQ-DAS V4-ext LeanVM benchmark table");
    println!(
        r"| Profile | WHIR log inv rate | Bytecode instructions | Read-only elements | Opened cells | $\log_2\nu_{{\mathrm{{rep}}}}$ | Commitment size | Proof size | Sample size | Encode + commit | Prover preprocess | LeanVM prove | Opening generation | Verifier rebuild | LeanVM verify | Verify openings | Reconstruct | VM cycles | Poseidon16 calls | ExtensionOp calls | LeanVM proving throughput | Full DAS throughput | Result |"
    );
    println!(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for result in results {
        println!("{}", v4_ext_row(result));
    }
}

fn v2_base_row(result: &v2_base::BenchmarkResult) -> String {
    let profile = result.profile;
    let proof_bytes = result.proof.serialized_size_bytes();
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
    let proof_bytes = result.proof.serialized_size_bytes();
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
    let proof_bytes = result.proof.serialized_size_bytes();
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
    let proof_bytes = result.proof.serialized_size_bytes();
    let commitment_bytes = v3_ext::commitment_size_bytes(&result.commitment);
    let sample_bytes = v3_ext::transcript_size_bytes(&result.transcript);
    let metadata = result.proof.execution.metadata.as_ref();
    let reconstruction = match result.reconstruction {
        Some(true) => format_duration(result.timings.reconstruct),
        Some(false) => "failed".to_string(),
        None => "skipped".to_string(),
    };
    let ok = result.accepted && result.reconstruction.unwrap_or(true);
    let leanvm_throughput = throughput_kib_per_sec(v3_ext_payload_bytes(profile), result.timings.prove.as_secs_f64());
    let full_throughput = full_das_throughput_kib_per_sec(result, commitment_bytes, proof_bytes, sample_bytes);
    format!(
        "| {} | {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {:.2} KiB/s | {:.2} KiB/s | {} |",
        profile.name,
        profile.whir_log_inv_rate,
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v3_ext::subset_log2_failure_with_replacement(profile, result.opened_cells),
        kb(commitment_bytes),
        kb(proof_bytes),
        kb(sample_bytes),
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
        leanvm_throughput,
        full_throughput,
        if ok { "accepted" } else { "failed" },
    )
}

fn v4_precompile_row(result: &v4_precompile::ExtBenchmarkResult) -> String {
    let profile = result.profile;
    let proof_bytes = result.proof.serialized_size_bytes();
    let commitment_bytes = v4_precompile::commitment_size_bytes(&result.commitment);
    let sample_bytes = v4_precompile::transcript_size_bytes(&result.transcript);
    let metadata = result.proof.execution.metadata.as_ref();
    let reconstruction = match result.reconstruction {
        Some(true) => format_duration(result.timings.reconstruct),
        Some(false) => "failed".to_string(),
        None => "skipped".to_string(),
    };
    let ok = result.accepted && result.reconstruction.unwrap_or(true);
    let leanvm_throughput =
        throughput_kib_per_sec(v4_precompile_payload_bytes(profile), result.timings.prove.as_secs_f64());
    let full_throughput =
        full_das_throughput_v4_precompile_kib_per_sec(result, commitment_bytes, proof_bytes, sample_bytes);
    format!(
        "| {} | {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {:.2} KiB/s | {:.2} KiB/s | {} |",
        profile.name,
        profile.whir_log_inv_rate,
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v4_precompile::subset_log2_failure_with_replacement(profile, result.opened_cells),
        kb(commitment_bytes),
        kb(proof_bytes),
        kb(sample_bytes),
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
        leanvm_throughput,
        full_throughput,
        if ok { "accepted" } else { "failed" },
    )
}

fn v4_ext_row(result: &v4_ext::ExtBenchmarkResult) -> String {
    let profile = result.profile;
    let proof_bytes = result.proof.serialized_size_bytes();
    let commitment_bytes = v4_ext::commitment_size_bytes(&result.commitment);
    let sample_bytes = v4_ext::transcript_size_bytes(&result.transcript);
    let metadata = result.proof.execution.metadata.as_ref();
    let reconstruction = match result.reconstruction {
        Some(true) => format_duration(result.timings.reconstruct),
        Some(false) => "failed".to_string(),
        None => "skipped".to_string(),
    };
    let ok = result.accepted && result.reconstruction.unwrap_or(true);
    let leanvm_throughput = throughput_kib_per_sec(v4_ext_payload_bytes(profile), result.timings.prove.as_secs_f64());
    let full_throughput = full_das_throughput_v4_ext_kib_per_sec(result, commitment_bytes, proof_bytes, sample_bytes);
    format!(
        "| {} | {} | {} | {} | {} | {:.3} | {} KB | {} KB | {} KB | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {:.3}s | {} | {} | {} | {} | {:.2} KiB/s | {:.2} KiB/s | {} |",
        profile.name,
        profile.whir_log_inv_rate,
        result.prepared.bytecode.size(),
        result.prepared.bytecode.read_only_data().len(),
        result.opened_cells,
        v4_ext::subset_log2_failure_with_replacement(profile, result.opened_cells),
        kb(commitment_bytes),
        kb(proof_bytes),
        kb(sample_bytes),
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
        leanvm_throughput,
        full_throughput,
        if ok { "accepted" } else { "failed" },
    )
}

fn format_duration(value: Option<Duration>) -> String {
    value
        .map(|duration| format!("{:.3}s", duration.as_secs_f64()))
        .unwrap_or_else(|| "n/a".to_string())
}

fn v3_ext_payload_bytes(profile: v3_ext::ExtProfile) -> usize {
    profile.n * profile.k * pq_das::EXT_DEGREE * 31 / 8
}

fn v4_ext_payload_bytes(profile: v4_ext::ExtProfile) -> usize {
    profile.n * profile.k * pq_das::EXT_DEGREE * 31 / 8
}

fn v4_precompile_payload_bytes(profile: v4_precompile::ExtProfile) -> usize {
    profile.n * profile.k * pq_das::EXT_DEGREE * 31 / 8
}

fn throughput_kib_per_sec(payload_bytes: usize, seconds: f64) -> f64 {
    if seconds == 0.0 {
        return f64::INFINITY;
    }
    payload_bytes as f64 / 1024.0 / seconds
}

fn full_das_throughput_kib_per_sec(
    result: &v3_ext::ExtBenchmarkResult,
    commitment_bytes: usize,
    proof_bytes: usize,
    sample_bytes: usize,
) -> f64 {
    const BANDWIDTH_BYTES_PER_SEC: f64 = 50_000_000.0 / 8.0;
    let profile = result.profile;
    let payload_bytes = v3_ext_payload_bytes(profile);
    let codeword_bytes = profile.n * profile.m * pq_das::EXT_DEGREE * size_of::<u32>();
    let upload_bytes = codeword_bytes + commitment_bytes + proof_bytes;
    let download_bytes = commitment_bytes + proof_bytes + sample_bytes;

    // Benedikt-style DA throughput: useful payload divided by builder-receives-data-to-validator-accepts latency.
    let network_time = (upload_bytes + download_bytes) as f64 / BANDWIDTH_BYTES_PER_SEC;
    let timings = &result.timings;
    let compute_time = timings.encode_commit.as_secs_f64()
        + timings.prover_preprocess.as_secs_f64()
        + timings.prove.as_secs_f64()
        + timings.opening_generation.as_secs_f64()
        + timings.verifier_rebuild.as_secs_f64()
        + timings.proof_verify.as_secs_f64()
        + timings.verify_openings.as_secs_f64();
    throughput_kib_per_sec(payload_bytes, compute_time + network_time)
}

fn full_das_throughput_v4_precompile_kib_per_sec(
    result: &v4_precompile::ExtBenchmarkResult,
    commitment_bytes: usize,
    proof_bytes: usize,
    sample_bytes: usize,
) -> f64 {
    const BANDWIDTH_BYTES_PER_SEC: f64 = 50_000_000.0 / 8.0;
    let profile = result.profile;
    let payload_bytes = v4_precompile_payload_bytes(profile);
    let codeword_bytes = profile.n * profile.m * pq_das::EXT_DEGREE * size_of::<u32>();
    let upload_bytes = codeword_bytes + commitment_bytes + proof_bytes;
    let download_bytes = commitment_bytes + proof_bytes + sample_bytes;

    // Benedikt-style DA throughput: useful payload divided by builder-receives-data-to-validator-accepts latency.
    let network_time = (upload_bytes + download_bytes) as f64 / BANDWIDTH_BYTES_PER_SEC;
    let timings = &result.timings;
    let compute_time = timings.encode_commit.as_secs_f64()
        + timings.prover_preprocess.as_secs_f64()
        + timings.prove.as_secs_f64()
        + timings.opening_generation.as_secs_f64()
        + timings.verifier_rebuild.as_secs_f64()
        + timings.proof_verify.as_secs_f64()
        + timings.verify_openings.as_secs_f64();
    throughput_kib_per_sec(payload_bytes, compute_time + network_time)
}

fn full_das_throughput_v4_ext_kib_per_sec(
    result: &v4_ext::ExtBenchmarkResult,
    commitment_bytes: usize,
    proof_bytes: usize,
    sample_bytes: usize,
) -> f64 {
    const BANDWIDTH_BYTES_PER_SEC: f64 = 50_000_000.0 / 8.0;
    let profile = result.profile;
    let payload_bytes = v4_ext_payload_bytes(profile);
    let codeword_bytes = profile.n * profile.m * pq_das::EXT_DEGREE * size_of::<u32>();
    let upload_bytes = codeword_bytes + commitment_bytes + proof_bytes;
    let download_bytes = commitment_bytes + proof_bytes + sample_bytes;

    let network_time = (upload_bytes + download_bytes) as f64 / BANDWIDTH_BYTES_PER_SEC;
    let timings = &result.timings;
    let compute_time = timings.encode_commit.as_secs_f64()
        + timings.prover_preprocess.as_secs_f64()
        + timings.prove.as_secs_f64()
        + timings.opening_generation.as_secs_f64()
        + timings.verifier_rebuild.as_secs_f64()
        + timings.proof_verify.as_secs_f64()
        + timings.verify_openings.as_secs_f64();
    throughput_kib_per_sec(payload_bytes, compute_time + network_time)
}

fn kb(bytes: usize) -> String {
    format!("{:.2}", bytes as f64 / 1024.0)
}
