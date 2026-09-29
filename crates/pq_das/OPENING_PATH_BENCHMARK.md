# All-Column Opening Paths Supplement

Run this lightweight supplement on the original PC, then separately on the server:

~~~bash
cargo run --release -p pq_das -- \
  --all-opening-paths-only \
  --path-benchmark-output pc-all-paths.json
~~~

Use a different output filename on the server. Existing files are not overwritten.
The command covers all column counts in Supplementary.md and Supplementary2.md:
256, 512, 1024, 2048, and 4096. It runs no LeanVM proving or erasure reconstruction.
The normal benchmark commands, throughput formula, and existing tables are unchanged.

- column-root: one independent outer path from each column root to the public column root.
- final-root: the same path followed by the row-root sibling, for the historical final-root-only commitment.
- Each packet contains a little-endian u32 column index followed by its siblings.
  Each sibling is eight canonical little-endian u32 field elements (32 bytes).
  Packet lengths follow from the profile; no codeword cell data is duplicated.
- All path size is the actual serialized buffer length: ell * (4 + 32 * h) bytes,
  where h = log2(ell) for column-root and h = log2(ell) + 1 for final-root.
  Raw JSON also separates index bytes from sibling bytes.
- Tree construction uses the existing Poseidon Merkle code with deterministic synthetic
  column digests, before timing. Digest values and row count do not affect path extraction.
- Timing includes fresh allocation, extraction of every path, and serialization.
  Cell copying, tree construction, verification, and buffer destruction are excluded.
  Each path is checked against its root outside timing.
- The default is 3 warmups and 100 measured assemblies per case. Only this small
  host operation is repeated. Set --path-benchmark-repetitions 1 for one measurement.
  Markdown shows the mean in milliseconds; JSON preserves every nanosecond measurement,
  median/min/max, exact bytes, CPU model, and Git revision.
- One assembly thread is used. Reuse a measurement only for the same machine,
  column count, and path format; it is independent of WHIR rate or proving variant.

For later table corrections, keep the old sampled-response time and all historical
measurements. The additional upload time is all_path_bytes / 6_250_000 seconds
at 50 Mbps. Add that and the measured all-path assembly time to the old total latency.
Do not count tree construction again, or replace the sampled multiproof with a
multiproof over all leaves: that would not provide independently authenticated packets.

Match the historical opening format before updating a row. Older PC rows used
paths to the final root; current server and precompile rows expose row hashes and
the column root. No table values are updated by this command.
