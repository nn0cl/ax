//! Integration tests for media-fit pack / restore (spec: media-fit-pack.md).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn archiver() -> Command {
    Command::cargo_bin("archiver").unwrap()
}

fn write_file(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn pack_one_folder_one_archive_when_fits() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("docs/a.txt"), b"aaa");
    write_file(&source.join("docs/b.txt"), b"bbb");

    let output = dir.path().join("out");
    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1M",
            "--format",
            "zip",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 archive"));

    let map = output.join("archiver-map.json");
    assert!(map.exists());
    assert!(output.join("docs/part-0001.zip").exists());

    let dest = dir.path().join("restored");
    archiver()
        .args([
            "restore",
            "--mapping",
            map.to_str().unwrap(),
            "--dest",
            dest.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dest.join("docs/a.txt")).unwrap(), "aaa");
    assert_eq!(fs::read_to_string(dest.join("docs/b.txt")).unwrap(), "bbb");
}

#[test]
fn pack_splits_by_extension_when_folder_too_large() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    // Each file 400 bytes; budget 1000 → folder total 1200 > budget → by ext
    write_file(&source.join("photos/a.jpg"), &vec![1u8; 400]);
    write_file(&source.join("photos/b.jpg"), &vec![2u8; 400]);
    write_file(&source.join("photos/c.png"), &vec![3u8; 400]);

    let output = dir.path().join("out");
    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1000",
            "--format",
            "zip",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 archive"));

    assert!(output.join("photos/part-0001.zip").exists());
    assert!(output.join("photos/part-0002.zip").exists());
}

#[test]
fn pack_accepts_huge_single_file() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("big/video.mkv"), &vec![9u8; 5000]);
    write_file(&source.join("big/note.txt"), b"hi");

    let output = dir.path().join("out");
    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1000",
        ])
        .assert()
        .success();

    let dest = dir.path().join("restored");
    let map = output.join("archiver-map.json");
    archiver()
        .args([
            "restore",
            "--mapping",
            map.to_str().unwrap(),
            "--dest",
            dest.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_eq!(fs::read(dest.join("big/video.mkv")).unwrap().len(), 5000);
    assert_eq!(fs::read_to_string(dest.join("big/note.txt")).unwrap(), "hi");
}

#[test]
fn pack_records_and_restores_empty_directories() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("keep/file.txt"), b"x");
    fs::create_dir_all(source.join("empty/nested")).unwrap();

    let output = dir.path().join("out");
    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1M",
        ])
        .assert()
        .success();

    let map_text = fs::read_to_string(output.join("archiver-map.json")).unwrap();
    assert!(map_text.contains("empty/nested") || map_text.contains("empty"));

    let dest = dir.path().join("restored");
    archiver()
        .args([
            "restore",
            "--mapping",
            output.join("archiver-map.json").to_str().unwrap(),
            "--dest",
            dest.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(dest.join("empty/nested").is_dir());
    assert_eq!(fs::read_to_string(dest.join("keep/file.txt")).unwrap(), "x");
}

#[test]
fn pack_tar_zst_round_trip() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("a.txt"), b"zstd-me");

    let output = dir.path().join("out");
    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1M",
            "--format",
            "tar.zst",
        ])
        .assert()
        .success();

    assert!(output.join("part-0001.tar.zst").exists());

    let dest = dir.path().join("restored");
    archiver()
        .args([
            "restore",
            "--mapping",
            output.join("archiver-map.json").to_str().unwrap(),
            "--dest",
            dest.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dest.join("a.txt")).unwrap(), "zstd-me");
}

#[test]
fn pack_requires_budget() {
    let dir = tempdir().unwrap();
    archiver()
        .args([
            "pack",
            "--source",
            dir.path().to_str().unwrap(),
            "--output",
            dir.path().join("out").to_str().unwrap(),
        ])
        .assert()
        .failure();
}

#[test]
fn estimate_reports_recommended_unit() {
    let dir = tempdir().unwrap();
    archiver()
        .args([
            "estimate",
            "--path",
            dir.path().to_str().unwrap(),
            "--connection",
            "usb2",
            "--target-secs",
            "45",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("recommended_unit:"))
        .stdout(predicate::str::contains("Usb2"));
}

#[test]
fn pack_auto_size_uses_estimate() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("a.txt"), b"hello");
    let output = dir.path().join("out");

    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--auto-size",
            "--connection",
            "usb3",
            "--target-secs",
            "30",
            "--format",
            "zip",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("budget"));

    assert!(output.join("archiver-map.json").exists());
}

#[test]
fn restore_verify_detects_tamper() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("a.txt"), b"original");
    let output = dir.path().join("out");

    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1M",
        ])
        .assert()
        .success();

    let map = fs::read_to_string(output.join("archiver-map.json")).unwrap();
    assert!(map.contains("sha256"));

    let dest = dir.path().join("restored");
    archiver()
        .args([
            "restore",
            "--mapping",
            output.join("archiver-map.json").to_str().unwrap(),
            "--dest",
            dest.to_str().unwrap(),
            "--verify",
            "true",
        ])
        .assert()
        .success();

    // Tamper after a clean restore, then re-verify by restoring again over dest
    // after corrupting the mapping expected hash path content is checked post-extract.
    fs::write(dest.join("a.txt"), b"tampered!").unwrap();
    // Re-run verify-only path: restore extracts again then verifies.
    // Corrupt the on-disk file that extract will overwrite — instead flip mapping hash.
    let mut map_val: serde_json::Value = serde_json::from_str(&map).unwrap();
    map_val["archives"][0]["members"][0]["sha256"] =
        serde_json::Value::String("0".repeat(64));
    let bad_map = dir.path().join("bad-map.json");
    fs::write(&bad_map, serde_json::to_string_pretty(&map_val).unwrap()).unwrap();

    let dest2 = dir.path().join("restored2");
    archiver()
        .args([
            "restore",
            "--mapping",
            bad_map.to_str().unwrap(),
            "--dest",
            dest2.to_str().unwrap(),
            "--archives-root",
            output.to_str().unwrap(),
            "--verify",
            "true",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("hash mismatch").or(predicate::str::contains("Hash")));
}

#[test]
fn pack_dry_run_writes_nothing() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("a.txt"), b"x");
    let output = dir.path().join("out");
    fs::create_dir_all(&output).unwrap();

    archiver()
        .args([
            "pack",
            "--source",
            source.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-size",
            "1M",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"));

    let entries: Vec<_> = fs::read_dir(&output)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.is_empty(),
        "dry-run must not write archives or mapping: {entries:?}"
    );
}

#[test]
fn pack_jobs_deterministic_members() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("src");
    write_file(&source.join("d1/a.txt"), b"a");
    write_file(&source.join("d2/b.txt"), b"b");
    write_file(&source.join("d3/c.txt"), b"c");

    let out1 = dir.path().join("out1");
    let out2 = dir.path().join("out2");

    for (out, jobs) in [(&out1, "1"), (&out2, "4")] {
        archiver()
            .args([
                "pack",
                "--source",
                source.to_str().unwrap(),
                "--output",
                out.to_str().unwrap(),
                "--target-size",
                "1M",
                "--jobs",
                jobs,
            ])
            .assert()
            .success();
    }

    let m1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out1.join("archiver-map.json")).unwrap()).unwrap();
    let m2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out2.join("archiver-map.json")).unwrap()).unwrap();

    let paths = |m: &serde_json::Value| -> Vec<String> {
        let mut v = Vec::new();
        for a in m["archives"].as_array().unwrap() {
            for mem in a["members"].as_array().unwrap() {
                v.push(mem["source_relpath"].as_str().unwrap().to_string());
            }
        }
        v.sort();
        v
    };
    assert_eq!(paths(&m1), paths(&m2));
}

#[test]
fn estimate_shows_detection() {
    let dir = tempdir().unwrap();
    archiver()
        .args([
            "estimate",
            "--path",
            dir.path().to_str().unwrap(),
            "--connection",
            "usb2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("detection:"))
        .stdout(predicate::str::contains("cli_hint"));
}

#[test]
fn application_layer_has_no_std_fs() {
    // Architecture gate (LISS-0004): use cases must not call std::fs directly.
    let app_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/core/application");
    for entry in fs::read_dir(&app_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("std::fs"),
            "{} must not reference std::fs",
            path.display()
        );
    }
}
