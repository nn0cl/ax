# LISS-0014: Pack log quiet default and `-v` / `-vv`

## Metadata

- Local issue ID: LISS-0014
- GitHub issue: n/a
- Status: done
- Phase: complete (retro + ADR 0023)
- Type: feature / ux
- Priority: P1
- Initial planning size: S
- Current planning size: S
- Owner/agent: grok-build
- Related branch: feature/liss-0011-progress-dashboard (logging changes)
- ADR: 0023
- Related: LISS-0011 (dashboard separate from text logs)

## Summary

Stop flooding the terminal with per-unit plan/transfer lines. Default text
output is summary-only; detailed dumps require `-vv`. Live dashboard is
independent (ADR 0022).

## User story

As an operator packing a large tree,  
I want quiet logs by default and optional detail with `-vv`,  
so that the terminal remains usable while the dashboard still shows workers.

## Acceptance Notes

### Must have

1. Default: no unbounded per-unit `plan` lines in stdout/stderr dumps.
2. `-vv`: capped plan/transfer detail + full size_estimate class breakdown.
3. Resource messages: compact + Warn/Fail only.
4. Dashboard not disabled when `-v` is omitted.
5. Tests updated for quiet dry-run output (class breakdown only with `-vv`).

### Explicit non-goals

- Structured JSON logging.
- Log files on disk by default.

## Dependencies

- ADR 0022, ADR 0023
- LISS-0011

## Verification

- [x] Quiet dry-run tests
- [x] ADR 0023
