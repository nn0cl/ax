# ADR 0013: Compressed-size packing mode

- Status: accepted (defer implementation)
- Date: 2026-07-17
- Issue: LISS-0010
- Deciders: Adjudicator (blanket implement approval for WP-0002; decision recorded here)

## Context

Pack unit budgets currently use **logical (uncompressed) file sizes**. This is
deterministic and cheap, but:

- highly compressible data yields archives much smaller than the budget;
- already-compressed data may produce archives larger than a naive “media fill”
  expectation;
- USB transfer time depends on **bytes written**, not logical source size.

Product discussion also noted that archives can make *copy* faster mainly by
reducing total bytes and collapsing many small files—not by raising bus speed.

## Options

| Option | Description | Accuracy | Cost | Complexity |
| --- | --- | --- | --- | --- |
| **A. Logical only (status quo)** | Budget = sum of file sizes | Low for media fill | Lowest | Lowest |
| **B. Safety factor** | Budget = logical × ratio (e.g. 0.7) | Rough | Low | Low |
| **C. Multi-pass measure** | Temp-compress, re-bin units | High | High CPU/disk | High |
| **D. Stream and cut** | Write archive until size threshold | High for archive size | Medium | High (format-dependent) |

## Decision

**Adopt A for production packing; optionally document B as a future CLI knob;
defer C/D.**

Rationale:

1. Primary product goal for unit sizing is **transfer-friendly chunking**
   (`throughput × window`, free-space cap)—already covered by `estimate` /
   `--auto-size`, which target **bytes on the wire after archives exist**.
2. C/D optimize for strict media-capacity packing, which is a different goal
   and conflicts with “fast path / simple restore mapping” until proven needed.
3. Users who need stricter fill can lower `--target-size` / `--target-secs`
   manually today without multi-pass complexity.

## Consequences

- `size_basis` remains `"logical"` in mapping v2.
- No implementation issue for C/D until real “media overflow” reports appear.
- If revisited: prefer **B** first (config ratio), then D for zip/tar writers
  that can report compressed bytes mid-stream.

## Related (not a packing change)

- **ADR 0018** adds content-class × algorithm **estimates** for progress ETA and
  `size_estimate:` logs. That does **not** change unit budgeting (still logical).
- Free-space hard checks remain logical × safety (worst case).

## Follow-up

- Do not file implementation LISS for C/D until this ADR is reopened.
- Option B (`budget *= safety`) may be added as an optional config knob without
  reopening C/D.
- Related: transfer estimate (LISS-0006/0007) remains the right lever for
  USB-oriented unit sizing.
