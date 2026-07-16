# Feature: Archive CLI (MVP)

Local-first command-line utility that creates, lists, and extracts file
archives in ZIP, TAR, and TAR.GZ formats.

## EARS

When the user requests archive creation with one or more existing input paths
and an output path, the system shall write an archive containing those files
and the relative tree under any input directories.

When the user omits `--format` for create, the system shall infer the format
from the output path extension (`.zip`, `.tar`, `.tar.gz` / `.tgz`).

When the user requests listing an existing archive, the system shall print each
member path contained in the archive.

When the user requests extract into a destination directory, the system shall
write all members under that directory and create intermediate directories as
needed.

If an archive member path would resolve outside the destination directory
(path traversal), the system shall refuse the extract and report an error
without writing that member.

If the format cannot be determined or is unsupported, the system shall report
an error and leave the filesystem unchanged for create.

If an input path for create does not exist, the system shall report an error.

## Gherkin

```gherkin
Scenario: Create a ZIP archive from a file and a directory
  Given a temporary workspace with file "a.txt" and directory "dir/b.txt"
  When the user runs create with output "out.zip" and those inputs
  Then the archive "out.zip" exists
  And listing "out.zip" includes "a.txt" and a path for "dir/b.txt"

Scenario: List members of an archive
  Given an existing archive with members "a.txt" and "dir/b.txt"
  When the user runs list on that archive
  Then the output includes "a.txt"
  And the output includes "dir/b.txt"

Scenario: Extract archive into a destination directory
  Given an existing archive with members "a.txt" and "dir/b.txt"
  And an empty destination directory "dest"
  When the user runs extract with destination "dest"
  Then "dest/a.txt" exists with the original contents
  And "dest/dir/b.txt" exists with the original contents

Scenario: Reject path traversal on extract
  Given an archive that contains a member named "../escape.txt"
  When the user runs extract into "dest"
  Then the command fails
  And "escape.txt" is not written outside "dest"

Scenario: Infer format from extension
  Given input file "a.txt"
  When the user runs create with output "bundle.tar.gz" without --format
  Then the archive is readable as TAR.GZ
  And listing includes "a.txt"
```

## External Dependencies

- Local filesystem via adapter (read inputs, write archive / extract tree).
- Archive engines (ZIP, TAR, TAR.GZ) via `ArchiveEngine` port.

## Out of Scope

- 7z, RAR, XZ, BZIP2, encrypted archives.
- Password-protected ZIP.
- Streaming from stdin/stdout.
- Progress bars / interactive UI.
- Multi-volume archives.
- Preserve of ownership / full Unix permissions beyond what crates provide by default.

## Ambiguities (resolved for MVP)

- **Relative paths in archives**: member names are relative to each input's
  parent for files, or to the directory itself for directory roots (so
  `archiver create -o out.zip ./src` stores paths under `src/...` using the
  last path component as root name).
- **Overwrite**: create overwrites an existing output file; extract overwrites
  existing files at the same destination path.
