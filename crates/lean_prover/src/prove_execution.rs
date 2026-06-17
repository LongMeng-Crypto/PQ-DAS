use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use crate::*;
use backend::ArenaVec;
use backend::ansi::Colorize;
use lean_vm::*;
use serde::{Deserialize, Serialize};
use sub_protocols::*;
use tracing::info_span;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProof {
    pub proof: Proof<F>,
    // benchmark / debug purpose
    #[serde(skip, default)]
    pub metadata: Option<ExecutionMetadata>,
    #[serde(skip, default)]
    pub prover_profile: Option<ProverProfile>,
}

#[derive(Debug, Clone)]
pub struct TableProfile {
    pub name: &'static str,
    pub actual_rows: usize,
    pub padded_rows: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ProverStageTimings {
    pub bytecode_execution: Duration,
    pub trace_generation: Duration,
    pub prover_setup: Duration,
    pub memory_access_count: Duration,
    pub bytecode_access_count: Duration,
    pub stack_and_commit: Duration,
    pub logup: Duration,
    pub air_preparation: Duration,
    pub air_sumcheck: Duration,
    pub statement_finalization: Duration,
    pub whir: Duration,
    pub grinding: Duration,
}

#[derive(Debug, Clone)]
pub struct ProverProfile {
    pub tables: Vec<TableProfile>,
    pub timings: ProverStageTimings,
}

pub fn prove_execution(
    bytecode: &Bytecode,
    public_input: &[F; PUBLIC_INPUT_LEN],
    witness: &ExecutionWitness,
    whir_config: &WhirConfigBuilder,
    vm_profiler: bool,
) -> Result<ExecutionProof, ProverError> {
    check_rate(whir_config.starting_log_inv_rate).map_err(|_| ProverError::InvalidRate)?;
    reset_pow_grinding_time();
    let mut timings = ProverStageTimings::default();
    let ExecutionTrace {
        traces,
        mut memory, // padded with zeros to next power of two
        metadata,
    } = info_span!("Witness generation").in_scope(|| -> Result<_, ProverError> {
        let started = Instant::now();
        let execution_result = info_span!("Executing bytecode")
            .in_scope(|| try_execute_bytecode(bytecode, public_input, witness, vm_profiler))?;
        timings.bytecode_execution = started.elapsed();

        let started = Instant::now();
        let trace = info_span!("Building execution trace")
            .in_scope(|| get_execution_trace(bytecode, execution_result, &witness.min_table_log_n_rows));
        timings.trace_generation = started.elapsed();
        Ok(trace)
    })?;
    let table_profiles = traces
        .iter()
        .map(|(table, trace)| TableProfile {
            name: table.name(),
            actual_rows: trace.non_padded_n_rows,
            padded_rows: 1 << trace.log_n_rows,
        })
        .collect();
    let started = Instant::now();

    // Memory must be at least MIN_LOG_MEMORY_SIZE and at least bytecode size
    // (required by the stacked polynomial ordering)
    let min_memory_size = (1 << MIN_LOG_MEMORY_SIZE).max(1 << bytecode.log_size());
    if memory.len() < min_memory_size {
        memory.resize(min_memory_size, F::ZERO);
    }
    let mut prover_state = ProverState::new(get_poseidon16().clone(), fiat_shamir_domain_sep(bytecode));
    prover_state.observe_scalars(public_input);
    prover_state.add_base_scalars(
        &[
            vec![whir_config.starting_log_inv_rate, log2_strict_usize(memory.len())],
            traces.values().map(|t| t.log_n_rows).collect::<Vec<_>>(),
        ]
        .concat()
        .into_iter()
        .map(F::from_usize)
        .collect::<Vec<_>>(),
    );
    for (table, table_trace) in &traces {
        let log_n_rows = table_trace.log_n_rows;
        assert!(log_n_rows >= MIN_LOG_N_ROWS_PER_TABLE, "missing padding");
        let log_limit = max_log_n_rows_per_table(table);
        if log_n_rows > log_limit {
            return Err(TooBigTableError {
                table_name: table.name(),
                log_n_rows,
                log_limit,
            }
            .into());
        }
    }

    let mut table_log = String::new();
    for (table, trace) in &traces {
        table_log.push_str(&format!(
            "{}: 2^{} * (1 + {:.2}) rows | ",
            table.name(),
            trace.log_n_rows - 1,
            (trace.non_padded_n_rows as f64) / (1 << (trace.log_n_rows - 1)) as f64 - 1.0
        ));
    }
    table_log = table_log.trim_end_matches(" | ").to_string();
    tracing::info!("Trace tables sizes: {}", table_log.magenta());
    timings.prover_setup = started.elapsed();

    let started = Instant::now();
    let memory_acc = info_span!("Building memory access count").in_scope(|| -> Result<ArenaVec<F>, ProverError> {
        let counters: Vec<_> = (0..memory.len()).map(|_| AtomicUsize::new(0)).collect();
        for (table, trace) in &traces {
            let buses = table.bus_interactions();
            for group in memory_lookup_groups(&buses) {
                let idx_col = &trace.columns[group.idx_col];
                let n = group.value_cols.len();
                parallel::for_each_index(idx_col.len(), |i| {
                    let base = idx_col[i].to_usize();
                    assert!(base + n <= counters.len(), "memory lookup out of bounds");
                    for offset in 0..n {
                        counters[base + offset].fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
        }
        Ok(ArenaVec::par_collect(memory.len(), |i| {
            F::from_usize(counters[i].load(Ordering::Relaxed))
        }))
    })?;
    timings.memory_access_count = started.elapsed();

    let started = Instant::now();
    let bytecode_acc =
        info_span!("Building bytecode access count").in_scope(|| -> Result<ArenaVec<F>, ProverError> {
            let pc_col = &traces[&Table::execution()].columns[EXEC_COL_PC];
            let counters: Vec<_> = (0..bytecode.padded_size()).map(|_| AtomicUsize::new(0)).collect();
            parallel::for_each_index(pc_col.len(), |i| {
                let pc = pc_col[i].to_usize();
                assert!(pc < counters.len(), "bytecode PC out of bounds");
                counters[pc].fetch_add(1, Ordering::Relaxed);
            });
            Ok(ArenaVec::par_collect(bytecode.padded_size(), |i| {
                F::from_usize(counters[i].load(Ordering::Relaxed))
            }))
        })?;
    timings.bytecode_access_count = started.elapsed();

    // 1st Commitment
    let started = Instant::now();
    let stacked_pcs_witness = stack_polynomials_and_commit(
        &mut prover_state,
        whir_config,
        &memory,
        &memory_acc,
        &bytecode_acc,
        &traces,
    );
    timings.stack_and_commit = started.elapsed();

    // logup (GKR)
    let started = Instant::now();
    let logup_c = prover_state.sample();
    prover_state.duplex();
    let logup_alphas = prover_state.sample_vec(LOG_MAX_BUS_WIDTH);
    let logup_alphas_eq_poly = eval_eq(&logup_alphas);

    let logup_statements = prove_generic_logup(
        &mut prover_state,
        logup_c,
        &logup_alphas_eq_poly,
        &memory,
        &memory_acc,
        bytecode.instructions_multilinear(),
        &bytecode_acc,
        &traces,
    );
    timings.logup = started.elapsed();
    let started = Instant::now();
    let gkr_point = &logup_statements.gkr_point;
    let mut committed_statements: CommittedStatements = Default::default();
    for table in ALL_TABLES {
        let log_n_rows = traces[&table].log_n_rows;
        committed_statements.insert(
            table,
            vec![(
                MultilinearPoint(from_end(gkr_point, log_n_rows).to_vec()),
                logup_statements.columns_values[&table].clone(),
                BTreeMap::new(),
            )],
        );
    }

    let air_alpha = prover_state.sample();
    let air_alpha_powers: Vec<EF> = air_alpha.powers().collect_n(total_air_constraints());

    let tables_log_heights: BTreeMap<Table, VarCount> =
        traces.iter().map(|(table, trace)| (*table, trace.log_n_rows)).collect();

    let column_refs: Vec<Vec<&[F]>> = ALL_TABLES
        .iter()
        .map(|table| {
            traces[table].columns[..table.n_columns()]
                .iter()
                .map(|c| c.as_slice())
                .collect()
        })
        .collect();
    let _span = info_span!("Computing shifted columns for AIR sumcheck").entered();
    let shifted_rows: Vec<Vec<ArenaVec<F>>> = ALL_TABLES
        .iter()
        .zip(&column_refs)
        .map(|(table, cols)| compute_shifted_columns(table.n_shift_columns(), cols))
        .collect();
    std::mem::drop(_span);
    let mut sessions = Vec::with_capacity(ALL_TABLES.len());
    let mut alpha_offset = 0;
    for (idx, table) in ALL_TABLES.iter().enumerate() {
        let log_n_rows = tables_log_heights[table];
        let n_constraints = table.n_constraints();
        let bus_numerator_value = logup_statements.bus_numerators_values[table];
        let bus_denominator_value = logup_statements.bus_denominators_values[table];
        let signed_numerator = bus_numerator_value
            * match table.bus_interactions()[0].direction {
                BusDirection::Pull => EF::NEG_ONE,
                BusDirection::Push => EF::ONE,
            };
        // Each table consumes a disjoint range of alpha powers; alpha^offset weights the bus
        // numerator (multiplicity), alpha^{offset+1} weights the bus fingerprint, alpha^{offset+2..}
        // weight the remaining AIR constraints.
        let bus_final_value = air_alpha_powers[alpha_offset] * signed_numerator
            + air_alpha_powers[alpha_offset + 1] * (logup_c - bus_denominator_value);

        let eq_suffix = from_end(gkr_point, log_n_rows).to_vec();

        let alpha_slice = air_alpha_powers[alpha_offset..alpha_offset + n_constraints].to_vec();
        let extra_data = ExtraDataForBuses::new(&logup_alphas_eq_poly, alpha_slice);

        let mut flat_and_shift: Vec<&[PF<EF>]> = column_refs[idx].to_vec();
        flat_and_shift.extend(shifted_rows[idx].iter().map(|c| c.as_slice()));
        let packed = MleGroupRef::<EF>::Base(flat_and_shift).pack();

        let non_padded = traces[table].non_padded_n_rows;

        macro_rules! make_session {
            ($t:expr) => {{
                let session = AirSumcheckSession::new(packed, eq_suffix, bus_final_value, *$t, extra_data, non_padded);
                Box::new(session) as Box<dyn OuterSumcheckSession<EF> + '_>
            }};
        }
        sessions.push(delegate_to_inner!(table => make_session));
        alpha_offset += n_constraints;
    }
    timings.air_preparation = started.elapsed();

    let started = Instant::now();
    let sumcheck_air_point =
        info_span!("batched AIR sumcheck").in_scope(|| prove_batched_air_sumcheck(&mut prover_state, &mut sessions));
    timings.air_sumcheck = started.elapsed();

    let started = Instant::now();
    let final_column_evals: Vec<Vec<EF>> = sessions.iter().map(|session| session.final_column_evals()).collect();
    for col_evals in &final_column_evals {
        prover_state.add_extension_scalars(&col_evals);
    }

    let claims = parallel::par_map_collect(ALL_TABLES.len(), |idx| {
        let table = ALL_TABLES[idx];
        let col_evals = &final_column_evals[idx];
        let natural_ordering_point =
            natural_ordering_point_for_session(&sumcheck_air_point.0, traces[&table].log_n_rows);
        macro_rules! split {
            ($t:expr) => {{ columns_evals_flat_and_shift($t, &col_evals, &natural_ordering_point) }};
        }
        delegate_to_inner!(&table => split)
    });
    for (table, claim) in ALL_TABLES.iter().zip(claims) {
        committed_statements.get_mut(table).unwrap().push(claim);
    }

    let public_memory_len = bytecode.public_memory_len();
    let public_memory_random_point = MultilinearPoint(prover_state.sample_vec(log2_strict_usize(public_memory_len)));
    let public_memory_eval = (&memory[..public_memory_len]).evaluate(&public_memory_random_point);

    let previous_statements = vec![
        SparseStatement::new(
            stacked_pcs_witness.stacked_n_vars,
            logup_statements.memory_and_acc_point,
            vec![
                SparseValue::new(0, logup_statements.value_memory),
                SparseValue::new(1, logup_statements.value_memory_acc),
            ],
        ),
        SparseStatement::new(
            stacked_pcs_witness.stacked_n_vars,
            public_memory_random_point,
            vec![SparseValue::new(0, public_memory_eval)],
        ),
        SparseStatement::new(
            stacked_pcs_witness.stacked_n_vars,
            logup_statements.bytecode_and_acc_point,
            vec![SparseValue::new(
                (2 * memory.len()) >> bytecode.log_size(),
                logup_statements.value_bytecode_acc,
            )],
        ),
    ];

    let global_statements_base = stacked_pcs_global_statements(
        stacked_pcs_witness.stacked_n_vars,
        log2_strict_usize(memory.len()),
        bytecode.log_size(),
        bytecode.ending_pc(),
        previous_statements,
        &tables_log_heights,
        &committed_statements,
    );
    timings.statement_finalization = started.elapsed();

    let started = Instant::now();
    WhirConfig::new(whir_config, stacked_pcs_witness.global_polynomial.by_ref().n_vars()).prove(
        &mut prover_state,
        global_statements_base,
        stacked_pcs_witness.inner_witness,
        &stacked_pcs_witness.global_polynomial.by_ref(),
    );
    let whir_total = started.elapsed();
    timings.grinding = pow_grinding_time();
    timings.whir = whir_total.saturating_sub(timings.grinding);

    tracing::info!("total pow_grinding time: {} ms", timings.grinding.as_millis());
    reset_pow_grinding_time();

    Ok(ExecutionProof {
        proof: prover_state.into_proof(),
        metadata: Some(metadata),
        prover_profile: Some(ProverProfile {
            tables: table_profiles,
            timings,
        }),
    })
}
