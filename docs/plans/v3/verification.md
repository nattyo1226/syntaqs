# v3 公開準備で参照する検証の経過

更新日: 2026-11-02

進行中の v3 公開準備で判断根拠とする、先行実装と v3 調整時の検証記録をまとめる。記載された日付・toolchain・テスト件数・包括 crate などの構成は当時のものであり、現在の構成や今回の文書再編で再実行した結果を表さない。

現在の実行方法・CI・未実施事項は[検証の構成](../../design/verification.md)、提供する契約は[機能別設計](../../README.md#2-機能別の詳細)を参照する。

## 1. 数値 export の先行検証（2026-10-27）

2026-10-27 の受入確認では、依存取得後に以下を実行し、すべて成功した。

- `cargo test --workspace --all-features --offline -j 2 --quiet`（doc test を含む151テスト）と `cargo test --workspace --no-default-features --offline -j 2 --quiet`（146テスト）。
- `cargo test -p syntaqs-export --no-default-features --features ndarray --offline -j 2 --quiet`（15テスト）と、同じ指定の sprs 構成（16テスト）。
- 当時の包括 crate での、正規化を無効にして export を有効にした構成（13テスト）。対応する縦断テストは現在 `examples` パッケージにある。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --offline -j 2 -- -D warnings`、`RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --offline -j 2`、および syntaqs-export の default feature 無効でのドキュメント生成。
- `cargo check -p syntaqs-core --no-default-features --offline -j 2` と heisenberg の `--all-features`、`--no-default-features --features export`、`--no-default-features` の各構成での実行。

6-site open chain の代数・密・疎の期待値はいずれも 1.25 であり、CSC 行列の shape は `(64, 64)`、非零成分は224だった。現在の MSRV 検証は `main` への push・PR 時に CI で実行する。専用 no_std target・性能ベンチマークの未実施事項は[今後の課題](../../future-work.md#3-未実施の検証と性能)に記録する。

## 2. Ordinary / graded の受入確認（2026-10-29）

Ordinary / graded の共通機構の実装時に行った受入確認を記録する。Fermion / spin-one の製品実装の検証は、それぞれ[局所代数の検証記録](../../design/local-structure.md#86-検証対象とテスト)・[Spin-one の検証記録](../../design/local-structure.md#96-検証対象とテスト)を参照する。共通機構の実装では新しい本番依存は追加していない。

| 対象 | 実装時の検証根拠 |
| --- | --- |
| kind・同一視・混在拒否・部分系・構築入口・fallible 優先順位 | [tests/tests/core_tensor_products.rs](../../../tests/tests/core_tensor_products.rs)、既存の spaces / fallible_construction / systems テスト |
| 符号・CAR・作用・随伴・内積・trace・Hilbert–Schmidt 内積 | テスト専用二状態代数と独立 Kronecker 行列による二・三因子の照合（f32 / f64、ordinary / graded） |
| Act scope・local embedding の準同型性・三つの odd 成分 | 同テストの scope / local expression / three odd factor ケース |
| intrinsic 座標と通常行列・再展開・列挙・空の基底 | 同テストと [tests/tests/core_basis_expansion.rs](../../../tests/tests/core_basis_expansion.rs)。非正規直交な三状態基底も両種類で検証 |
| 密疎 export と metadata | [tests/tests/export_tensor_products.rs](../../../tests/tests/export_tensor_products.rs) の Vec / ndarray / sprs、両精度の行列・遅延作用 vector |
| 正規化・期待値 | [tests/tests/normalize_tensor_products.rs](../../../tests/tests/normalize_tensor_products.rs) の独立行列との照合（両精度） |
| 最小能力・借用・共有・疎性 | 既存 capability / contract テストの移行、Product / 係数演算なしの graded 作用、80因子の分離状態と局所 trace、32因子の割り当て上限 |
| 深い木 | 既存 ordinary 回帰と graded の2万段 wrapper の構築・比較・評価・破棄（128 KiB stack） |
| spin-half・利用例 | 全 primitive が even、P=I、両種類での作用と trace の一致。[tensor_products.rs](../../../examples/src/bin/tensor_products.rs) を実行 |

非自明な grading の参照代数は[テスト専用二状態代数](../../../tests/src/graded.rs)であり、製品用 fermion API ではない。`|0>` even、`|1>` odd、`a=|0><1|`、`P=diag(1,-1)` とし、二 mode の基底順は `|00>,|01>,|10>,|11>` とする。独立した Kronecker 行列で `c_1|11>=-|10>`、`c_0 c_1|11>=-|00>`、`c_1 c_0|11>=|00>`、局所・異なる mode の CAR、`Tr(c_1†c_1)=2` を確認した。参照行列は評価器・export を使わずに構築し、intrinsic 座標と通常行列を別々に照合した。近似的な零成分の削除は行わない。

Rust 1.98.0 の direnv devShell で次を実行し、成功した。

- `cargo test --workspace --all-features`（doctest を含む）
- `cargo check -p syntaqs-core -p syntaqs-geometry --no-default-features`
- `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo run -p examples --bin tensor_products`

最終調整した構築入口と export の contract テストも再実行した。no_std の確認はホスト上のビルドであり、専用 target の検証ではない。MSRV `1.85.0` でのビルド・テストは、現在は `main` への push・PR 時に [CI workflow](../../../.github/workflows/ci.yml) で検証する。専用 no_std target・性能ベンチマークは引き続き [未実施事項](../../future-work.md#3-未実施の検証と性能)として残す。

## 3. 実装・移設時の検証記録

2026-10-31、Rust 1.99.1 の direnv devShell で geometry・relation の実装と受入確認を完了した。workspace 全 feature・default feature 無効のテスト、最終状態の geometry テスト13件と doc test、多体項・Pauli string・graded 積の数値比較3件、format、全 target の Clippy、警告をエラーとする rustdoc の生成が成功した。全 feature の workspace 一括実行後に追加した境界値テストは、geometry 単独と default feature 無効の workspace 全体でも確認した。

examples への移設時には両サンプルを実行し、feature 無効時のテスト11件・全 feature 有効時のテスト13件、format、両 feature 構成の全 target の Clippy が成功した。Pauli の行列比較を含む2件は `export` feature を有効にする。これらは実装・移設時の記録であり、今回の文書整理でテストを再実行したものではない。

```sh
direnv exec . cargo test -p syntaqs-geometry --all-features --offline
direnv exec . cargo test -p examples --all-features --offline --test geometry_relations
direnv exec . cargo test -p examples --all-features --offline --bin geometry_relations --bin pauli_strings
direnv exec . cargo test --workspace --all-features --offline -j2
direnv exec . cargo test --workspace --no-default-features --offline -j2
direnv exec . cargo fmt --all --check
direnv exec . cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
direnv exec . sh -c 'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --offline'
```

MSRV `1.85.0` でのビルド・テストは、現在は `main` への push・PR 時に [CI workflow](../../../.github/workflows/ci.yml) で検証する。実装・移設時の専用 no_std target の試行は依存解決とネットワーク制限で完了しておらず、成功した検証には数えない。性能測定・キャッシュと再生成の比較、周期正方格子の正式実装も未実施である。共通の未実施事項は[今後の課題](../../future-work.md#3-未実施の検証と性能)にまとめる。

## 4. 規格化変更の検証記録

2026-11-01、Rust 1.99.1 の direnv devShell で実装と受入検証を行った。本節は実装時の記録であり、今回の文書整理でテストを再実行したものではない。以下のコマンドはリポジトリ直下で `direnv exec .` を付けて実行し、すべて成功した。

```sh
cargo test --workspace --all-features --no-fail-fast --quiet
cargo test --workspace --no-default-features --offline --no-fail-fast --quiet
cargo test -p syntaqs-spin-one -p syntaqs-fermion --all-features --offline --quiet
cargo test -p syntaqs-spin-one --test tensor_products -p syntaqs-fermion --test models --offline --quiet
cargo check -p syntaqs-core -p syntaqs-geometry --no-default-features --offline
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --offline -- -D warnings
sh -c 'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --offline'
```

syntaqs-export は単独でも `cargo test -p syntaqs-export --no-default-features --offline --quiet` を実行し、追加 feature なし・`--features ndarray`・`--features sprs` の3構成を確認した。両 backend 有効の構成は workspace の全 feature テストで確認した。

確認した内容は次のとおり。

- 各代数の全 primitive の積・作用・随伴・通常 trace、両演算子基底の変換と座標を、独立行列と f32 / f64 で照合した。spin-one の全225積・45作用、結合則・作用との整合性も検証した。
- 局所の正規化 HS Gram 行列に加え、spin-one の Cartesian / Ladder tensor 基底と fermion の graded Majorana tensor 基底の Gram 行列を検証した。graded 評価の符号・CAR と通常作用行列も照合した。
- [HS 内積の回帰テスト](../../../tests/tests/core_hs_inner_product.rs)で、明示行列による `Tr(A†B)/D`、空空間、spectator の追加、明示空間による束縛の上書き、および2048因子での `⟨I,I⟩=1` を確認した。状態型・Dimension・整数変換を要求しないこと、Divide だけを持つ係数でも計算できること、通常 trace に除算能力を要求しないことも検証した。
- 物理的なモデルの独立参照行列とエネルギー・定数シフトを維持した。微小係数の保存と厳密な相殺、解析的に零の局所規則の空性も確認した。暗黙の近似的な切り捨ては追加していない。

examples の全8バイナリを `cargo run -p examples --bin <name> --offline --quiet` により、既定 feature・`--all-features`・`--no-default-features` の3構成で実行した。一重項・三重項のエネルギー −0.75 / 0.25、symmetric Hubbard のエネルギー −2、spin-one BLBQ の期待値0.5、Pauli string の37項と密疎 export の一致が保たれた。

依存関係と feature 要件は変更していない。MSRV `1.85.0` でのビルド・テストは、現在は `main` への push・PR 時に [CI workflow](../../../.github/workflows/ci.yml) で検証する。専用 no_std target での実ビルドは、[未実施の検証](../../future-work.md#3-未実施の検証と性能)として引き続き扱う。

## 5. 式の表示の検証（2026-11-01）

2026-11-01、Rust 1.99.1 / aarch64-apple-darwin、direnv の devShell で確認した。関連5 crate のテスト・doc test と、以下の workspace 検証は成功した。

```sh
direnv exec . cargo test --workspace --all-features --offline --quiet
direnv exec . cargo check --workspace --all-targets --no-default-features --offline
direnv exec . cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
direnv exec . cargo fmt --all -- --check
direnv exec . cargo doc --workspace --all-features --no-deps --offline
direnv exec . cargo run -p examples --no-default-features --bin expression_formatting --offline
```

書式・型制約・エラー・深さの回帰テストは [core の formatting.rs](../../../crates/syntaqs-core/tests/formatting.rs)、Tag と Id128 は [basic の indices.rs](../../../crates/syntaqs-basic/tests/indices.rs)、primitive 全 variant は各 builtin crate の `tests/formatting.rs` で確認する。深さ20,000の局所／複合の作用・和・係数倍・積・随伴と混合 tensor を128 KiBのスタックで表示・破棄できた。内部テストでは深い単純 tensor の ket 化と共有部分式の分類も確認した。最後に追加した深さ・Debug・分類のテストは対象を絞って再実行した。

上記の受入確認では、no_std 対応はホスト上の確認であり、専用 target でのビルドは未実施（環境にはホスト target のライブラリのみ導入済み）。MSRV `1.85.0` でのビルド・テストは、現在は `main` への push・PR 時に [CI workflow](../../../.github/workflows/ci.yml) で検証する。専用 no_std target・性能ベンチマーク・速度比較の未実施事項は[今後の課題](../../future-work.md#3-未実施の検証と性能)を参照する。
