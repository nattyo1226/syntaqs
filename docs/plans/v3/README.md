# v3 に向けた実装案

作成日: 2026-10-30
更新日: 2026-11-01

v3 への更新の対象は以下の5項目とする。未実装の項目は本書とリンク先の修正案で検討し、実装済みの項目は現行の設計文書・使用例を参照する。

## 演算子 primitive の名前と規格化

**名前・規格化・昇降演算子・HS 内積の変更を実装・検証済み（2026-11-01）。**

primitive の一覧・規格化は[局所 primitive の設計](../../design/local-structure.md#21-提供する-primitive-と規格化)、HS 内積の定義は[次元規格化の契約](../../design/primitive-expression.md#42-hilbertschmidt-内積の次元規格化)、モデルの物理的な結合定数は[モデル構築の設計](../../design/geometry.md)にまとめる。

数値的な相殺の扱いは[数値比較の要件](../../requirements.md#56-数値的近似等価性と検証の規約)、実装時の受入確認は[規格化変更の検証記録](../../design/local-structure.md#11-規格化変更の検証記録)を参照する。

## 式木表示のカスタムフォーマット

**実装・検証済み（2026-11-01）。** Display・複数行表示・Debug、Index の8文字制限、Tag と builtin primitive の表示を提供する。契約と検証結果は[式木の表示](../../design/expression-formatting.md)、利用例は [expression_formatting](../../../examples/src/bin/expression_formatting.rs)、一時文書は[実装記録](expression-formatting-draft.md)を参照する。

## プロジェクトルートの README とライセンスファイル

プロジェクトルートに `README.md` とライセンスファイルを整備する。

- `README.md` には、プロジェクトの目的・提供機能、crate の構成、依存設定と feature、最小の利用例、examples・設計文書へのリンク、開発時のビルド・検証方法をまとめる。
- 実装済みの機能と将来の計画を区別し、利用例やコマンドは現行の公開 API・リポジトリ構成と一致させる。
- 採用ライセンス、著作権者表記、適用範囲を確定したうえで、対応するライセンスファイルを配置する。採用ライセンスは現時点では未定とする。
- ライセンス確定後、README の案内と workspace・各 crate の Cargo メタデータを整合させる。

検証では、README 内のリンク・利用例・コマンドと、ライセンスファイル・Cargo メタデータの対応を確認する。

## GitHub Actions の workflow 整備

`.github/workflows/` に継続的な検証のための workflow を整備する。現時点では同ディレクトリは存在しない。

- pull request・push 時に、format、Clippy、ビルド・テスト（doc test を含む）、ドキュメント生成を実行する。
- default feature 無効・全 feature 有効に加え、`qmb-export` の backend ごとの feature 構成を確認する。
- MSRV でのビルド・テストと、`no_std + alloc` 対応 crate の専用 target でのビルドを検討し、[未実施の検証](../../future-work.md#3-未実施の検証と性能)を補う。
- 対象ブランチ・OS・Rust toolchain、ジョブ分割、依存キャッシュ、実行時間を整理し、ローカルの検証手順と README の案内を一致させる。

具体的な workflow 構成は未確定とする。導入時には GitHub Actions 上での実行結果を確認し、実施できた検証と残る制約を記録する。

## 3体以上の相互作用に対応する関係の列挙

**実装・受入確認済み（2026-10-31）**。

幾何に基づく多体の Site 配置を列挙し、外部の HilbertSpace 列へ借用接続する。

API 契約と検証記録は[Chain・Site と自由度の配置](../../design/geometry.md)、使い方は [examples の README](../../../examples/README.md)にまとめる。
