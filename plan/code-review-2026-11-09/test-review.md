# テスト妥当性レビュー

## Design Concerns

### [P2] Lie 分解の成功例が三次元単純イデアルに偏っている

対象：[decomposition.rs](../../crates/syntaqs-lie/tests/decomposition.rs#L58)、[共通 workflow](../../tests/tests/lie_analysis.rs#L288)。

分解テストの主要 fixture は Pauli の三要素を並べた `su(2)` の一〜三個の直和と中心であり、検証 helper も各単純成分の次元を 3 としている。graded の分解例も三要素からなる。共通テストの三準位例も A/B 間の x/y/z と恒等演算子だけで、八次元の `su(3)` を検証しているわけではない。

任意のコンパクト Hermitian Lie 代数を扱う公開契約に対し、高次元の単純成分を過分割せず保持すること、異なる次元の単純成分を分離することの独立な回帰検証が不足している。現行実装に誤動作を確認したという指摘ではない。

改善方向：三準位の行列単位 fixture を使い、完全な `su(3)` が八次元の単純成分一個になるケースを f32/f64 で追加する。次に `su(3) + su(2)` など異なる次元の直和を、独立に定めた成分 projector と比較する。すべての既存 fixture を増やすより、この二つを優先する。

### [P2] no_std 契約を専用ターゲットで検証していない

対象：[CI](../../.github/workflows/ci.yml)、[既知の検証限界](../../docs/design/verification.md#3-ci-and-verification-limits)。

core・geometry・spin-half・fermion は no_std を提供するが、CI のテストは Linux host 上に限られる。workspace には std を有効にする依存元も含まれ、host の全体テストだけでは、対象クレートが std のない依存構成でコンパイル可能という保証にならない。この不足は既存ドキュメントにも明記されており、新発見の実装不具合ではない。

改善方向：ポインタ幅 atomic を持つ、プロジェクトがサポートする no_std ターゲットを一つ選び、対象パッケージ単独の library check を CI に追加する。一般の組込み全機種を対象とした巨大な matrix や実機実行まで増やす必要はない。

### [P3] feature matrix 内で構成を変えない補助チェックを三重実行している

対象：[CI 50〜76 行](../../.github/workflows/ci.yml#L50)。

Rust 二種類 × workspace feature 三種類の各 job で、Lie-only example、Lie-only 共通テスト、下流 fixture、provider feature 確認を実行している。これらのコマンドは matrix の `configuration.args` を使わず、各 toolchain について同じ構成を三回実行する。先行 workspace テストによってビルドキャッシュは異なるが、別 Cargo コマンド間で選択 feature が引き継がれるわけではない。

構成別 workspace テスト自体は維持し、これらの補助チェックだけを toolchain ごとに一回へ集約する余地がある。各補助チェックの実行回数は六回から二回になる。CI 全体の時間短縮率は計測していないため断定しない。

## 過剰とは判断しなかった項目

- f32/f64 の両方：精度別の閾値と数値演算があるため必要。
- ローカル行列照合、代数法則、複合系の統合テスト：一部の演算は重なるが、局所規則と配置・符号・縮約という異なる障害を検出する。単純な重複として削除しない。
- crate 内の検証と共通テスト：独立した三準位・graded fixture を通すことで、組込み代数に固有の仮定を検出できる。
- export backend の個別実行：workspace の feature 統合による漏れを防ぐ役割がある。
- capability テストと compile_fail doctest：実行時の数値テストでは検出できない型・trait 契約を検証する。
- 深さ 20,000・小スタックのテスト、allocation 制限：深い式の反復処理や疎出力の契約に直接対応する。前回の実行では長時間を要するテストもあったが、それだけでケース数や深さが過剰とは判断しない。通常の単体テストと stress job の分離は運用上の選択肢。

## Verification

テストの期待値が実装の自己確認だけになっていないか、独立な行列参照、assertion、fixture の構成、精度、異常系、CI の呼出しを静的に確認した。主に以下を参照した。

- `tests/README.md`、`docs/design/verification.md`、`.github/workflows/ci.yml`、workspace と関係クレートの manifests。
- `tests/tests/lie_analysis.rs`、Lie の分解・数値 kernel テスト。
- 共有 three-level の比較 helper、fermion の独立参照 helper、spin-one の局所規則テスト。
- `core_allocation.rs`、`sparse_alloc.rs`、core tensor/basis テストと normalization テストの関連箇所。

既存のテストが広く網羅している正常系・境界条件を踏まえ、すべての組合せやランダムテストを追加すべきという評価はしていない。上記以外の全 assertion の網羅的監査、mutation testing、coverage 測定は行っていない。

前回レビューで `cargo test --workspace --all-features --quiet`（doctest 含む）、fmt、Clippy の成功を確認済み。今回は静的なテスト設計レビューでコード変更もないため、同じ suite の再実行はしていない。no_std の専用ターゲットや追加提案ケースの成功は未確認。ソース・テスト・CI は変更していない。

確認範囲では、誤った期待値による確定的なテスト不具合は特定できなかった。優先するのは高次元単純代数の追加と no_std の検証であり、削減するなら同一構成の CI 補助チェックから着手する。
