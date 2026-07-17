//! Integration tests for archive create / list / extract (spec: archive-cli.md).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn archiver() -> Command {
    // Primary binary is `ax`; `archiver` remains a compat alias.
    Command::cargo_bin("ax").unwrap()
}

fn write_tree(root: &Path) {
    fs::write(root.join("a.txt"), b"hello a").unwrap();
    fs::create_dir_all(root.join("dir")).unwrap();
    fs::write(root.join("dir/b.txt"), b"hello b").unwrap();
}

#[test]
fn create_list_extract_zip() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("ws");
    fs::create_dir_all(&workspace).unwrap();
    write_tree(&workspace);

    let archive = dir.path().join("out.zip");
    archiver()
        .args([
            "create",
            "--output",
            archive.to_str().unwrap(),
            workspace.join("a.txt").to_str().unwrap(),
            workspace.join("dir").to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(archive.exists());

    archiver()
        .args(["list", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("dir/b.txt"));

    let dest = dir.path().join("dest");
    archiver()
        .args([
            "extract",
            "--dest",
            dest.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dest.join("a.txt")).unwrap(), "hello a");
    assert_eq!(
        fs::read_to_string(dest.join("dir/b.txt")).unwrap(),
        "hello b"
    );
}

#[test]
fn create_tar_gz_infers_format() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, b"data").unwrap();
    let archive = dir.path().join("bundle.tar.gz");

    archiver()
        .args([
            "create",
            "--output",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("tar.gz"));

    archiver()
        .args(["list", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));
}

#[test]
fn create_plain_tar() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, b"data").unwrap();
    let archive = dir.path().join("bundle.tar");

    archiver()
        .args([
            "create",
            "-o",
            archive.to_str().unwrap(),
            "-f",
            "tar",
            file.to_str().unwrap(),
        ])
        .assert()
        .success();

    let dest = dir.path().join("out");
    archiver()
        .args([
            "extract",
            "--dest",
            dest.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dest.join("a.txt")).unwrap(), "data");
}

#[test]
fn rejects_missing_input() {
    let dir = tempdir().unwrap();
    let archive = dir.path().join("out.zip");
    archiver()
        .args([
            "create",
            "--output",
            archive.to_str().unwrap(),
            dir.path().join("missing.txt").to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn rejects_unknown_format_extension() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, b"x").unwrap();
    let archive = dir.path().join("out.unknown");

    archiver()
        .args([
            "create",
            "--output",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("format"));
}

#[test]
fn rejects_path_traversal_on_zip_extract() {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let dir = tempdir().unwrap();
    let archive = dir.path().join("evil.zip");
    {
        let file = fs::File::create(&archive).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("../escape.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"pwned").unwrap();
        zip.finish().unwrap();
    }

    let dest = dir.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    archiver()
        .args([
            "extract",
            "--dest",
            dest.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("traversal"));

    assert!(!dir.path().join("escape.txt").exists());
}
