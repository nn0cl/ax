# archiver

[English](README.md)

**ローカルファースト**なアーカイブ用 CLI です。

- USB などの保存先に合わせて、ディレクトリツリーを**複数の圧縮ファイル**に分割（pack）
- 階層に散在したアーカイブを **JSON マッピング**で元の構造に復元（restore）
- 接続方式・空き容量から**高速転送向けのユニットサイズ**を見積もり（estimate）
- 単発の create / list / extract も利用可能

対応形式: **ZIP** / **TAR** / **TAR.GZ** / **TAR.ZST**（zstd・マルチスレッド圧縮）

---

## インストール（macOS / Linux）

詳細は **[docs/INSTALL.md](docs/INSTALL.md)** を参照。

### 最短（ソースから・推奨）

```bash
# 前提: Rust 1.74+ (https://rustup.rs/) と C コンパイラ
# macOS: xcode-select --install
# Linux: sudo apt install build-essential   # Debian/Ubuntu 例
# クレートの正確な版は Cargo.lock / docs/INSTALL.md を参照

cd /path/to/archiver
./scripts/install.sh
# → 既定: ~/.local/bin/ax（+ archiver → ax のシンボリックリンク）

export PATH="$HOME/.local/bin:$PATH"
ax --version
```

| 方法 | コマンド |
|------|----------|
| このリポジトリから release ビルド | `./scripts/install.sh` |
| プレフィックス指定 | `./scripts/install.sh --prefix /usr/local`（権限が要る場合あり） |
| `cargo install` | `./scripts/install.sh --cargo-install` |
| GitHub Release バイナリ | `REPO=owner/repo ./scripts/install.sh --from-release` |
| アンインストール | `./scripts/install.sh --uninstall` |

```bash
# 手動でも可
cargo build --release
install -m 755 target/release/ax ~/.local/bin/ax
ln -sf ax ~/.local/bin/archiver
```

**Windows は公式サポート対象外**です（自己責任でソースビルドは可能な場合があります）。

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

### pack の分割ルール

1. 予算内なら **複数フォルダを 1 アーカイブに同梱**（本数削減）
2. **同一フォルダが予算超過** → **ファイル単位で greedy 分割**（可能な限り unit ≤ 予算）
3. **1 ファイルが予算超過** → **バイト分割しない**（そのファイル単独 unit）
4. **空ディレクトリ**はマッピングに記録し、restore で再現
5. アーカイブは階層散在。複数フォルダ同梱時は **最長共通親** の下に配置
6. 予算は当面 **論理サイズ**（圧縮後は通常より小さい）

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

バイナリ: `target/release/ax`（既定）。互換名は `ln -s ax archiver` または `--features compat-bin`

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

## 製品情報

```bash
archiver --version
archiver --help
```

- **Version**: **v0.0.1**
- **Author**: nn0cl \<nn0cl@dstechnology.co.jp\>
- **Copyright**: (c) nn0cl@dstechnology.co.jp. All rights reserved.
- **依存バージョン**: [docs/INSTALL.md](docs/INSTALL.md)（Rust 1.74+、クレート範囲と `Cargo.lock`）
- **リリース手順**: [docs/RELEASE.md](docs/RELEASE.md) / `./scripts/release.sh X.Y.Z`
- **変更履歴**: [CHANGELOG.md](CHANGELOG.md)

### 免責・自己責任（保証なし）

本ソフトウェアは **現状有姿（AS IS）** で提供され、**明示・黙示を問わず一切の保証はありません**  
（特定目的適合性、データ無損失、継続稼働などを含みますがこれらに限りません）。

- 自動テスト（`cargo test` / CI）は **回帰の抑制** を目的とします。  
  **すべての環境・データでの正しさを保証するものではありません。**
- **SLA（稼働率・応答時間・復旧保証）はありません。**
- バックアップの成否・完全性は **利用者自身の検証**（`verify` 等）と責任範囲です。
- 重要データでは、復元テストと複数媒体への複製を推奨します。

**利用は自己責任（use at your own risk）です。**

### サポート区分（[ADR 0019](docs/architecture/adr/0019-supported-platforms-and-verification-scope.md)）

| 区分 | 範囲 |
|------|------|
| **Supported** | macOS / Linux、通常ファイル、主経路 pack/restore/verify/estimate、plan v2 |
| **Best-effort** | 低速 USB、巨大単一ファイル、`--verify-stage`、コンテンツ種別のサイズ試算 |
| **Unsupported** | Windows 保証なし、同一 output への並列 pack、数百万 inode の体系保証 |

手動チェックリスト: [`docs/QA.md`](docs/QA.md)。製品方針: [ADR 0014](docs/architecture/adr/0014-product-quality-versioning-and-support.md)。  
製品 ADR 要約: [`docs/architecture/adr/README.md`](docs/architecture/adr/README.md)。

### 体系的に検証しきれていない領域（例）

テストや実運用でカバーしている範囲は限定的です。次のようなケースは  
**体系的な自動検証が薄い**ため、失敗・想定外挙動があり得ます。

| 領域 | 意味・起きうること |
|------|-------------------|
| **極端な木** | 数百万 inode、深さ数百、超長いパス、循環に近いリンク構造など。走査時間・メモリ・OS 制限に依存 |
| **特殊権限** | root 専用、ACL、macOS TCC（Desktop/Documents）、SIP、読み取り不可ファイル。sudo や権限不足でスキップ/失敗 |
| **壊れた / 不安定な USB** | 途中切断、書き込みエラー、I/O タイムアウト、見せかけの成功、リトライ不能なメディア不良 |
| **文字化け・特殊名** | 異体字、制御文字、非常に長いファイル名、ケースのみが異なる名（OS 依存） |
| **同時実行** | 同一 output / work-dir への並列 pack、他プロセスとの同時書き込み（**Unsupported**） |
| **全 OS マトリクス** | 主に macOS 実運用 + Linux CI。Windows 等は未保証 |

これらは「テストが無いから品質ゼロ」ではなく、**境界条件を網羅した保証はしていない**という意味です。

## コマンド一覧

**デフォルトバイナリは `ax`**（`cargo build` / `cargo run` / `cargo install`）。  
パッケージ名は `archiver` のまま。互換名 `archiver` は `ln -s ax archiver` か `--features compat-bin`。  
2 文字サブコマンド＋長いエイリアス。グローバル: `-c` 設定 / `-s` 予算 / `-o` 出力。

```bash
ax --help
ax pa --help
```

| 短縮 | エイリアス | 説明 |
|------|------------|------|
| `es` | `estimate -p <path> …` | 高速転送向けユニットサイズの見積 |
| `pa` | `pack -i <dir> -o <dir> (-s SIZE \| --auto-size \| --media) …` | 分割 pack + JSON マッピング（ソースは `-i`） |
| `re` | `restore -m <map.json> -d <dir> …` | マッピングから復元 |
| `ve` | `verify -m <map.json> …` | 復元せずマッピングとアーカイブの整合を検証 |
| `cr` | `create` + グローバル `-o` | 単一アーカイブ作成 |
| `ls` | `list [-f] <archive>` | メンバー一覧 |
| `ex` | `extract -d <dir> [-f] <archive>` | 単一アーカイブ展開 |

例: `ax pa -i ./data -o ./out -s 4G -f tar.zst`

**サイズプリセット**（`-s` / `target_size`）:

| キーワード | 予算 | 用途 |
|------------|------|------|
| `c2` | 10 MiB | 低帯域・細切れ resume（`low-net` も可） |
| `fat32` | 3.8 GiB | FAT32 の 4GB 制限回避 |
| `flash` | 1 GiB | USB メモリ等のバランス |
| `max` | 実質分割なし（16 TiB） | 高速ローカル / 単一アーカイブ志向 |

数値指定（`10M`, `4G`）も従来どおり可。
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
| `-f, --format` | `zip` / `tar` / `tar.gz` / `tar.zst`（**既定: tar.zst**）。`size_estimate` / 進捗 write ETA は **アルゴリズム効率 × 拡張子ベースのコンテンツ種別**で圧縮後サイズを試算（転送完了後は実測比）。空き容量のハード判定は論理サイズのまま |
| `--mapping` | マッピング出力パス（既定: `<output>/archiver-map.json`） |
| `--dry-run` | 計画のみ（アーカイブ・マッピングを書かない） |
| `-v` / `-vv` | テキストログの詳細度（ADR 0023）。**ダッシュボードは `-v` と独立**（TTY なら常に worker 行を表示） |
| `--progress` | 固定行数の進捗ダッシュボードを TTY 上で上書き表示（既定: TTY なら true） |
| `-j, --jobs N` | 並列圧縮ワーカー数（**既定: CPU 数、最大 8**）。指定時は `--at` より優先 |
| `--at` / `--auto-jobs` | plan 後に jobs を自動決定: **CPU × 残 unit × ファイル数 × I/O 上限**（`--connection` / `--throughput` と `--work-dir` 有無を考慮。遅い媒体ではスレッドを抑える。既定 max 8 を外し全体上限 32）。`-j` があるときは無効 |
| `--max-use SIZE` | 低空き容量モード。**1 次アーカイブ（unit）≤ SIZE**（例 `4G`）。同時圧縮は **1 本だけ**、処理は **圧縮 → 転送 → stage 削除 → 次** の直列。stage のピークを unit 1 本分に抑える |

同一 `output` / `work-dir` への **同時 pack はロックで拒否**されます（`output/.ax-pack.lock`）。落ちたプロセスのロックが残った場合はファイルを手で削除してください（ADR 0026）。
| `--work-dir <dir>` | ステージ用の**親ディレクトリ**。直下に `archiver-<random>/` を新規作成（既存ファイルには触れない）。**既定では成否に関わらず削除** |
| `--keep-stage` | デバッグ用にステージを残す（`--work-dir` 必須） |
| `--resume` | `output/archiver-plan.json` を使い、完了済みユニットを fingerprint でスキップ |
| `--check-resources` | output / work-dir の空き容量とメモリ目安を検査（既定 true） |
| `--force-resources` | 空き不足でも続行（非推奨） |
| `--resource-safety-factor` | 所要量見積の倍率（既定 1.0 = 残り論理サイズ相当を output に確保） |
| `--compress-only` | 最終成果を `--output` に完結（外部メディア前提なし）。`--work-dir` 併用時は rename 優先で finalize |
| `--volumes N` | アーカイブを最大 N 枚のボリューム `vol-001/`… に greedy 配置。容量 = 予算（単独指定時は ceil(合計/N)） |
| `--verify-stage` | **オプション**。転送前: ステージ上のアーカイブを展開しメンバー SHA-256 を照合。転送後: 出力先アーカイブの SHA-256 がステージ時と一致すること。CPU・一時ディスク増。既定オフ |
| `--config PATH` | 設定ファイル（`.inf` / `.ini` / `key=value`）。グローバル。`[pack]` セクション。**CLI がファイルより優先** |

### 設定ファイル（`.inf` / `.ini`）

```bash
# 設定ファイルだけ（source/output/target_size 等をファイルに書く）
archiver pack --config ./archiver.inf

# ファイル + CLI 上書き
archiver --config ./archiver.inf pack --jobs 8 --verify-stage
```

例: [`examples/archiver.inf`](examples/archiver.inf)

```ini
[pack]
source = /data/photos
output = /Volumes/USB
target_size = 4G
format = tar.zst
jobs = 4
work_dir = /tmp/archiver-work
verify_stage = true
compress_only = false
```

主な `[pack]` キー: `source`, `output`, `target_size`, `media`, `auto_size`, `volumes`, `format`, `jobs`, `mapping`, `work_dir`, `keep_stage`, `resume`, `compress_only`, `verify_stage`, `check_resources`, `force_resources`, `resource_safety_factor`, `dry_run`, `verbose`, `progress`, `connection`, `target_secs`, `throughput`, `measure`。

### compress-only / volumes の例

```bash
# ローカルに完全パック（後で USB へコピーする前提）
archiver pack -s ~/Photos -o ./pack-out --target-size 4G --format tar.zst \
  --compress-only --jobs 4

# 3 枚のメディア想定（各枚の容量 = --target-size）
archiver pack -s ~/Photos -o ./pack-out --target-size 4G --volumes 3 --format tar.zst

# 予算フラグ無し: 合計を N 等分した容量で unit + volume を決める
archiver pack -s ~/Photos -o ./pack-out --volumes 2 --format zip --dry-run
```

出力レイアウト例（`--volumes 2`）:

```text
pack-out/
  vol-001/docs/part-0001.tar.zst
  vol-001/…
  vol-002/…
  archiver-map.json
  archiver-plan.json
```

`restore` / `verify` は `archive_relpath`（`vol-001/…`）をそのまま使うので追加フラグ不要です。

### 並列パイプライン（`--jobs` + `--work-dir` 推奨）

意図: **確保した圧縮スレッドを USB 待ちで遊ばせない**。低速メディアへの書き込みだけ直列にする。

```text
小さい unit を優先してキュー投入（大きい unit も空きスレッドで並列圧縮）

  [compress × --jobs N]     hash + ローカル圧縮（work-dir）
         │ 完了次第
         ▼  有界キュー（work-dir 時は jobs×4 程度）
  [transfer × 1]            USB 等へ順番にコピー（同時書き込みしない）
```

| 処理 | 並列 | 備考 |
|------|------|------|
| 大・小の圧縮 | **並列**（`--jobs`） | 小が先に終わりやすく、転送を早く始められる |
| USB 書き込み | **直列 1 本** | 遅いメディアの帯域争いを避ける |
| 圧縮スレッドの待ち | なるべく作らない | キューに空きがある限り圧縮を継続。満杯時だけ backpressure |

`--work-dir` 無しだと圧縮先が output 側になり、分離の効果は限定的です。

### resume の考え方（厳格・後方互換なし）

1. pack 開始時（dry-run 以外）に **`archiver-plan.json`（version 2）を先に書く**
2. 各アーカイブ完了ごとに plan を更新（`done` + **archive バイト数** + **archive_sha256** + メンバー sha256）
3. `--resume` は **plan ファイル必須**（無いとエラー。初回 soft 開始はしない）
4. スキップ条件（すべて必須）:
   - メンバー fingerprint（path+size）一致
   - output 上にアーカイブが存在
   - サイズ一致
   - **archive_sha256 再計算一致**（size のみの旧 plan はスキップしない）
5. plan version 不一致もエラー（再 pack）

**work-dir は resume に使いません**

```text
--work-dir /tmp/scratch/          ← 既存 dir でよい（他ファイルがあっても OK）
  keep-me.txt                     ← 触らない
  archiver-a1b2…/                 ← 今回の作業だけ（既定では必ず消える）
```

### 進捗ダッシュボード（TTY）

`/dev/tty`（なければ stderr の TTY）上で **固定行数**を上書き描画します（ADR 0022 / 0024）。  
高さ = `--jobs` + 2（`total` + `cmp0..N` + `trn`）。ワーカーは TTY を触らず、UI スレッドだけが描画します。

```text
total      3/   12   25.0%    85.2M/s  elapsed  1m23s  ETA  4m12s  [compr]
cmp0      90.1M/s  [compr]  part-0003.zip·a1b2
cmp1     120.0M/s  [hash ]  report.docx·c3d4
trn      120.5M   25.0%    40.1M/s  elapsed  1m23s  ETA  4m12s  part-0003.zip·a1b2
```

広い端末（**≥120 列**、描画上限 **200**）では compress 中に成果物と元ファイルを並べます:

```text
cmp0      90.1M/s  [compr]  part-0003.zip·a1b2 | photo.jpg·9c1b
```

| 項目 | 意味 |
|------|------|
| `total` | 全体: unit 進捗・論理％・入力速度・**elapsed / ETA**・フェーズ（**パスなし**） |
| `cmpN` | 圧縮ワーカー N の速度・フェーズ・作業ラベル |
| `trn` | メディア書き出し量・％・書き込み速度・時計・成果物ラベル |
| **elapsed** | 開始からの経過（固定幅） |
| **ETA** | 全処理完了の近似 = `max(入力残り, 転送残り)`（unit 個数ではない） |
| **total %** | ソース**論理バイト**進捗 |
| **trn %** | 書いた量 / 推定総アーカイブ量（圧縮率 prior→実測） |
| **ラベル** | `ファイル名·xxxx`（フルパス由来の **安定 4 桁タグ**。乱数ではない。同名の別パスを区別） |
| 幅 | 設計下限 **80** 列。&lt;120 は 80 キャップ。≥120 は `min(検出, 200)`。幅が取れない環境は 80 |

テキストログの静音化・`-vv` 詳細は ADR 0023 / LISS-0014。ダッシュボードは **`-v` 無しでも worker 行を表示**します。

### verify

復元先に展開せず、マッピングとアーカイブの整合を確認します。

```bash
# フル検証（各アーカイブを一時展開してメンバー SHA-256 を照合）
archiver verify -m ./out/archiver-map.json

# アーカイブが mapping と別ディレクトリにある場合
archiver verify -m ./out/archiver-map.json --archives-root ./out

# 高速: 存在確認 + メンバー一覧の一致のみ（ハッシュなし）
archiver verify -m ./out/archiver-map.json --quick
```

| オプション | 説明 |
|------------|------|
| `-m, --mapping` | `archiver-map.json` |
| `--archives-root` | `archive_relpath` の基準（既定: マッピングの親ディレクトリ） |
| `--quick` | 存在 + 一覧のみ（extract / SHA-256 を省略） |
| `--progress` | 進捗 UI（既定: TTY なら true）。pack と同様に archives / ハッシュバイト / ETA |

問題があると非ゼロ終了し、`issue: ...` を stderr に出します。

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
cargo clippy --all-targets -- -D warnings   # CI / pre-push と同じゲート
```

**CI（GitHub Actions）** と **任意の pre-push hook** の両方で  
`cargo clippy --all-targets -- -D warnings` を実行します。

```bash
./scripts/install-git-hooks.sh   # 一度だけ（core.hooksPath = scripts/git-hooks）
# スキップ: AX_SKIP_CLIPPY=1 git push  または  git push --no-verify
```

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
- 圧縮後サイズを厳密に予算とするモード（[ADR 0013](docs/architecture/adr/0013-compressed-size-packing-mode.md) で延期；試算は [ADR 0018](docs/architecture/adr/0018-content-algorithm-size-estimate.md)）
- GUI / TUI

---

## ライセンス

MIT（プロジェクト固有の LICENSE ファイルは必要に応じて追加）
