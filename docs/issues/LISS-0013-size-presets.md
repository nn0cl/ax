# LISS-0013: Environment size presets (`c2` / `fat32` / `flash` / `max`)

## Metadata

- Local issue ID: LISS-0013
- GitHub issue: n/a (tracked with product docs PR)
- Status: done
- Phase: complete (retro + ADR 0021)
- Type: feature
- Priority: P2
- Initial planning size: S
- Current planning size: S
- Owner/agent: grok-build
- Related branch: implemented on main / LISS-0011 branch docs
- ADR: 0021

## Summary

Allow `-s` / `target_size` to accept named environment presets in addition to
numeric sizes, so operators can pick common media/transfer profiles quickly.

## User story

As an operator,  
I want named pack budgets (`c2`, `fat32`, `flash`, `max`),  
so that I do not re-derive byte sizes for common USB and network scenarios.

## Acceptance Notes

### Must have

1. Presets resolve to domain budgets (see ADR 0021 table).
2. Numeric sizes (`10M`, `4G`) still work.
3. Aliases documented (`low-net` → `c2`, etc.).
4. Unit tests for preset parsing; integration dry-run/pack accepts `-s flash` / `-s c2`.
5. Help / README list presets.

### Explicit non-goals

- Compressed-size packing (ADR 0013).
- Auto-detecting filesystem type to pick fat32.

## Dependencies

- Related: LISS-0012 (CLI surface), ADR 0013, ADR 0021

## Verification

- [x] Domain tests `size_presets`
- [x] `pack_size_preset_*` integration (where present)
- [x] ADR 0021 written
