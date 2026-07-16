# Stack-Specific Examples

These files are not part of the generic operating contract. They are
placeholder examples that show the *pattern* for filling in
`docs/architecture/project-structure.md`,
`docs/architecture/dependency-policy.md`, per-directory `AGENTS.md` files, and
adoption prompts.

Copy the pattern, not the content:

- `rust-agent-instructions.md` / `frontend-agent-instructions.md` show how to
  scope a directory-local `AGENTS.md` once a subproject exists.
- `deny.toml` / `dependency-cruiser.config.cjs` show how to wire a dependency
  policy tool into `docs/architecture/dependency-policy.md` and CI.
- `adoption-prompts.md` shows how to prompt an AI agent after copying the
  template into a target repository.

Delete this folder, or replace its contents with your own stack's examples,
once your project's real architecture documents exist.
