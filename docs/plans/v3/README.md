# v3 に向けた実装案

作成日: 2026-10-30
更新日: 2026-11-02

v3 への更新の対象は以下の項目とする。未実装の項目は本書とリンク先の修正案で検討し、実装済みの項目は現行の設計文書・使用例を参照する。

## 演算子 primitive の名前と規格化

**名前・規格化・昇降演算子・HS 内積の変更を実装・検証済み（2026-11-01）。**

primitive の一覧・規格化は[局所 primitive の設計](../../design/local-structure.md#21-提供する-primitive-と規格化)、HS 内積の定義は[次元規格化の契約](../../design/primitive-expression.md#42-hilbertschmidt-内積の次元規格化)、モデルの物理的な結合定数は[モデル構築の設計](../../design/geometry.md)にまとめる。

数値的な相殺の扱いは[数値比較の要件](../../requirements.md#56-数値的近似等価性と検証の規約)、実装時の受入確認は[規格化変更の検証記録](../../design/local-structure.md#11-規格化変更の検証記録)を参照する。

## 式木表示のカスタムフォーマット

**実装・検証済み（2026-11-01）。** Display・複数行表示・Debug、Index の8文字制限、Tag と builtin primitive の表示を提供する。契約と検証結果は[式木の表示](../../design/expression-formatting.md)、利用例は [expression_formatting](../../../examples/src/bin/expression_formatting.rs)を参照する。

## プロジェクトルートの README とライセンスファイル

**整備・検証済み（2026-11-02）。** プロジェクトの説明・導入方法は[トップの README](../../../README.md)、各 crate の担当責任は[crate 一覧](../../../README.md#crates)から各 README を参照する。ライセンス本文はルートの [LICENSE](../../../LICENSE) に配置した。

- `README.md` には、プロジェクトの目的・提供機能、crate の構成、依存設定と feature、最小の利用例、examples・設計文書へのリンク、開発時のビルド・検証方法をまとめる。
- 実装済みの機能と将来の計画を区別し、利用例やコマンドは現行の公開 API・リポジトリ構成と一致させる。
- 採用ライセンスは MIT、著作権表示は `Copyright (c) 2026 Haruki SAITO`、適用範囲はリポジトリ全体とする。
- README の案内と workspace・各 crate のライセンス・README・repository メタデータを整合させる。

README 内のローカルリンク・利用例、ドキュメント生成とフォーマット確認のコマンド、ライセンスファイル・Cargo メタデータの対応を確認した。利用例の Git 依存は手元のリポジトリへ対応付けて検証しており、改名済みコードの GitHub 反映後の導入確認は未実施である。

## GitHub Actions の workflow 整備

**整備・GitHub Actions 上でのテスト実行確認済み（2026-11-02）。** 定義は [CI workflow](../../../.github/workflows/ci.yml) を参照する。

- `main` への push、pull request、手動実行を契機とし、Linux の64 bit 環境で検証する。
- MSRV `1.85.0` と最新 `stable` のそれぞれで、workspace 全体の既定 feature・全 feature 有効・既定 feature 無効の3構成をテストする。doc test と `compile_fail` の検証も含む。
- 最新 `stable` で `syntaqs-export` と `syntaqs-tests` を個別にテストし、backend なし・`ndarray` のみ・`sprs` のみを確認する。両 backend 有効の構成は workspace の全 feature テストで確認する。
- format・Clippy と、警告をエラーにする API ドキュメント生成を別ジョブで実行する。依存キャッシュと、同じ PR・ブランチの古い実行のキャンセルを設定する。

利用者が push 対象に `feat/workflow` を一時的に追加し、GitHub Actions 上でテストが正常に実行されることを確認した。確認後の push 対象は `main` のみとする。

`no_std + alloc` 対応 crate の専用 target でのビルドは今回の workflow に含めておらず、引き続き未実施である。

## 3体以上の相互作用に対応する関係の列挙

**実装・受入確認済み（2026-10-31）**。

幾何に基づく多体の Site 配置を列挙し、外部の HilbertSpace 列へ借用接続する。

API 契約と検証記録は[Chain・Site と自由度の配置](../../design/geometry.md)、使い方は [examples の README](../../../examples/README.md)にまとめる。

## ドキュメントの最終調整

**未着手。** 文書間の責任分離、`docs/` 以下の英語化、およびルート README の crate 一覧と Cargo.toml の依存一覧の順序を見直す。修正項目は[一時ドキュメント](documentation.md)に記録する。
