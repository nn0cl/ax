# Feature: Media-Fit Pack / Restore

> 状態: **accepted** (Adjudicator 2026-07-17)

関連: 低レベル単一アーカイブは `docs/specs/archive-cli.md`。

## 目的

保存先メディアと接続方式に応じた予算で、指定パス以下のツリーを複数の
圧縮ファイルに分割して階層散在配置する。復元は JSON マッピングを用いる。

## 確定した方針

| 項目 | 決定 |
|------|------|
| 分割の第一原則 | **できるだけ 1 フォルダ = 1 アーカイブ** |
| フォルダ過大時 | そのフォルダ内の**直下ファイル**を **拡張子ごとにまとめて**圧縮 |
| 予算超過の 1 ファイル | **受け入れ**、分割せず単独アーカイブ |
| 単独閾値 | 予算と同じ。1 ファイルなら閾値超過可 |
| 空ディレクトリ | マッピングに記録し restore で再現 |
| 配置 | 階層に散在（各フォルダ下に `part-NNNN.*`） |
| マッピング | JSON |
| 形式 | `zip`, `tar.gz`, およびマルチスレッド向け `tar.zst` (zstd) |
| CLI | `pack` / `restore` を追加。既存 `create`/`list`/`extract` は維持 |
| 予算調整 | メディア種別 + 接続方式、または **転送見積 (`estimate` / `--auto-size`)** |

## EARS

When the user packs a source directory with a media budget, the system shall
prefer one archive per directory for that directory's immediate files when
their total logical size is within budget.

When a directory's immediate files exceed the budget, the system shall group
those files by file extension and pack each group (further splitting a group
by size only when the group still exceeds the budget).

When a single file exceeds the budget, the system shall still place it alone
in one archive without splitting the file bytes.

When packing completes, the system shall write a JSON mapping that lists every
archive, every member with original relative path, and every empty directory.

When the user restores from a mapping, the system shall recreate files and
empty directories under the destination according to the mapping.

## Gherkin

```gherkin
Scenario: One folder fits in one archive
  Given directory "docs" with files totaling under the budget
  When pack runs
  Then exactly one archive is placed under "docs/"
  And the mapping lists those files under that archive

Scenario: Oversized folder splits by extension
  Given directory "photos" whose immediate files exceed the budget
  And files with extensions ".jpg" and ".png"
  When pack runs
  Then jpg files and png files are not mixed in the same archive
  unless a further size split within one extension is required

Scenario: Huge single file is accepted alone
  Given a file larger than the budget
  When pack runs
  Then the mapping contains one archive whose only member is that file

Scenario: Empty directories are restored
  Given an empty directory "empty/nested" under the source
  When pack then restore
  Then "empty/nested" exists under the destination

Scenario: Round-trip preserves file bytes
  Given a packed output with mapping
  When restore runs
  Then every source file content matches at the original relative path
```

## パッキング規則（規範）

### スコープ

- 各ディレクトリについて、**直下の通常ファイルのみ**をそのディレクトリの
  候補集合とする（サブディレクトリ内のファイルはサブ側で処理）。
- 走査はソースルート以下すべて。シンボリックリンクは **スキップ**。

### アルゴリズム

```text
for each directory D (including root), with immediate files F:
  if F is empty:
    record D as empty-dir candidate if D has no children files anywhere
      handled globally: any dir with zero files in whole subtree still listed
  else if sum(size(f) for f in F) <= budget:
    emit one PackUnit for F   // 1フォルダ1アーカイブ
  else:
    group F by lowercase extension (no-extension → "")
    for each extension group G in stable order:
      bin-pack G into units with capacity = budget
      // 単一ファイルが budget 超過でも 1 ユニットとして emit（分割しない）
```

空ディレクトリ: ソース上に存在し、**直下にも子孫にもファイルが無い**
ディレクトリをマッピングの `empty_directories` に相対パスで列挙する。
（ファイルを持つディレクトリのパスはアーカイブ配置から復元可能なので
必須ではないが、空のみ必須。）

### 予算

2 系統ある。

#### A. 容量ベース（従来）

```text
effective_budget = floor(media_capacity_bytes * connection_factor) - mapping_reserve
```

| media | capacity (logical budget base) |
|-------|--------------------------------|
| `cd` | 700 MiB |
| `dvd` | 4482 MiB (~4.7GB 実用) |
| `bd25` | 23866 MiB |
| `custom` / `--target-size` | 明示値 |

| connection | factor |
|------------|--------|
| `local` | 1.00 |
| `usb3` | 0.98 |
| `usb2` | 0.95 |
| `network` | 0.90 |
| `thunderbolt` | 0.99 |

#### B. 転送見積ベース（USB 接続情報など）

**目的**: メディア全容量ではなく、「**その接続で高速に書き切れる 1 ユニットサイズ**」を
パック予算にする。

```text
throughput = measured OR practical_default(connection)
raw_unit   = throughput_bytes_per_sec × target_transfer_secs   # 既定 45s
unit       = clamp(raw_unit, 4MiB, 4GiB)
if free_bytes known:
  ceiling = free_bytes - mapping_reserve
  unit    = min(unit, ceiling)
```

| connection | practical write throughput（既定） |
|------------|-------------------------------------|
| `local` | 400 MiB/s |
| `thunderbolt` | 800 MiB/s |
| `usb3` | 120 MiB/s（スティック実効寄り） |
| `usb2` | 25 MiB/s |
| `network` | 40 MiB/s |

- 空き容量は出力先パスを `statvfs` 等で probe。
- 接続クラスは `--connection` 明示、またはマウントパスヒューリスティック
  （`/Volumes` `/media` `/mnt` → usb3 既定、それ以外 local）。
- USB `bcdUSB` が取れる環境では 2.x → usb2、3.x+ → usb3 に写像可能
  （`connection_from_bcd_usb`）。
- 実測スループットは `--throughput 80M` で上書き可能。

`mapping_reserve` 既定: min(1 MiB, 1% of free/budget)。

パック時のファイル合計の測り方: **論理ファイルサイズ合計**（非圧縮）。

### 配置

- ユニットが属するディレクトリ相対パス `D` の下に  
  `part-NNNN.zip` / `part-NNNN.tar.gz` / `part-NNNN.tar.zst`
- `NNNN` はディレクトリごとに 1 始まり連番
- ルート直下ユニットは output ルートに配置

### アーカイブ内メンバー名

`source_relpath`（`/` 区切り）をそのまま member 名に使う。

## マッピング JSON schema (v1)

```json
{
  "version": 2,
  "source_root_name": "data",
  "media_budget_bytes": 1048576,
  "format": "zip",
  "size_basis": "logical",
  "empty_directories": ["empty/nested"],
  "archives": [
    {
      "id": "a0001",
      "archive_relpath": "docs/part-0001.zip",
      "logical_bytes": 1234,
      "members": [
        {
          "source_relpath": "docs/a.txt",
          "size": 100,
          "sha256": "..."
        }
      ]
    }
  ]
}
```

Restore verifies `sha256` by default (`--verify true`).

## CLI

```text
# 転送見積のみ（USB 等の出力先を probe）
archiver estimate \
  --path /Volumes/USB \
  [--connection usb2|usb3|...] \
  [--target-secs 45] \
  [--throughput 80M] \
  [--measure] [--measure-sample 64M]

# 見積サイズで pack
archiver pack \
  --source <dir> \
  --output /Volumes/USB/backup \
  --auto-size \
  [--connection usb3] \
  [--target-secs 45] \
  [--throughput 80M] \
  [--measure] \
  [--dry-run] [-v] [--jobs N] \
  [--format zip|tar.gz|tar.zst]

# 容量 / 明示サイズで pack
archiver pack \
  --source <dir> \
  --output <dir> \
  (--target-size <SIZE> | --media <cd|dvd|bd25>) \
  [--connection local|usb2|usb3|thunderbolt|network] \
  [--format zip|tar.gz|tar.zst] \
  [--mapping <path>]

archiver restore \
  --mapping <path> \
  --dest <dir> \
  [--archives-root <dir>]
```

SIZE: 整数バイト、または `700M` / `4G` 形式。

## External Dependencies

- Filesystem scan / write via adapters
- `ArchiveEngine` (zip, tar.gz, tar.zst)
- JSON mapping read/write

## Out of Scope

- ファイル内容のバイト分割
- 複数物理メディアへの自動ボリュームまたぎ
- 暗号化
- 圧縮後サイズ厳密保証
- シンボリックリンクの保存

## Ambiguities (resolved)

- 拡張子グループ内がなお予算超過 → サイズ順ビンパッキング、単独超過は 1 アーカイブ
- 拡張子なし → 空文字キーで 1 グループ
- 大文字小文字 → 拡張子は小文字化して比較
