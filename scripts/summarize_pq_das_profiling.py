#!/usr/bin/env python3
import csv
import re
import sys
from pathlib import Path


def capture(text: str, pattern: str, default: str = "") -> str:
    match = re.search(pattern, text, re.MULTILINE)
    return match.group(1) if match else default


def parse_log(path: Path) -> dict[str, str]:
    text = path.read_text(encoding="utf-8")
    run_match = re.search(r"_run([0-9]+)\.log$", path.name)
    row: dict[str, str] = {
        "file": path.name,
        "run": run_match.group(1) if run_match else "",
        "profile": capture(text, r"^profile: ([^ ]+)"),
        "relation": capture(text, r"^proved relation: (.+)$"),
        "proof_accepted": capture(text, r"^proof accepted: (.+)$"),
        "proof_size_fe": capture(text, r"^proof size: ([0-9]+) field elements"),
        "encode_commit_s": capture(text, r"^encoding \+ commitment time: ([0-9.]+)s$"),
        "prover_preprocess_s": capture(text, r"^prover preprocessing time: ([0-9.]+)s$"),
        "leanvm_prove_s": capture(text, r"^LeanVM proving time: ([0-9.]+)s$"),
        "verifier_s": capture(
            text,
            r"^verifier statement rebuild \+ LeanVM proof verification time: ([0-9.]+)s$",
        ),
        "openings_s": capture(text, r"^opening verification time: ([0-9.]+)s$"),
        "vm_cycles": capture(text, r"^VM cycles: ([0-9]+)$"),
        "vm_memory": capture(text, r"^VM memory elements: ([0-9]+)$"),
        "vm_public_memory": capture(text, r"^VM public memory elements: ([0-9]+)$"),
        "vm_runtime_memory": capture(text, r"^VM runtime memory elements: ([0-9]+)$"),
        "poseidon_calls": capture(text, r"^VM Poseidon16 calls: ([0-9]+)$"),
        "extension_calls": capture(text, r"^VM extension-op calls: ([0-9]+)$"),
    }
    for table in ("execution", "extension_op", "poseidon16"):
        match = re.search(
            rf"^LeanVM table {table}: actual_rows=([0-9]+), padded_rows=([0-9]+)$",
            text,
            re.MULTILINE,
        )
        row[f"{table}_actual_rows"] = match.group(1) if match else ""
        row[f"{table}_padded_rows"] = match.group(2) if match else ""
    stages = {
        "bytecode_execution_s": "bytecode execution",
        "trace_generation_s": "trace generation",
        "prover_setup_s": "prover setup",
        "memory_access_count_s": "memory access count",
        "bytecode_access_count_s": "bytecode access count",
        "stack_and_commit_s": "stack and commit",
        "logup_s": "logup",
        "air_preparation_s": "AIR preparation",
        "air_sumcheck_s": "AIR sumcheck",
        "statement_finalization_s": "statement finalization",
        "whir_s": "WHIR excluding grinding",
        "grinding_s": "grinding",
    }
    for key, label in stages.items():
        row[key] = capture(text, rf"^LeanVM prover stage {re.escape(label)}: ([0-9.]+)s$")
    return row


def markdown_table(headers: list[str], rows: list[list[str]]) -> str:
    lines = [
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join("---" if i == 0 else "---:" for i in range(len(headers))) + " |",
    ]
    lines.extend("| " + " | ".join(row) + " |" for row in rows)
    return "\n".join(lines)


def main() -> None:
    output_dir = Path(sys.argv[1])
    rows = [parse_log(path) for path in sorted((output_dir / "logs").glob("*.log"))]
    if not rows:
        raise SystemExit(f"no logs found under {output_dir / 'logs'}")

    fieldnames = list(rows[0])
    with (output_dir / "results.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    full = [row for row in rows if row["relation"] == "all"]
    runtime_rows = [
        [
            row["profile"],
            row["run"],
            row["encode_commit_s"],
            row["prover_preprocess_s"],
            row["leanvm_prove_s"],
            row["verifier_s"],
            row["openings_s"],
        ]
        for row in full
    ]
    table_rows = [
        [
            row["profile"],
            row["run"],
            f'{row["execution_actual_rows"]}/{row["execution_padded_rows"]}',
            f'{row["extension_op_actual_rows"]}/{row["extension_op_padded_rows"]}',
            f'{row["poseidon16_actual_rows"]}/{row["poseidon16_padded_rows"]}',
        ]
        for row in full
    ]
    stage_rows = [
        [
            row["profile"],
            row["run"],
            row["bytecode_execution_s"],
            row["trace_generation_s"],
            row["prover_setup_s"],
            row["memory_access_count_s"],
            row["bytecode_access_count_s"],
            row["stack_and_commit_s"],
            row["logup_s"],
            row["air_preparation_s"],
            row["air_sumcheck_s"],
            row["statement_finalization_s"],
            row["whir_s"],
            row["grinding_s"],
        ]
        for row in full
    ]
    vm_rows = [
        [
            row["profile"],
            row["run"],
            row["vm_cycles"],
            row["vm_memory"],
            row["vm_public_memory"],
            row["vm_runtime_memory"],
            row["poseidon_calls"],
            row["extension_calls"],
        ]
        for row in full
    ]
    isolation_rows = [
        [
            row["profile"],
            row["run"],
            row["relation"],
            row["leanvm_prove_s"],
            row["vm_cycles"],
            row["vm_memory"],
            row["poseidon_calls"],
            row["extension_calls"],
            row["proof_size_fe"],
        ]
        for row in rows
    ]

    summary = [
        "# PQ-DAS LeanVM Profiling Results",
        "",
        "All times are in seconds. Table row counts are shown as actual/padded.",
        "",
        "## End-to-End Runtime",
        "",
        markdown_table(
            ["Profile", "Run", "Encode + commit", "Preprocess", "Prove", "Verifier", "Openings"],
            runtime_rows,
        ),
        "",
        "## LeanVM Table Rows",
        "",
        markdown_table(
            ["Profile", "Run", "Execution", "Extension op", "Poseidon16"],
            table_rows,
        ),
        "",
        "## VM Operation Statistics",
        "",
        markdown_table(
            ["Profile", "Run", "Cycles", "Memory", "Public memory", "Runtime memory", "Poseidon", "Ext ops"],
            vm_rows,
        ),
        "",
        "## Prover Stage Breakdown",
        "",
        markdown_table(
            [
                "Profile",
                "Run",
                "Execute",
                "Trace",
                "Setup",
                "Mem count",
                "Bytecode count",
                "Stack + commit",
                "Logup",
                "AIR prep",
                "AIR sumcheck",
                "Finalize",
                "WHIR",
                "Grinding",
            ],
            stage_rows,
        ),
        "",
        "## Relation Isolation",
        "",
        markdown_table(
            ["Profile", "Run", "Relation", "Prove", "VM cycles", "VM memory", "Poseidon", "Ext ops", "Proof FE"],
            isolation_rows,
        ),
        "",
    ]
    (output_dir / "SUMMARY.md").write_text("\n".join(summary), encoding="utf-8")


if __name__ == "__main__":
    main()
