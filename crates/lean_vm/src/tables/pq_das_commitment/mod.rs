use crate::execution::memory::MemoryAccess;
use crate::*;
use backend::*;

mod air;
use air::*;

pub const PQ_DAS_COMMITMENT_NAME: &str = "pq_das_commitment";
pub const PQ_DAS_COMMITMENT_DOMAINSEP: usize = 10;

pub const PQ_DAS_COMMITMENT_BASELINE_N: usize = 15;
pub const PQ_DAS_COMMITMENT_BASELINE_M_EXT: usize = 32_768;
pub const PQ_DAS_COMMITMENT_BASELINE_C_EXT: usize = 32;

const SUPPORTED_COMMITMENT_PROFILES: &[(usize, usize, usize)] = &[
    (15, 32_768, 32),
    (14, 32_768, 128),
    (30, 32_768, 32),
    (14, 65_536, 128),
    (14, 65_536, 64),
];

pub const PQ_DAS_COMMITMENT_SCRATCH_LEN: usize = 5_079_064;

#[derive(Debug, Clone, Copy)]
struct CommitmentProfile {
    rows: usize,
    rows_padded: usize,
    row_len: usize,
    k: usize,
    cell_size: usize,
    cell_base_len: usize,
    n_cells: usize,
    systematic_cells: usize,
    cell_chunks: usize,
    row_stride: usize,
    row_hashes_offset: usize,
    column_roots_offset: usize,
    cell_temps_offset: usize,
    cell_temps_per_cell: usize,
    row_temps_offset: usize,
    row_temps_per_row: usize,
    column_temps_offset: usize,
    column_temps_per_column: usize,
    row_root_temps_offset: usize,
    outer_temps_offset: usize,
    zero_digest_offset: usize,
    scratch_len: usize,
}

impl CommitmentProfile {
    fn supported(rows: usize, row_len: usize, cell_size: usize) -> Option<Self> {
        if !SUPPORTED_COMMITMENT_PROFILES.contains(&(rows, row_len, cell_size)) {
            return None;
        }
        if row_len % cell_size != 0 || (row_len / 2) % cell_size != 0 {
            return None;
        }
        let rows_padded = rows.next_power_of_two();
        let k = row_len / 2;
        let cell_base_len = cell_size * DIMENSION;
        let n_cells = row_len / cell_size;
        let systematic_cells = k / cell_size;
        let cell_chunks = cell_base_len / DIGEST_LEN;
        if rows_padded < 2 || !rows_padded.is_power_of_two() || n_cells < 2 || !n_cells.is_power_of_two() {
            return None;
        }
        if cell_chunks < 2 || systematic_cells < 2 {
            return None;
        }

        let row_stride = row_len * DIMENSION;
        let cell_digests_len = n_cells * rows_padded * DIGEST_LEN;
        let row_hashes_offset = cell_digests_len;
        let row_hashes_len = rows_padded * DIGEST_LEN;
        let column_roots_offset = row_hashes_offset + row_hashes_len;
        let column_roots_len = n_cells * DIGEST_LEN;
        let cell_temps_offset = column_roots_offset + column_roots_len;
        let cell_temps_per_cell = (cell_chunks - 2) * DIGEST_LEN;
        let cell_temps_len = rows * n_cells * cell_temps_per_cell;
        let row_temps_offset = cell_temps_offset + cell_temps_len;
        let row_temps_per_row = (systematic_cells - 2) * DIGEST_LEN;
        let row_temps_len = rows * row_temps_per_row;
        let column_temps_offset = row_temps_offset + row_temps_len;
        let column_temps_per_column = (rows_padded - 1) * DIGEST_LEN;
        let column_temps_len = n_cells * column_temps_per_column;
        let row_root_temps_offset = column_temps_offset + column_temps_len;
        let row_root_temps_len = (rows_padded - 1) * DIGEST_LEN;
        let outer_temps_offset = row_root_temps_offset + row_root_temps_len;
        let outer_temps_len = (n_cells - 1) * DIGEST_LEN;
        let zero_digest_offset = outer_temps_offset + outer_temps_len;
        let scratch_len = zero_digest_offset + DIGEST_LEN;
        if scratch_len > PQ_DAS_COMMITMENT_SCRATCH_LEN {
            return None;
        }

        Some(Self {
            rows,
            rows_padded,
            row_len,
            k,
            cell_size,
            cell_base_len,
            n_cells,
            systematic_cells,
            cell_chunks,
            row_stride,
            row_hashes_offset,
            column_roots_offset,
            cell_temps_offset,
            cell_temps_per_cell,
            row_temps_offset,
            row_temps_per_row,
            column_temps_offset,
            column_temps_per_column,
            row_root_temps_offset,
            outer_temps_offset,
            zero_digest_offset,
            scratch_len,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PqDasCommitmentPrecompile<const BUS: bool>;

impl<const BUS: bool> TableT for PqDasCommitmentPrecompile<BUS> {
    fn name(&self) -> &'static str {
        "pq_das_commitment"
    }
    fn table(&self) -> Table {
        Table::pq_das_commitment()
    }
    fn n_columns_total(&self) -> usize {
        NUM_COLS_TOTAL_PQ_DAS_COMMITMENT
    }

    fn bus_interactions(&self) -> Vec<BusInteraction> {
        vec![
            BusInteraction {
                direction: BusDirection::Pull,
                multiplicity: BusMultiplicity::Column(COL_PQ_COM_EXEC_MULTIPLICITY),
                domainsep: BusData::Constant(PQ_DAS_COMMITMENT_DOMAINSEP),
                data: vec![
                    BusData::Column(COL_PQ_COM_CODEWORD_BASE),
                    BusData::Column(COL_PQ_COM_PUBLIC_ROOT_PTR),
                    BusData::Column(COL_PQ_COM_SCRATCH_BASE),
                ],
            },
            BusInteraction {
                direction: BusDirection::Push,
                multiplicity: BusMultiplicity::Column(COL_PQ_COM_ACTIVE),
                domainsep: BusData::Constant(poseidon_compress_half_domainsep()),
                data: vec![
                    BusData::Column(COL_PQ_COM_IDX_A),
                    BusData::Column(COL_PQ_COM_IDX_B),
                    BusData::Column(COL_PQ_COM_IDX_RES),
                ],
            },
        ]
    }

    fn padding_row(&self, zero_vec_ptr: usize, _null_hash_ptr: usize, _ending_pc: usize) -> Vec<F> {
        let mut row = vec![F::ZERO; NUM_COLS_TOTAL_PQ_DAS_COMMITMENT];
        row[COL_PQ_COM_PUBLIC_ROOT_PTR] = F::from_usize(zero_vec_ptr);
        row[COL_PQ_COM_SCRATCH_BASE] = F::from_usize(zero_vec_ptr);
        row[COL_PQ_COM_IDX_A] = F::from_usize(zero_vec_ptr);
        row[COL_PQ_COM_IDX_B] = F::from_usize(zero_vec_ptr);
        row[COL_PQ_COM_IDX_RES] = F::from_usize(zero_vec_ptr);
        row
    }

    fn execute<M: MemoryAccess>(
        &self,
        codeword_base: F,
        public_root_ptr: F,
        scratch_base: F,
        args: PrecompileCompTimeArgs<usize>,
        ctx: &mut InstructionContext<'_, M>,
    ) -> Result<(), RunnerError> {
        let PrecompileCompTimeArgs::PqDasCommitment {
            rows,
            row_len,
            cell_size,
        } = args
        else {
            unreachable!("PqDasCommitment table called with non-PQ-DAS args");
        };
        let Some(profile) = CommitmentProfile::supported(rows, row_len, cell_size) else {
            return Err(RunnerError::InvalidExtensionOp);
        };
        for k in 0..DIGEST_LEN {
            ctx.memory
                .set(scratch_base.to_usize() + profile.zero_digest_offset + k, F::ZERO)?;
        }
        let mut relay = CommitmentRelay::new(
            profile,
            codeword_base.to_usize(),
            public_root_ptr.to_usize(),
            scratch_base.to_usize(),
        );
        relay.run(ctx)
    }
}

struct CommitmentRelay {
    profile: CommitmentProfile,
    codeword_base: usize,
    public_root_ptr: usize,
    scratch_base: usize,
    step: usize,
}

impl CommitmentRelay {
    fn new(profile: CommitmentProfile, codeword_base: usize, public_root_ptr: usize, scratch_base: usize) -> Self {
        Self {
            profile,
            codeword_base,
            public_root_ptr,
            scratch_base,
            step: 0,
        }
    }

    fn s(&self, offset: usize) -> usize {
        self.scratch_base + offset
    }
    fn cell_digest(&self, cell: usize, row: usize) -> usize {
        self.s(cell * self.profile.rows_padded * DIGEST_LEN + row * DIGEST_LEN)
    }
    fn row_hash(&self, row: usize) -> usize {
        self.s(self.profile.row_hashes_offset + row * DIGEST_LEN)
    }
    fn column_root(&self, cell: usize) -> usize {
        self.s(self.profile.column_roots_offset + cell * DIGEST_LEN)
    }
    fn cell_temp(&self, row: usize, cell: usize, temp: usize) -> usize {
        self.s(self.profile.cell_temps_offset
            + (row * self.profile.n_cells + cell) * self.profile.cell_temps_per_cell
            + temp * DIGEST_LEN)
    }
    fn row_temp(&self, row: usize, temp: usize) -> usize {
        self.s(self.profile.row_temps_offset + row * self.profile.row_temps_per_row + temp * DIGEST_LEN)
    }
    fn column_temp(&self, cell: usize, temp: usize) -> usize {
        self.s(self.profile.column_temps_offset + cell * self.profile.column_temps_per_column + temp * DIGEST_LEN)
    }
    fn row_root_temp(&self, temp: usize) -> usize {
        self.s(self.profile.row_root_temps_offset + temp * DIGEST_LEN)
    }
    fn outer_temp(&self, temp: usize) -> usize {
        self.s(self.profile.outer_temps_offset + temp * DIGEST_LEN)
    }
    fn zero_digest(&self) -> usize {
        self.s(self.profile.zero_digest_offset)
    }

    fn push_call<M: MemoryAccess>(
        &mut self,
        a: usize,
        b: usize,
        c: usize,
        ctx: &mut InstructionContext<'_, M>,
    ) -> Result<(), RunnerError> {
        Table::poseidon16().execute(
            F::from_usize(a),
            F::from_usize(b),
            F::from_usize(c),
            PrecompileCompTimeArgs::Poseidon16 {
                half_output: false,
                hardcoded_offset_left: None,
                permute: false,
            },
            ctx,
        )?;
        let trace = ctx.traces.get_mut(&Table::pq_das_commitment()).unwrap();
        trace.columns[COL_PQ_COM_ACTIVE].push(F::ONE);
        trace.columns[COL_PQ_COM_EXEC_MULTIPLICITY].push(F::from_bool(self.step == 0));
        trace.columns[COL_PQ_COM_STEP].push(F::from_usize(self.step));
        trace.columns[COL_PQ_COM_CODEWORD_BASE].push(F::from_usize(self.codeword_base));
        trace.columns[COL_PQ_COM_PUBLIC_ROOT_PTR].push(F::from_usize(self.public_root_ptr));
        trace.columns[COL_PQ_COM_SCRATCH_BASE].push(F::from_usize(self.scratch_base));
        trace.columns[COL_PQ_COM_IDX_A].push(F::from_usize(a));
        trace.columns[COL_PQ_COM_IDX_B].push(F::from_usize(b));
        trace.columns[COL_PQ_COM_IDX_RES].push(F::from_usize(c));
        self.step += 1;
        Ok(())
    }

    fn hash_cell<M: MemoryAccess>(
        &mut self,
        row: usize,
        cell: usize,
        input: usize,
        dest: usize,
        ctx: &mut InstructionContext<'_, M>,
    ) -> Result<(), RunnerError> {
        if self.profile.cell_chunks == 2 {
            return self.push_call(input, input + DIGEST_LEN, dest, ctx);
        }
        let first = self.cell_temp(row, cell, 0);
        self.push_call(input, input + DIGEST_LEN, first, ctx)?;
        for chunk in 1..(self.profile.cell_chunks - 2) {
            let prev = self.cell_temp(row, cell, chunk - 1);
            let out = self.cell_temp(row, cell, chunk);
            self.push_call(prev, input + (chunk + 1) * DIGEST_LEN, out, ctx)?;
        }
        let last_temp = self.cell_temp(row, cell, self.profile.cell_chunks - 3);
        self.push_call(
            last_temp,
            input + (self.profile.cell_chunks - 1) * DIGEST_LEN,
            dest,
            ctx,
        )
    }

    fn merkle_power_of_two<M: MemoryAccess>(
        &mut self,
        leaves: Vec<usize>,
        temps_start: usize,
        dest: usize,
        ctx: &mut InstructionContext<'_, M>,
    ) -> Result<usize, RunnerError> {
        debug_assert!(leaves.len().is_power_of_two());
        let mut current = leaves;
        let mut cursor = 0usize;
        while current.len() > 1 {
            let next_len = current.len() / 2;
            let mut next = Vec::with_capacity(next_len);
            for node in 0..next_len {
                let out = if next_len == 1 {
                    dest
                } else {
                    let temp = temps_start + cursor * DIGEST_LEN;
                    cursor += 1;
                    temp
                };
                self.push_call(current[2 * node], current[2 * node + 1], out, ctx)?;
                next.push(out);
            }
            current = next;
        }
        Ok(current[0])
    }

    fn run<M: MemoryAccess>(&mut self, ctx: &mut InstructionContext<'_, M>) -> Result<(), RunnerError> {
        for row in 0..self.profile.rows {
            let row_base = self.codeword_base + row * self.profile.row_stride;
            for cell in 0..self.profile.n_cells {
                let input = row_base + cell * self.profile.cell_base_len;
                let dest = self.cell_digest(cell, row);
                self.hash_cell(row, cell, input, dest, ctx)?;
            }
            if self.profile.systematic_cells == 2 {
                self.push_call(
                    self.cell_digest(0, row),
                    self.cell_digest(1, row),
                    self.row_hash(row),
                    ctx,
                )?;
            } else {
                let row_state0 = self.row_temp(row, 0);
                self.push_call(self.cell_digest(0, row), self.cell_digest(1, row), row_state0, ctx)?;
                for cell in 2..(self.profile.systematic_cells - 1) {
                    self.push_call(
                        self.row_temp(row, cell - 2),
                        self.cell_digest(cell, row),
                        self.row_temp(row, cell - 1),
                        ctx,
                    )?;
                }
                self.push_call(
                    self.row_temp(row, self.profile.systematic_cells - 3),
                    self.cell_digest(self.profile.systematic_cells - 1, row),
                    self.row_hash(row),
                    ctx,
                )?;
            }
        }
        for row in self.profile.rows..self.profile.rows_padded {
            for k in 0..DIGEST_LEN {
                ctx.memory.set(self.row_hash(row) + k, F::ZERO)?;
            }
        }

        let row_leaves = (0..self.profile.rows_padded).map(|i| self.row_hash(i)).collect();
        let root_row = self.merkle_power_of_two(
            row_leaves,
            self.row_root_temp(0),
            self.row_root_temp(self.profile.rows_padded - 2),
            ctx,
        )?;

        for cell in 0..self.profile.n_cells {
            let leaves = (0..self.profile.rows_padded)
                .map(|row| {
                    if row < self.profile.rows {
                        self.cell_digest(cell, row)
                    } else {
                        self.zero_digest()
                    }
                })
                .collect();
            self.merkle_power_of_two(leaves, self.column_temp(cell, 0), self.column_root(cell), ctx)?;
        }

        let column_leaves = (0..self.profile.n_cells).map(|cell| self.column_root(cell)).collect();
        let root_col = self.merkle_power_of_two(
            column_leaves,
            self.outer_temp(0),
            self.outer_temp(self.profile.n_cells - 2),
            ctx,
        )?;
        self.push_call(root_row, root_col, self.public_root_ptr, ctx)
    }
}

pub const fn poseidon_compress_half_domainsep() -> usize {
    POSEIDON_DOMAINSEP_BASE + POSEIDON_FLAG_OUT8_SHIFT
}
