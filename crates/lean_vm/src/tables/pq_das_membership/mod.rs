use crate::execution::memory::MemoryAccess;
use crate::*;
use backend::*;

mod air;
use air::*;

pub const PQ_DAS_MEMBERSHIP_BATCH_NAME: &str = "pq_das_membership_batch";
pub const PQ_DAS_MEMBERSHIP_DOMAINSEP: usize = 6;

const SUPPORTED_MEMBERSHIP_PROFILES: &[(usize, usize)] = &[(15, 32_768), (14, 32_768), (30, 32_768), (14, 65_536)];

fn supported_membership_profile(rows: usize, row_len: usize) -> bool {
    SUPPORTED_MEMBERSHIP_PROFILES.contains(&(rows, row_len))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PqDasMembershipBatchPrecompile<const BUS: bool>;

impl<const BUS: bool> TableT for PqDasMembershipBatchPrecompile<BUS> {
    fn name(&self) -> &'static str {
        "pq_das_membership_batch"
    }

    fn table(&self) -> Table {
        Table::pq_das_membership_batch()
    }

    fn n_columns_total(&self) -> usize {
        NUM_COLS_TOTAL_PQ_DAS_MEMBERSHIP_BATCH
    }

    fn bus_interactions(&self) -> Vec<BusInteraction> {
        let mut buses = vec![
            BusInteraction {
                direction: BusDirection::Pull,
                multiplicity: BusMultiplicity::Column(COL_PQ_MEM_EXEC_MULTIPLICITY),
                domainsep: BusData::Constant(PQ_DAS_MEMBERSHIP_DOMAINSEP),
                data: vec![
                    BusData::Column(COL_PQ_MEM_CODEWORD_BASE),
                    BusData::Column(COL_PQ_MEM_CHECK_VECTOR_PTR),
                    BusData::Column(COL_PQ_MEM_RESULT_BASE),
                ],
            },
            BusInteraction {
                direction: BusDirection::Push,
                multiplicity: BusMultiplicity::Column(COL_PQ_MEM_ACTIVE),
                domainsep: BusData::Column(COL_PQ_MEM_DOT_DOMAINSEP),
                data: vec![
                    BusData::Column(COL_PQ_MEM_IDX_A),
                    BusData::Column(COL_PQ_MEM_CHECK_VECTOR_PTR),
                    BusData::Column(COL_PQ_MEM_IDX_RES),
                ],
            },
        ];
        buses.extend(memory_lookups_consecutive(
            COL_PQ_MEM_IDX_RES,
            COL_PQ_MEM_ZERO_RESULT_START,
            DIMENSION,
        ));
        buses
    }

    fn padding_row(&self, zero_vec_ptr: usize, _null_hash_ptr: usize, _ending_pc: usize) -> Vec<F> {
        let mut row = vec![F::ZERO; NUM_COLS_TOTAL_PQ_DAS_MEMBERSHIP_BATCH];
        row[COL_PQ_MEM_CHECK_VECTOR_PTR] = F::from_usize(zero_vec_ptr);
        row[COL_PQ_MEM_RESULT_BASE] = F::from_usize(zero_vec_ptr);
        row[COL_PQ_MEM_IDX_RES] = F::from_usize(zero_vec_ptr);
        row
    }

    #[inline(always)]
    fn execute<M: MemoryAccess>(
        &self,
        codeword_base: F,
        check_vector_ptr: F,
        result_base: F,
        args: PrecompileCompTimeArgs<usize>,
        ctx: &mut InstructionContext<'_, M>,
    ) -> Result<(), RunnerError> {
        let PrecompileCompTimeArgs::PqDasMembershipBatch { rows, row_len } = args else {
            unreachable!("PqDasMembershipBatch table called with non-PQ-DAS args");
        };
        if !supported_membership_profile(rows, row_len) {
            return Err(RunnerError::InvalidExtensionOp);
        }

        let ext_mode = ExtensionOpMode {
            op: ExtensionOp::DotProduct,
            flag_be: false,
        };
        let row_stride = row_len * DIMENSION;
        let dot_domainsep = F::from_usize(extension_dot_product_domainsep(row_len));
        for row in 0..rows {
            let idx_a = codeword_base + F::from_usize(row * row_stride);
            let idx_b = check_vector_ptr;
            let idx_res = result_base + F::from_usize(row * DIMENSION);
            Table::extension_op().execute(
                idx_a,
                idx_b,
                idx_res,
                PrecompileCompTimeArgs::ExtensionOp {
                    size: row_len,
                    mode: ext_mode,
                },
                ctx,
            )?;

            if ctx.memory.get_ef_element(idx_res.to_usize())? != EF::ZERO {
                return Err(RunnerError::InvalidExtensionOp);
            }

            let trace = ctx.traces.get_mut(&self.table()).unwrap();
            trace.columns[COL_PQ_MEM_ACTIVE].push(F::ONE);
            trace.columns[COL_PQ_MEM_EXEC_MULTIPLICITY].push(F::from_bool(row == 0));
            trace.columns[COL_PQ_MEM_ROW].push(F::from_usize(row));
            trace.columns[COL_PQ_MEM_CODEWORD_BASE].push(codeword_base);
            trace.columns[COL_PQ_MEM_CHECK_VECTOR_PTR].push(check_vector_ptr);
            trace.columns[COL_PQ_MEM_RESULT_BASE].push(result_base);
            trace.columns[COL_PQ_MEM_ROW_STRIDE].push(F::from_usize(row_stride));
            trace.columns[COL_PQ_MEM_DOT_DOMAINSEP].push(dot_domainsep);
            trace.columns[COL_PQ_MEM_IDX_A].push(idx_a);
            trace.columns[COL_PQ_MEM_IDX_RES].push(idx_res);
            for k in 0..DIMENSION {
                trace.columns[COL_PQ_MEM_ZERO_RESULT_START + k].push(F::ZERO);
            }
        }
        Ok(())
    }
}

pub const fn extension_dot_product_domainsep(row_len: usize) -> usize {
    EXT_OP_FLAG_DOT_PRODUCT + EXT_OP_LEN_MULTIPLIER * row_len
}
