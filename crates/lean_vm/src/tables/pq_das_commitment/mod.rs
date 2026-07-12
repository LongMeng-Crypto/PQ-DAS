use crate::execution::memory::MemoryAccess;
use crate::*;
use backend::*;

mod air;
use air::*;

pub const PQ_DAS_COMMITMENT_NAME: &str = "pq_das_commitment";
pub const PQ_DAS_COMMITMENT_DOMAINSEP: usize = 10;

pub const PQ_DAS_COMMITMENT_N: usize = 15;
pub const PQ_DAS_COMMITMENT_N_PADDED: usize = 16;
pub const PQ_DAS_COMMITMENT_M_EXT: usize = 32_768;
pub const PQ_DAS_COMMITMENT_K_EXT: usize = 16_384;
pub const PQ_DAS_COMMITMENT_C_EXT: usize = 32;
pub const PQ_DAS_COMMITMENT_CELL_BASE_LEN: usize = PQ_DAS_COMMITMENT_C_EXT * DIMENSION;
pub const PQ_DAS_COMMITMENT_N_CELLS: usize = PQ_DAS_COMMITMENT_M_EXT / PQ_DAS_COMMITMENT_C_EXT;
pub const PQ_DAS_COMMITMENT_SYSTEMATIC_CELLS: usize = PQ_DAS_COMMITMENT_K_EXT / PQ_DAS_COMMITMENT_C_EXT;
pub const PQ_DAS_COMMITMENT_CELL_CHUNKS: usize = PQ_DAS_COMMITMENT_CELL_BASE_LEN / DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_ROW_STRIDE: usize = PQ_DAS_COMMITMENT_M_EXT * DIMENSION;

pub const PQ_DAS_COMMITMENT_CELL_DIGESTS_LEN: usize =
    PQ_DAS_COMMITMENT_N_CELLS * PQ_DAS_COMMITMENT_N_PADDED * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_ROW_HASHES_OFFSET: usize = PQ_DAS_COMMITMENT_CELL_DIGESTS_LEN;
pub const PQ_DAS_COMMITMENT_ROW_HASHES_LEN: usize = PQ_DAS_COMMITMENT_N_PADDED * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_COLUMN_ROOTS_OFFSET: usize =
    PQ_DAS_COMMITMENT_ROW_HASHES_OFFSET + PQ_DAS_COMMITMENT_ROW_HASHES_LEN;
pub const PQ_DAS_COMMITMENT_COLUMN_ROOTS_LEN: usize = PQ_DAS_COMMITMENT_N_CELLS * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_CELL_TEMPS_OFFSET: usize =
    PQ_DAS_COMMITMENT_COLUMN_ROOTS_OFFSET + PQ_DAS_COMMITMENT_COLUMN_ROOTS_LEN;
pub const PQ_DAS_COMMITMENT_CELL_TEMPS_PER_CELL: usize = (PQ_DAS_COMMITMENT_CELL_CHUNKS - 2) * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_CELL_TEMPS_LEN: usize =
    PQ_DAS_COMMITMENT_N * PQ_DAS_COMMITMENT_N_CELLS * PQ_DAS_COMMITMENT_CELL_TEMPS_PER_CELL;
pub const PQ_DAS_COMMITMENT_ROW_TEMPS_OFFSET: usize =
    PQ_DAS_COMMITMENT_CELL_TEMPS_OFFSET + PQ_DAS_COMMITMENT_CELL_TEMPS_LEN;
pub const PQ_DAS_COMMITMENT_ROW_TEMPS_PER_ROW: usize = (PQ_DAS_COMMITMENT_SYSTEMATIC_CELLS - 2) * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_ROW_TEMPS_LEN: usize = PQ_DAS_COMMITMENT_N * PQ_DAS_COMMITMENT_ROW_TEMPS_PER_ROW;
pub const PQ_DAS_COMMITMENT_COLUMN_TEMPS_OFFSET: usize =
    PQ_DAS_COMMITMENT_ROW_TEMPS_OFFSET + PQ_DAS_COMMITMENT_ROW_TEMPS_LEN;
pub const PQ_DAS_COMMITMENT_COLUMN_TEMPS_PER_COLUMN: usize = 15 * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_COLUMN_TEMPS_LEN: usize =
    PQ_DAS_COMMITMENT_N_CELLS * PQ_DAS_COMMITMENT_COLUMN_TEMPS_PER_COLUMN;
pub const PQ_DAS_COMMITMENT_ROW_ROOT_TEMPS_OFFSET: usize =
    PQ_DAS_COMMITMENT_COLUMN_TEMPS_OFFSET + PQ_DAS_COMMITMENT_COLUMN_TEMPS_LEN;
pub const PQ_DAS_COMMITMENT_ROW_ROOT_TEMPS_LEN: usize = 15 * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_OUTER_TEMPS_OFFSET: usize =
    PQ_DAS_COMMITMENT_ROW_ROOT_TEMPS_OFFSET + PQ_DAS_COMMITMENT_ROW_ROOT_TEMPS_LEN;
pub const PQ_DAS_COMMITMENT_OUTER_TEMPS_LEN: usize = (PQ_DAS_COMMITMENT_N_CELLS - 1) * DIGEST_LEN;
pub const PQ_DAS_COMMITMENT_ZERO_DIGEST_OFFSET: usize =
    PQ_DAS_COMMITMENT_OUTER_TEMPS_OFFSET + PQ_DAS_COMMITMENT_OUTER_TEMPS_LEN;
pub const PQ_DAS_COMMITMENT_SCRATCH_LEN: usize = PQ_DAS_COMMITMENT_ZERO_DIGEST_OFFSET + DIGEST_LEN;

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
        if rows != PQ_DAS_COMMITMENT_N || row_len != PQ_DAS_COMMITMENT_M_EXT || cell_size != PQ_DAS_COMMITMENT_C_EXT {
            return Err(RunnerError::InvalidExtensionOp);
        }
        for k in 0..DIGEST_LEN {
            ctx.memory.set(
                scratch_base.to_usize() + PQ_DAS_COMMITMENT_ZERO_DIGEST_OFFSET + k,
                F::ZERO,
            )?;
        }
        let mut relay = CommitmentRelay::new(
            codeword_base.to_usize(),
            public_root_ptr.to_usize(),
            scratch_base.to_usize(),
        );
        relay.run(ctx)
    }
}

struct CommitmentRelay {
    codeword_base: usize,
    public_root_ptr: usize,
    scratch_base: usize,
    step: usize,
}

impl CommitmentRelay {
    fn new(codeword_base: usize, public_root_ptr: usize, scratch_base: usize) -> Self {
        Self {
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
        self.s(cell * PQ_DAS_COMMITMENT_N_PADDED * DIGEST_LEN + row * DIGEST_LEN)
    }
    fn row_hash(&self, row: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_ROW_HASHES_OFFSET + row * DIGEST_LEN)
    }
    fn column_root(&self, cell: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_COLUMN_ROOTS_OFFSET + cell * DIGEST_LEN)
    }
    fn cell_temp(&self, row: usize, cell: usize, temp: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_CELL_TEMPS_OFFSET
            + (row * PQ_DAS_COMMITMENT_N_CELLS + cell) * PQ_DAS_COMMITMENT_CELL_TEMPS_PER_CELL
            + temp * DIGEST_LEN)
    }
    fn row_temp(&self, row: usize, temp: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_ROW_TEMPS_OFFSET + row * PQ_DAS_COMMITMENT_ROW_TEMPS_PER_ROW + temp * DIGEST_LEN)
    }
    fn column_temp(&self, cell: usize, temp: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_COLUMN_TEMPS_OFFSET
            + cell * PQ_DAS_COMMITMENT_COLUMN_TEMPS_PER_COLUMN
            + temp * DIGEST_LEN)
    }
    fn row_root_temp(&self, temp: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_ROW_ROOT_TEMPS_OFFSET + temp * DIGEST_LEN)
    }
    fn outer_temp(&self, temp: usize) -> usize {
        self.s(PQ_DAS_COMMITMENT_OUTER_TEMPS_OFFSET + temp * DIGEST_LEN)
    }
    fn zero_digest(&self) -> usize {
        self.s(PQ_DAS_COMMITMENT_ZERO_DIGEST_OFFSET)
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
        let first = self.cell_temp(row, cell, 0);
        self.push_call(input, input + DIGEST_LEN, first, ctx)?;
        for chunk in 1..(PQ_DAS_COMMITMENT_CELL_CHUNKS - 2) {
            let prev = self.cell_temp(row, cell, chunk - 1);
            let out = self.cell_temp(row, cell, chunk);
            self.push_call(prev, input + (chunk + 1) * DIGEST_LEN, out, ctx)?;
        }
        let last_temp = self.cell_temp(row, cell, PQ_DAS_COMMITMENT_CELL_CHUNKS - 3);
        self.push_call(
            last_temp,
            input + (PQ_DAS_COMMITMENT_CELL_CHUNKS - 1) * DIGEST_LEN,
            dest,
            ctx,
        )
    }

    fn merkle16<M: MemoryAccess>(
        &mut self,
        leaves: &[usize; 16],
        dest: usize,
        temps: &[usize; 15],
        ctx: &mut InstructionContext<'_, M>,
    ) -> Result<(), RunnerError> {
        for node in 0..8 {
            self.push_call(leaves[2 * node], leaves[2 * node + 1], temps[node], ctx)?;
        }
        for node in 0..4 {
            self.push_call(temps[2 * node], temps[2 * node + 1], temps[8 + node], ctx)?;
        }
        for node in 0..2 {
            self.push_call(temps[8 + 2 * node], temps[8 + 2 * node + 1], temps[12 + node], ctx)?;
        }
        self.push_call(temps[12], temps[13], dest, ctx)
    }

    fn run<M: MemoryAccess>(&mut self, ctx: &mut InstructionContext<'_, M>) -> Result<(), RunnerError> {
        for row in 0..PQ_DAS_COMMITMENT_N {
            let row_base = self.codeword_base + row * PQ_DAS_COMMITMENT_ROW_STRIDE;
            for cell in 0..PQ_DAS_COMMITMENT_N_CELLS {
                let input = row_base + cell * PQ_DAS_COMMITMENT_CELL_BASE_LEN;
                let dest = self.cell_digest(cell, row);
                self.hash_cell(row, cell, input, dest, ctx)?;
            }
            let row_state0 = self.row_temp(row, 0);
            self.push_call(self.cell_digest(0, row), self.cell_digest(1, row), row_state0, ctx)?;
            for cell in 2..(PQ_DAS_COMMITMENT_SYSTEMATIC_CELLS - 1) {
                self.push_call(
                    self.row_temp(row, cell - 2),
                    self.cell_digest(cell, row),
                    self.row_temp(row, cell - 1),
                    ctx,
                )?;
            }
            self.push_call(
                self.row_temp(row, PQ_DAS_COMMITMENT_SYSTEMATIC_CELLS - 3),
                self.cell_digest(PQ_DAS_COMMITMENT_SYSTEMATIC_CELLS - 1, row),
                self.row_hash(row),
                ctx,
            )?;
        }
        for row in PQ_DAS_COMMITMENT_N..PQ_DAS_COMMITMENT_N_PADDED {
            for k in 0..DIGEST_LEN {
                ctx.memory.set(self.row_hash(row) + k, F::ZERO)?;
            }
        }

        let row_leaves: [usize; 16] = std::array::from_fn(|i| self.row_hash(i));
        let row_temps: [usize; 15] = std::array::from_fn(|i| self.row_root_temp(i));
        let root_row = self.row_root_temp(14);
        self.merkle16(&row_leaves, root_row, &row_temps, ctx)?;

        for cell in 0..PQ_DAS_COMMITMENT_N_CELLS {
            let leaves: [usize; 16] = std::array::from_fn(|row| {
                if row < PQ_DAS_COMMITMENT_N {
                    self.cell_digest(cell, row)
                } else {
                    self.zero_digest()
                }
            });
            let column_temps: [usize; 15] = std::array::from_fn(|i| self.column_temp(cell, i));
            self.merkle16(&leaves, self.column_root(cell), &column_temps, ctx)?;
        }

        let mut current_start = PQ_DAS_COMMITMENT_COLUMN_ROOTS_OFFSET;
        let mut current_count = PQ_DAS_COMMITMENT_N_CELLS;
        let mut out_cursor = 0usize;
        while current_count > 1 {
            let next_count = current_count / 2;
            for node in 0..next_count {
                let left = self.s(current_start + (2 * node) * DIGEST_LEN);
                let right = self.s(current_start + (2 * node + 1) * DIGEST_LEN);
                let out = self.outer_temp(out_cursor + node);
                self.push_call(left, right, out, ctx)?;
            }
            current_start = PQ_DAS_COMMITMENT_OUTER_TEMPS_OFFSET + out_cursor * DIGEST_LEN;
            out_cursor += next_count;
            current_count = next_count;
        }
        let root_col = self.outer_temp(out_cursor - 1);
        self.push_call(root_row, root_col, self.public_root_ptr, ctx)
    }
}

pub const fn poseidon_compress_half_domainsep() -> usize {
    POSEIDON_DOMAINSEP_BASE + POSEIDON_FLAG_OUT8_SHIFT
}
