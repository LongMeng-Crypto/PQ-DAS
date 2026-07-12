use crate::{EF, ExtraDataForBuses, PqDasCommitmentPrecompile, eval_bus_virtual};
use backend::*;

pub const COL_PQ_COM_ACTIVE: usize = 0;
pub const COL_PQ_COM_EXEC_MULTIPLICITY: usize = 1;
pub const COL_PQ_COM_STEP: usize = 2;
pub const COL_PQ_COM_CODEWORD_BASE: usize = 3;
pub const COL_PQ_COM_PUBLIC_ROOT_PTR: usize = 4;
pub const COL_PQ_COM_SCRATCH_BASE: usize = 5;
pub const COL_PQ_COM_IDX_A: usize = 6;
pub const COL_PQ_COM_IDX_B: usize = 7;
pub const COL_PQ_COM_IDX_RES: usize = 8;
pub const COL_PQ_COM_POSEIDON_DOMAINSEP: usize = 9;
pub const NUM_COLS_TOTAL_PQ_DAS_COMMITMENT: usize = 10;

impl<const BUS: bool> Air for PqDasCommitmentPrecompile<BUS> {
    type ExtraData = ExtraDataForBuses<EF>;
    fn n_columns(&self) -> usize {
        NUM_COLS_TOTAL_PQ_DAS_COMMITMENT
    }
    fn degree_air(&self) -> usize {
        3
    }
    fn n_constraints(&self) -> usize {
        13
    }
    fn n_shift_columns(&self) -> usize {
        COL_PQ_COM_POSEIDON_DOMAINSEP + 1
    }

    fn eval<AB: AirBuilder>(&self, builder: &mut AB, extra_data: &Self::ExtraData) {
        let (
            active,
            exec_mult,
            step,
            codeword_base,
            public_root,
            scratch_base,
            idx_a,
            idx_b,
            idx_res,
            poseidon_domainsep,
        ) = {
            let flat = builder.flat();
            (
                flat[COL_PQ_COM_ACTIVE],
                flat[COL_PQ_COM_EXEC_MULTIPLICITY],
                flat[COL_PQ_COM_STEP],
                flat[COL_PQ_COM_CODEWORD_BASE],
                flat[COL_PQ_COM_PUBLIC_ROOT_PTR],
                flat[COL_PQ_COM_SCRATCH_BASE],
                flat[COL_PQ_COM_IDX_A],
                flat[COL_PQ_COM_IDX_B],
                flat[COL_PQ_COM_IDX_RES],
                flat[COL_PQ_COM_POSEIDON_DOMAINSEP],
            )
        };
        let (active_shift, step_shift, codeword_base_shift, public_root_shift, scratch_base_shift) = {
            let shift = builder.shift();
            (
                shift[COL_PQ_COM_ACTIVE],
                shift[COL_PQ_COM_STEP],
                shift[COL_PQ_COM_CODEWORD_BASE],
                shift[COL_PQ_COM_PUBLIC_ROOT_PTR],
                shift[COL_PQ_COM_SCRATCH_BASE],
            )
        };
        if BUS {
            eval_bus_virtual::<AB, EF>(
                builder,
                extra_data,
                exec_mult,
                AB::IF::from_usize(crate::PQ_DAS_COMMITMENT_DOMAINSEP),
                &[codeword_base, public_root, scratch_base],
            );
            eval_bus_virtual::<AB, EF>(
                builder,
                extra_data,
                active,
                poseidon_domainsep,
                &[idx_a, idx_b, idx_res],
            );
        } else {
            builder.declare_values(&[active, exec_mult]);
            builder.declare_values(&[
                codeword_base,
                public_root,
                scratch_base,
                idx_a,
                idx_b,
                idx_res,
                poseidon_domainsep,
            ]);
        }
        builder.assert_bool(active);
        builder.assert_bool(exec_mult);
        builder.assert_zero(exec_mult * step);
        builder.assert_zero((AB::IF::ONE - active) * exec_mult);
        builder
            .assert_zero(active * (poseidon_domainsep - AB::F::from_usize(crate::poseidon_compress_half_domainsep())));
        builder.assert_zero(active * active_shift * (step_shift - step - AB::F::ONE));
        builder.assert_zero(active * active_shift * (codeword_base_shift - codeword_base));
        builder.assert_zero(active * active_shift * (public_root_shift - public_root));
        builder.assert_zero(active * active_shift * (scratch_base_shift - scratch_base));
    }
}
