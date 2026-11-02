# v3 公開前のドキュメント修正項目

作成日: 2026-11-02
更新日: 2026-11-02

v3 公開前に修正したい項目と合意した方針を記録する一時ドキュメント。文書間の責任分離は再構成を実施した。英語化と、README・Cargo.toml の crate の並び順の修正は未着手である。

## 1. 文書間の責任分離

`docs/` 以下の各ドキュメントの責任が適切に分離されていない。各文書が扱う範囲を整理し、その責任に沿って記述を配置する。

### 責任分担（確定）

| 文書 | 責任 |
| --- | --- |
| `requirements.md` | プロジェクトの目的・責任範囲・機能要件など、「何を作っているか」を記述する。 |
| `architecture.md` | プロジェクトの全体構造など、「何を提供しているか」を大まかに記述する。 |
| `future-work.md` | 今後実装する機能を記述する。 |
| `design/xxx` | 各機能単位ごとに、「何を提供しているか」を詳述する。 |
| `plans/xxx` | 現在進行形で実装を進めているものについて、その計画や経過を記述する。 |

既存文書の整理はこの責任分担に従う。現在提供している内容は `architecture.md` と `design/` に概要と詳細を分けて記述し、今後実装する機能と現在進行中の計画・経過は、それぞれ `future-work.md` と `plans/` に配置する。

### 実施した整理

文書の案内を [docs/README](../../README.md) に追加した。Requirements は目的・範囲・機能要件・非機能要件・受入条件、architecture は全体構造の概要、design は現在の機能別仕様に整理した。未実装機能と未着手の API 候補は `future-work/`、公開準備で参照する検証の履歴は[v3 の経過](verification.md)に配置した。

英語化と README・Cargo.toml の並べ替えは別項目として残す。

## 2. ドキュメントの英語化

`docs/` 以下のドキュメントはこれまで日本語で書いてきたが、英語へ移行する。

### 移行方針（確定）

`requirements.md`・`architecture.md`・`future-work.md`・`design/`・`plans/` を含む、`docs/` 以下のドキュメントを英語に統一する。第1項の責任分担に沿って文書を整理し、英語化する。

## 3. ルート README の crate 一覧と Cargo.toml の依存一覧の順序

ディレクトリトップの [README](../../../README.md#crates) に掲載されている crate が、それぞれの責任に基づく意味論に沿った順序になっていない。各 crate の責任と関係を踏まえて、掲載順序を見直す。

### 掲載方針

共通の中核を担う `syntaqs-core` を先頭とし、識別子・幾何・共通機能を続け、具体的な局所代数を担う `syntaqs-spin-half` などは後方にまとめる。

具体的な掲載順序案は以下とする。

| 順序 | Crate | 責任 |
| --- | --- | --- |
| 1 | `syntaqs-core` | 共通の代数契約・式・空間・評価・基底処理。 |
| 2 | `syntaqs-basic` | 因子を識別する UUID・名前付き Index。 |
| 3 | `syntaqs-geometry` | 幾何・Site 間の関係と外部データへの接続。 |
| 4 | `syntaqs-normalize` | 共通のノルム・正規化・正規化期待値。 |
| 5 | `syntaqs-export` | 共通の数値コンテナへの出力。 |
| 6 | `syntaqs-spin-half` | Spin-1/2 の局所代数・基底・モデル。 |
| 7 | `syntaqs-spin-one` | Spin-1 の局所代数・基底・モデル。 |
| 8 | `syntaqs-fermion` | Fermion の局所代数・基底・モデル。 |

### Cargo.toml への適用

**適用範囲・方針は確定。** Cargo.toml の依存一覧も、各依存の責任・役割に基づく意味論的な順序で整備する。

- `syntaqs-*` だけでなく、外部依存も対象とする。プロジェクト内 crate の相対順序は上表にそろえ、外部依存もその責任・役割を踏まえて並べる。
- ルートの `[workspace.dependencies]` に加え、各 crate の Cargo.toml にも同じ方針を適用する。`examples`・`tests` の Cargo.toml も対象とする。
- `[dependencies]`・`[dev-dependencies]`・`[build-dependencies]` など、各依存セクション内でこの方針に従って並べる。
