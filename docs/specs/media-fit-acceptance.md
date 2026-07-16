# Media-fit acceptance matrix

Freeze date: 2026-07-17  
Spec: `docs/specs/media-fit-pack.md`  
Issue: LISS-0002

| # | Behavior | Automated test | Status |
|---|----------|----------------|--------|
| A1 | 1 folder fits → 1 archive under that folder | `pack_restore::pack_one_folder_one_archive_when_fits` | covered |
| A2 | Oversized folder splits by extension | `pack_restore::pack_splits_by_extension_when_folder_too_large` + `pack_plan::oversized_folder_splits_by_extension` | covered |
| A3 | Huge single file alone, no byte split | `pack_restore::pack_accepts_huge_single_file` + `pack_plan::huge_file_alone_even_over_budget` | covered |
| A4 | Empty dirs in mapping + restore | `pack_restore::pack_records_and_restores_empty_directories` | covered |
| A5 | Round-trip bytes preserved | A1 / A3 / tar.zst tests | covered |
| A6 | tar.zst pack/restore | `pack_restore::pack_tar_zst_round_trip` | covered |
| A7 | pack requires budget | `pack_restore::pack_requires_budget` | covered |
| A8 | estimate prints recommended_unit | `pack_restore::estimate_reports_recommended_unit` | covered |
| A9 | pack --auto-size works | `pack_restore::pack_auto_size_uses_estimate` | covered |
| A10 | USB2 unit < USB3 unit | `transfer_estimate::usb2_unit_smaller_than_usb3` | covered |
| A11 | free space caps unit | `transfer_estimate::free_space_caps_unit` | covered |
| A12 | path traversal rejected (low-level) | `archive_cli::rejects_path_traversal_on_zip_extract` | covered |
| A13 | member SHA-256 recorded + verify | `pack_restore::restore_verify_detects_tamper` | required (LISS-0005) |
| A14 | dry-run writes no archives | `pack_restore::pack_dry_run_writes_nothing` | required (LISS-0008) |
| A15 | --jobs>1 mapping multiset equals jobs=1 | `pack_restore::pack_jobs_deterministic_members` | required (LISS-0009) |

Low-level CLI matrix: see `tests/archive_cli.rs` and `docs/specs/archive-cli.md`.
