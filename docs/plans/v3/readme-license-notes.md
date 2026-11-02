# README・ライセンス整備の作業メモ（一時）

作成日: 2026-11-01
更新日: 2026-11-02

[README・ライセンス整備計画](README.md#プロジェクトルートの-readme-とライセンスファイル)に反映するため、会話で決めた事項を一時的に記録する。

## 名前と発音

- プロジェクト名は **syntaqs**。
- 由来は **synta**x for **q**uantum **s**ystems。
- 読みは英語の **syntax** と同じ「シンタックス」とする。
- README には発音記号 **/ˈsɪn.tæks/** を掲載する。参照: [Cambridge Dictionary — syntax](https://dictionary.cambridge.org/pronunciation/english/syntax)。

README での表記案:

> syntaqs /ˈsɪn.tæks/ — syntax for quantum systems

今回の新名称について、同名ソフトウェアの調査と crates.io での登録可否の確認は未実施。

- crate 名とそのディレクトリ名は `syntaqs-*` に統一する。横断テスト用パッケージも `syntaqs-tests` とする。
- import、依存設定、利用例、文書内の名称とパスを新名称に統一する。
- GitHub のリポジトリ名は **nattyo1226/syntaqs**、origin は `git@github.com:nattyo1226/syntaqs.git`。2026-11-02 に両方とも改名済みであることを確認した。

## ライセンス

- **MIT** を単独で採用する。
- 著作権表示は **Copyright (c) 2026 Haruki SAITO** とする。著作権者は本名で表記する。
- 適用範囲は **リポジトリ全体**とする。`crates/` 内に限定せず、自作のコード・利用例・テスト・関連文書を含める。
- 研究・商用を問わず利用できる方針とする。
- ライセンスファイル、README の案内、Cargo のライセンス指定を整合させる。

ルートに LICENSE を配置し、全体への適用を案内する。MIT 本文はソフトウェアと関連文書を対象とする。参照: [MIT 原文](https://opensource.org/license/mit)。

## README の言語と導入方法

- プロジェクトと各 crate の README は **英語**とする。
- トップの README はプロジェクトの説明・導入方法・最小例・crate 一覧・開発方法をまとめる。
- 各 crate の README は担当責任・主な API・他の crate との関係・関連文書へのリンクをまとめる。
- 現時点の導入方法は **Git 依存**として案内する。
- 将来的には **crates.io への公開を目指す**。今回の方針確定は公開操作の依頼ではない。

## 作業状況

既存の利用者向け README である `examples/README.md` と `tests/README.md` を英訳した。トップと8つのライブラリ crate の README を英語で作成した。設計・計画文書は既存の日本語を維持している。

crate 名・ディレクトリ名・依存設定・import・利用例・文書を新名称へ変更した。

ルートに MIT の LICENSE を配置し、workspace と全パッケージのライセンス・README・repository メタデータを整合させた。

Git の案内先は現在の origin に合わせて `https://github.com/nattyo1226/syntaqs` としている。

## 名称変更の検証

2026-11-02、direnv の devShell（Rust 1.99.1）で以下を実行し、成功した。

```sh
direnv exec . cargo test --workspace --all-features --offline --quiet
direnv exec . cargo check --workspace --all-targets --no-default-features --offline --quiet
direnv exec . cargo fmt --all -- --check
direnv exec . cargo clippy --workspace --all-targets --all-features --offline --quiet -- -D warnings
direnv exec . env RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --offline --quiet
direnv exec . cargo run -p examples --no-default-features --bin two_spins --offline --quiet
```

全 feature のテストには doctest を含む。Cargo メタデータ上の10パッケージと依存名、旧名称と以前の名前に由来する発音表記がソース・設定・文書に残っていないこと、ルートの Cargo.lock に旧パッケージ名が残っていないこと、文書内のローカルリンク240件の参照先を確認した。

## README・ライセンス整備の検証

2026-11-02、以下を確認した。今回の変更は文書と Cargo メタデータであり、製品コードは変更していない。

- トップと8つのライブラリ crate の README を作成し、ローカルリンクの参照先を確認した。
- トップ README の Cargo 設定と Rust コードをそのまま一時アプリへ抽出した。Git URL をプロセス内の Git 設定で手元のリポジトリへ対応付け、依存解決と実行（出力 `1: 1+0i`）を確認した。registry 依存は `cargo vendor --offline --locked` で既存キャッシュから一時ディレクトリへ用意した。
- 同じアプリに README の normalize・ndarray・sprs 設定を追加し、`cargo check --offline` が成功した。
- Cargo メタデータ上の全10パッケージについて、README・MIT・LICENSE の参照先・repository URL を確認した。
- 各ライブラリの `cargo package --list --allow-dirty --offline` で README.md と LICENSE の同梱を確認した。公開操作は実施していない。
- `cargo fmt --all -- --check` と、警告をエラーにした workspace 全 feature のドキュメント生成が成功した。

確認時点の GitHub 既定ブランチは改名前のコードだったため、新 crate 名による実リモートからの導入は未検証。GitHub 上への改名済みコードの反映後に確認する。リポジトリと origin はすでに syntaqs であることを確認済み。
