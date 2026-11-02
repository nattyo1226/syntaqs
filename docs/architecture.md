# syntaqs アーキテクチャ概要

更新日: 2026-11-02

## 1. 全体構造

syntaqs は、局所代数の規則を共通の式・空間・評価機構へ接続し、幾何に基づくモデル構築と数値出力をその上に組み合わせる構成である。本書は現在提供する全体構造を示す。目的と機能要件は[requirements](requirements.md)、各機能の契約は[design の案内](README.md#2-機能別の詳細)を参照する。

## 2. Crate と責任

workspace は8つのライブラリと、実行例・横断テスト用の2パッケージで構成する。

| Crate / パッケージ | 提供するもの |
| --- | --- |
| `syntaqs-core` | 局所代数の能力 trait、係数能力、有限線型結合、局所・複合式、Hilbert 空間、構築入口、評価、基底・座標。 |
| `syntaqs-basic` | UUID と文字列による標準 Index。 |
| `syntaqs-geometry` | Chain・境界条件、Site 関係の列挙、外部データ・空間列への借用接続。 |
| `syntaqs-normalize` | ノルム、状態正規化、正規化期待値の拡張 trait。 |
| `syntaqs-export` | Vec・ndarray・sprs への出力と解釈に必要なメタデータ。 |
| `syntaqs-spin-half` | Spin-1/2 の primitive・基底と XYZ・Heisenberg・Neel の構築。 |
| `syntaqs-spin-one` | Spin-1 の primitive・基底と XYZ・Heisenberg・BLBQ の構築。 |
| `syntaqs-fermion` | 単一 mode fermion の primitive・基底と標準・symmetric Hubbard の構築。 |
| [examples](../examples/README.md) | 各 crate を直接利用する実行可能な利用例。 |
| [syntaqs-tests](../tests/README.md) | 独自代数の共有 fixture と公開 API・横断テスト。 |

## 3. 依存方向

core は具体的代数・basic・geometry・normalize・export に依存しない。局所規則・基底は公開 trait で接続し、具体的代数ごとの式木や evaluator は設けない。

geometry・normalize・export と3つの具体的代数は core に依存する。各代数のモデル構築は geometry を利用する。basic の core への依存はテスト・doctest に限られ、UUID と乱数源は basic に閉じる。本番ライブラリは examples・syntaqs-tests に依存しない。

core・geometry・spin-half・fermion は `no_std + alloc`、basic・normalize・export・spin-one は std を利用する。export の既定 feature は空で、Vec 出力を常時提供し、ndarray・sprs を独立に有効化する。

## 4. 主なデータと処理

| 対象 | 役割と関係 |
| --- | --- |
| Primitive と係数型 | 状態・演算子の意味と局所規則を型と個別の能力実装で定める。必要な数値能力は操作ごとに要求する。 |
| `HilbertSpace<I>` | 因子の同一性・順序・grouping と ordinary / graded の構成を保持する。局所代数・次元・基底は保持しない。 |
| 局所式・複合式 | 不変な節点を共有し、和・積・作用・随伴などを遅延保持する。状態は所属空間、演算子は任意の全作用空間の束縛を持つ。 |
| `ExpressionFactory` / `QuantumSystem` | 構築入口と primitive・係数の型情報を分離する。`SpinHalf<C>` などは factory の型エイリアスである。 |
| 評価 | 選択した空間と局所規則を使う。ordinary / graded は共通 API で扱い、core が因子順序と parity に基づく符号を処理する。 |
| `Basis` / `Coordinates` | 基底を操作時に選び、通常の式への展開・疎座標・遅延列挙を提供する。式の節点に基底情報を加えない。 |
| `Exported` | 数値データと基底・空間・shape・格納順序を対応付ける。作用行列は状態基底への作用から列を生成する。 |
| `Chain` / `SiteMapper` | 幾何と外部の Site 順データ列を借用接続する。Index 発行・空間の構築保持・全体順序は利用側が担う。 |

典型的には、利用側が Index と空間を構築し、型と係数を選んで局所式を配置・合成する。状態への作用・内積・期待値・trace は行列を生成せずに評価でき、座標や数値行列が必要な時点で基底を選ぶ。モデル補助も一般の状態・演算子式を返す。

## 5. モジュール構成

core は `ops`・`coefficient`・`space`・`system`・`expr`・`basis` と非公開の `evaluate` に分かれる。共通の構文パーツは `expr::parts`、内部走査は `expr::traversal` が担う。公開の式ハンドルと主要 trait は crate root から利用でき、節点 enum と共有表現は非公開である。

geometry は幾何・境界・関係・Group・接続、normalize は norm・期待値・エラー、export は引数・結果・メタデータ・座標／ベクトル／行列の変換・backend に分かれる。具体的代数は primitive・係数 helper・基底・models を分離する。詳細な型・能力・エラー・処理方式は各設計文書に定める。

検証の構成は[verification](design/verification.md)、未実装の拡張は[future-work](future-work.md)、公開準備の進行は[v3 計画](plans/v3/README.md)を参照する。
