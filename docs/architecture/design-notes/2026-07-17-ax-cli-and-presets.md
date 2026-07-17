# Design note: `ax` CLI surface and size presets

## Path

Feature Path (retro documentation). Implementation already landed; this note
records intent for future changes.

## Scope

- In: binary name, short commands, globals, size presets, help/install.
- Out: progress dashboard (see LISS-0011 design note), pack algorithm.

## Decisions (normative: ADR 0020, 0021)

### Binary

- Default artifact: **`ax`**.
- Package name remains `archiver` (Cargo).
- Compat: `compat-bin` feature or symlink.

### Commands

Short names are primary in help order; long aliases remain for scripts.

### Globals

| Flag | Role |
|------|------|
| `-c` | config file |
| `-s` | budget size **or** preset |
| `-o` | output / archive / mapping |

Pack source: **`-i`**.

### Presets

See ADR 0021. Domain: `SizePreset` + `parse_size`.

## Data flow

```text
CLI (clap) → resolve_pack_settings (config merge)
           → parse_size_with_preset
           → effective_budget
           → PackRequest.budget
```

## Tests

- `delivery::cli` / domain media preset tests
- Integration: short `pa -i -o -s`, preset dry-run

## Open follow-ups

- None required for acceptance; Windows binary naming out of scope (ADR 0019).
