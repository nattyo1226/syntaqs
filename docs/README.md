# ドキュメントの案内

更新日: 2026-11-02

## 1. 文書の責任

| 文書 | 記述する内容 |
| --- | --- |
| [requirements](requirements.md) | 目的・責任範囲・機能要件など、何を作っているか。 |
| [architecture](architecture.md) | 全体構造と責任の配置など、何を提供しているかの概要。 |
| [future-work](future-work.md) | 今後実装する機能と未採用の候補。詳細は `future-work/` に置く。 |
| `design/` | 現在提供している各機能の API・数学的契約・制約・実装方式。 |
| [plans/v3](plans/v3/README.md) | 現在進行中の v3 公開準備の計画と経過。 |

現行仕様の詳細は対応する設計文書に集約し、要件と概要から参照する。未実装機能の検討内容は現行仕様と分け、実装を開始した段階で `plans/` に計画を置く。進行中の計画で完了した作業はその経過として記録し、提供する仕様は `design/` に反映する。

## 2. 機能別の詳細

| 機能 | 文書 |
| --- | --- |
| 局所代数・primitive・grading 能力 | [local-structure](design/local-structure.md) |
| 係数型・数値能力・零と近似比較 | [numerics](design/numerics.md) |
| 因子の識別と標準 Index | [index](design/index.md) |
| 空間・因子順序・grouping・互換性 | [hilbert-space](design/hilbert-space.md) |
| 型選択・式の構築・fallible 集約 | [expression-construction](design/expression-construction.md) |
| 局所式とその評価 | [primitive-expression](design/primitive-expression.md) |
| 複合式・配置・束縛・ordinary / graded 評価 | [composite-expression](design/composite-expression.md) |
| ノルム・正規化・正規化期待値 | [normalization](design/normalization.md) |
| 基底展開・座標・基底列挙 | [basis](design/basis.md) |
| 式の表示 | [expression-formatting](design/expression-formatting.md) |
| 幾何・関係・Site への接続 | [geometry](design/geometry.md) |
| Spin・Hubbard モデルの構築 | [models](design/models.md) |
| 数値コンテナへの出力 | [export](design/export.md) |
| 検証の構成と実行方法 | [verification](design/verification.md) |
