# archiver

[English](README.md)

**ローカルファースト**なアーカイブ用 CLI です。

- USB などの保存先に合わせて、ディレクトリツリーを**複数の圧縮ファイル**に分割（pack）
- 階層に散在したアーカイブを **JSON マッピング**で元の構造に復元（restore）
- 接続方式・空き容量から**高速転送向けのユニットサイズ**を見積もり（estimate）
- 単発の create / list / extract も利用可能

対応形式: **ZIP** / **TAR** / **TAR.GZ** / **TAR.ZST**（zstd・マルチスレッド圧縮）

---

## 目的

大きなディレクトリを、保存メディアの事情（容量・USB の実効速度）に合わせて
扱いやすい大きさのアーカイブに分けて書き出します。

圧縮してまとめることで、

1. **転送バイト数が減る**（圧縮が効くデータ）
2. **小さなファイル大量コピーよりシーケンシャル書き込みに近づく**

といった効果で、USB への退避がしやすくなることを意図しています。
（バス速度そのものが上がるわけではありません。）

---

## 主な機能

| 機能 | 説明 |
|------|------|
| **pack** | ソースツリーを走査し、予算単位で複数アーカイブ + `archiver-map.json` を出力 |
| **restore** | マッピングとアーカイブから元ツリーを復元（既定で SHA-256 検証） |
| **estimate** | 保存先パスの空き容量・接続クラスから推奨ユニットサイズを算出 |
| **create / list / extract** | 単一アーカイブの作成・一覧・展開 |

### pack の分割ルール（受け入れ済み）

1. **できるだけ 1 フォルダ = 1 アーカイブ**（そのフォルダ直下のファイル）
2. フォルダ内容が予算超過 → **拡張子ごとにまとめて**圧縮。さらに大きければサイズで分割
3. **1 ファイルが予算超過**でもバイト分割せず、そのファイル単独のアーカイブにする
4. **空ディレクトリ**はマッピングに記録し、restore で再現
5. アーカイブは出力先に**階層散在**（例: `docs/part-0001.zip`）

### 予算の決め方

次のいずれか（排他）:

| 方式 | フラグ | 意味 |
|------|--------|------|
| 明示サイズ | `--target-size 700M` | 指定バイトをベースに接続係数を適用 |
| メディアプリセット | `--media cd\|dvd\|bd25` | 既定容量 × 接続係数 |
| 転送見積 | `--auto-size` | 保存先を probe し「実用書き込み速度 × 秒数」でユニットサイズを決定 |

接続クラス（`--connection`）:

| 値 | 用途の目安 | 既定の実用書き込み速度（見積用） |
|----|------------|----------------------------------|
| `local` | 内蔵ディスク | 400 MiB/s |
| `usb3` | USB 3.x メモリ等 | 120 MiB/s |
| `usb2` | USB 2.0 | 25 MiB/s |
| `thunderbolt` | Thunderbolt | 800 MiB/s |
| `network` | SMB/NFS 等 | 40 MiB/s |

見積のイメージ:

```text
推奨ユニット ≈ 実用スループット × 目標秒数（既定 45s）
             （4 MiB〜4 GiB にクランプ、空き容量で上限）
```

`--connection` 未指定時は自動検出（best-effort）:

1. CLI の明示指定
2. プラットフォーム情報（macOS では `diskutil` の Protocol など）
3. パスヒューリスティクス（`/Volumes` 等 → usb3 寄り）
4. `local` にフォールバック

---

## 必要環境

- Rust（edition 2021 対応の安定版）
- Cargo
- （任意）USB や外付けボリュームのマウント先

---

## ビルド

```bash
cargo build --release
```

バイナリ: `target/release/archiver`

```bash
# 動作確認
cargo run -- --help
cargo test
```

---

## 使い方

### 1. 転送サイズの見積もり

```bash
# USB マウント先を probe（接続クラスを明示）
archiver estimate --path /Volumes/USB --connection usb3

# 短い書き込みサンプルで実測スループットを使う
archiver estimate --path /Volumes/USB --measure

# スループットを手で指定（例: 80 MiB/s）
archiver estimate --path /Volumes/USB --throughput 80M --target-secs 30
```

出力例の意味:

- `recommended_unit` … pack の予算として使う推奨サイズ
- `detection` … 接続クラスの判定元（`cli_hint` など）
- `media_ceiling` … 空き容量から見た上限
- `measured` … 書き込み微測定を使ったか

### 2. pack（分割アーカイブ + マッピング）

```bash
# 見積サイズで自動 pack（推奨の運用イメージ）
archiver pack \
  --source ./photos \
  --output /Volumes/USB/backup \
  --auto-size \
  --connection usb3 \
  --format tar.zst \
  --jobs 4 \
  -v

# 事前に計画だけ確認（ディスクに書かない）
archiver pack \
  --source ./photos \
  --output /Volumes/USB/backup \
  --auto-size \
  --connection usb3 \
  --dry-run

# 明示サイズ
archiver pack -s ./data -o ./out --target-size 700M --format zip

# DVD 相当の容量プリセット
archiver pack -s ./data -o ./out --media dvd --connection usb3 -f tar.gz
```

出力のイメージ:

```text
output/
  archiver-map.json      # 復元用マッピング（SHA-256 付き）
  docs/part-0001.zip
  photos/part-0001.zip
  part-0001.zip
```

### 3. restore（復元）

```bash
# 既定で SHA-256 検証あり
archiver restore \
  --mapping /Volumes/USB/backup/archiver-map.json \
  --dest ./restored

# アーカイブのルートがマッピングと別にある場合
archiver restore \
  -m ./out/archiver-map.json \
  -d ./restored \
  --archives-root ./out

# 検証をオフ（非推奨）
archiver restore -m ./out/archiver-map.json -d ./restored --verify false
```

### 4. 単一アーカイブ（低レベル）

```bash
archiver create -o backup.zip ./src README.md
archiver list backup.zip
archiver extract -d ./restored backup.zip
```

展開時、アーカイブ内の **パストラバーサル**（`../` 等）は拒否します。

---

## コマンド一覧

| コマンド | 説明 |
|----------|------|
| `estimate -p <path> [options]` | 高速転送向けユニットサイズの見積 |
| `pack -s <dir> -o <dir> (--auto-size \| --target-size \| --media) [options]` | 分割 pack + JSON マッピング |
| `restore -m <map.json> -d <dir> [options]` | マッピングから復元 |
| `create -o <path> [-f] <入力...>` | 単一アーカイブ作成 |
| `list [-f] <archive>` | メンバー一覧 |
| `extract -d <dir> [-f] <archive>` | 単一アーカイブ展開 |

### pack の主なオプション

| オプション | 説明 |
|------------|------|
| `-s, --source` | 入力ディレクトリ |
| `-o, --output` | 出力ルート |
| `--target-size` | 予算サイズ（例: `700M`, `4G`） |
| `--media` | `cd` / `dvd` / `bd25` |
| `--auto-size` | 出力先の見積から予算決定 |
| `--connection` | `local` / `usb2` / `usb3` / `thunderbolt` / `network` |
| `--target-secs` | 見積の転送時間ウィンドウ（既定 45） |
| `--throughput` | スループット上書き（バイト/秒、例: `80M`） |
| `--measure` | 書き込み微測定（estimate / auto-size） |
| `-f, --format` | `zip` / `tar` / `tar.gz` / `tar.zst` |
| `--mapping` | マッピング出力パス（既定: `<output>/archiver-map.json`） |
| `--dry-run` | 計画のみ（アーカイブ・マッピングを書かない） |
| `-v, --verbose` | 進捗を stderr に出力 |
| `--jobs N` | 並列圧縮ワーカー数（既定 1） |

### estimate の主なオプション

| オプション | 説明 |
|------------|------|
| `-p, --path` | 保存先パス |
| `--connection` | 接続クラス |
| `--target-secs` | 1 ユニット転送の目標秒数 |
| `--throughput` | スループット上書き |
| `--measure` | 短いシーケンシャル書き込みで実測 |
| `--measure-sample` | 測定サンプルサイズ（既定 64M） |

### restore の主なオプション

| オプション | 説明 |
|------------|------|
| `-m, --mapping` | `archiver-map.json` |
| `-d, --dest` | 復元先ディレクトリ |
| `--archives-root` | アーカイブ相対パスの基準（既定: マッピングの親） |
| `--verify` | SHA-256 検証（既定 `true`） |

---

## マッピングファイル

pack 完了時に JSON（schema version 2）を書き出します。主な内容:

- 各アーカイブの相対パス
- メンバーの元相対パス・サイズ・**SHA-256**
- 空ディレクトリ一覧
- 使用した予算・形式

restore はこれを読んで展開し、既定でハッシュを検証します。

仕様の詳細: [`docs/specs/media-fit-pack.md`](docs/specs/media-fit-pack.md)

---

## 開発

このリポジトリは AI–人間協働テンプレート（llm-project-template）を採用しています。

| 文書 | 内容 |
|------|------|
| [`docs/specs/media-fit-pack.md`](docs/specs/media-fit-pack.md) | pack / restore / estimate 仕様 |
| [`docs/specs/media-fit-acceptance.md`](docs/specs/media-fit-acceptance.md) | 受け入れテスト対応表 |
| [`docs/specs/archive-cli.md`](docs/specs/archive-cli.md) | 単一アーカイブ CLI 仕様 |
| [`docs/architecture/rust-cli-architecture.md`](docs/architecture/rust-cli-architecture.md) | レイヤ構成 |
| [`docs/work-plans/WP-0002-media-fit-archiver-delivery.md`](docs/work-plans/WP-0002-media-fit-archiver-delivery.md) | 実装計画 |
| [`AGENTS.md`](AGENTS.md) | エージェント契約 |

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

CI（GitHub Actions）: `cargo test` と `clippy -D warnings`（Ubuntu）。

### ディレクトリ構成

```text
src/
  core/domain/       # 分割計画・予算・マッピング型・パス安全性
  core/application/  # pack / restore / estimate / create / list / extract
  core/ports/        # ArchiveEngine, FileScanner, FileSystem など
  adapters/          # zip/tar/zstd、FS、JSON マッピング、測定
  delivery/          # clap CLI
tests/               # 統合テスト
docs/                # 仕様・ADR・issue・協働ドキュメント
```

---

## スコープ外（現状）

- ファイル内容のバイト分割
- 複数物理メディアへの自動ボリュームまたぎ
- 暗号化アーカイブ
- 圧縮後サイズを厳密に予算とするモード（[ADR 0013](docs/architecture/adr/0013-compressed-size-packing-mode.md) で延期）
- GUI / TUI

---

## ライセンス

MIT（プロジェクト固有の LICENSE ファイルは必要に応じて追加）
