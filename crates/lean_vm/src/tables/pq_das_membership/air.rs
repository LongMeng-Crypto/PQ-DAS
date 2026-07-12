use crate::{
    EF, ExtraDataForBuses, PQ_DAS_MEMBERSHIP_BASELINE_ROW_LEN, PQ_DAS_MEMBERSHIP_CODEWORD_ROW_STRIDE,
    PqDasMembershipBatchPrecompile, eval_bus_virtual, tables::pq_das_membership::extension_dot_product_domainsep,
};
use backend::*;

pub const COL_PQ_MEM_ACTIVE: usize = 0;
pub const COL_PQ_MEM_EXEC_MULTIPLICITY: usize = 1;
pub const COL_PQ_MEM_ROW: usize = 2;
pub const COL_PQ_MEM_CODEWORD_BASE: usize = 3;
pub const COL_PQ_MEM_CHECK_VECTOR_PTR: usize = 4;
pub const COL_PQ_MEM_RESULT_BASE: usize = 5;
pub const COL_PQ_MEM_IDX_A: usize = 6;
pub const COL_PQ_MEM_IDX_RES: usize = 7;
pub const COL_PQ_MEM_ZERO_RESULT_START: usize = 8;
pub const NUM_COLS_TOTAL_PQ_DAS_MEMBERSHIP_BATCH: usize = COL_PQ_MEM_ZERO_RESULT_START + crate::DIMENSION;

impl<const BUS: bool> Air for PqDasMembershipBatchPrecompile<BUS> {
    type ExtraData = ExtraDataForBuses<EF>;

    fn n_columns(&self) -> usize {
        NUM_COLS_TOTAL_PQ_DAS_MEMBERSHIP_BATCH
    }

    fn degree_air(&self) -> usize {
        3
    }

    fn n_constraints(&self) -> usize {
        19
    }

    fn n_shift_columns(&self) -> usize {
        COL_PQ_MEM_RESULT_BASE + 1
    }

    fn eval<AB: AirBuilder>(&self, builder: &mut AB, extra_data: &Self::ExtraData) {
        let (active, exec_mult, row, codeword_base, check_ptr, result_base, idx_a, idx_res, zero_result) = {
            let flat = builder.flat();
            (
                flat[COL_PQ_MEM_ACTIVE],
                flat[COL_PQ_MEM_EXEC_MULTIPLICITY],
                flat[COL_PQ_MEM_ROW],
                flat[COL_PQ_MEM_CODEWORD_BASE],
                flat[COL_PQ_MEM_CHECK_VECTOR_PTR],
                flat[COL_PQ_MEM_RESULT_BASE],
                flat[COL_PQ_MEM_IDX_A],
                flat[COL_PQ_MEM_IDX_RES],
                std::array::from_fn::<_, { crate::DIMENSION }, _>(|k| flat[COL_PQ_MEM_ZERO_RESULT_START + k]),
            )
        };
        let (active_shift, row_shift, codeword_base_shift, check_ptr_shift, result_base_shift) = {
            let shift = builder.shift();
            (
                shift[COL_PQ_MEM_ACTIVE],
                shift[COL_PQ_MEM_ROW],
                shift[COL_PQ_MEM_CODEWORD_BASE],
                shift[COL_PQ_MEM_CHECK_VECTOR_PTR],
                shift[COL_PQ_MEM_RESULT_BASE],
            )
        };

        if BUS {
            eval_bus_virtual::<AB, EF>(
                builder,
                extra_data,
                exec_mult,
                AB::IF::from_usize(crate::PQ_DAS_MEMBERSHIP_DOMAINSEP),
                &[codeword_base, check_ptr, result_base],
            );
            eval_bus_virtual::<AB, EF>(
                builder,
                extra_data,
                active,
                AB::IF::from_usize(extension_dot_product_domainsep(PQ_DAS_MEMBERSHIP_BASELINE_ROW_LEN)),
                &[idx_a, check_ptr, idx_res],
            );
        } else {
            builder.declare_values(&[active, exec_mult]);
            builder.declare_values(&[codeword_base, check_ptr, result_base, idx_a, idx_res]);
        }

        builder.assert_bool(active);
        builder.assert_bool(exec_mult);
        builder.assert_zero(exec_mult * row);
        builder.assert_zero((AB::IF::ONE - active) * exec_mult);

        builder.assert_zero(
            active * (idx_a - codeword_base - row * AB::F::from_usize(PQ_DAS_MEMBERSHIP_CODEWORD_ROW_STRIDE)),
        );
        builder.assert_zero(active * (idx_res - result_base - row * AB::F::from_usize(crate::DIMENSION)));

        for value in zero_result {
            builder.assert_zero(value);
        }

        // Active rows are generated as one contiguous fixed-profile block. These transition
        // constraints bind the row schedule for all non-padding successors.
        builder.assert_zero(active * active_shift * (row_shift - row - AB::F::ONE));
        builder.assert_zero(active * active_shift * (codeword_base_shift - codeword_base));
        builder.assert_zero(active * active_shift * (check_ptr_shift - check_ptr));
        builder.assert_zero(active * active_shift * (result_base_shift - result_base));
    }
}
