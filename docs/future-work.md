# 今後の課題

更新日: 2026-11-02

今後実装する機能、必須の設計見直し、未採用の候補をまとめる。現在提供している仕様は[architecture](architecture.md)と[design](README.md#2-機能別の詳細)、現在進行中の公開準備は[v3 計画](plans/v3/README.md)に記述する。

## 1. 必須の設計見直し

State・Operator・Structure・Space の所有・共有・関連付け、および Geometry と空間の境界を再検討する。Operator と State を一般の演算子の特別な形として整理できるかを含め、数学的対象と API、共通化する範囲、別に維持する範囲を理由とともに決める。論点と完了条件は[設計見直しの検討事項](future-work/state-and-operator.md)にまとめる。

## 2. 未実装の機能と検討事項

以下は未着手の機能または候補であり、公開 API・実施時期・優先順位は未定である。実装開始時に `plans/` へ具体的な計画を置き、提供する仕様は実装に合わせて `design/` へ反映する。

| 対象 | 今後扱う内容 |
| --- | --- |
| [Partial trace](future-work/partial-trace.md) | 全作用空間と縮約対象の指定、遅延節点、残存空間、graded な通常作用の縮約、評価器と検証。 |
| [写像の表現係数](future-work/representation-matrix.md) | 汎用的な線型・共役線型・複数入力の写像、異なる入出力基底、係数データの保持と再利用。 |
| [API の使いやすさ](future-work/api-usability.md) | 構築補助・因子検索・型推論・診断・利用開始と独自代数の案内。 |
| [保存・復元](future-work/identity-and-storage.md) | Index を保つ復元と、新しい Index に一貫して付け直す独立した系の作成、保存形式。 |
| [外部接続](future-work/external-connections.md) | 自動微分・ML module・tensor network の規約とパラメータ依存性の保持。 |

### 2.1. 局所代数と基底の拡張

一つの系への異種局所代数の混在・合成、そのための型消去・保持方式、実行時の表現パラメータ、異なる実装型間の定義照合を検討する。因子ごとに異なる基底、部分空間基底・射影・制約付き部分空間、保存量・粒子数 sector は後続とする。

Spin-1/2・spin-1・単一 mode fermion とそのモデルは提供済みである。追加候補は一般 spin-S・qudit・boson と truncation、spin-one の MatrixUnitBasis・球面テンソル位相を持つ基底・Float を要求しない厳密な Product / Act、fermion の汎用 spinful 構築補助・任意格子・可変結合である。

### 2.2. 式と空間の操作

Bra 型・構築 API、一般の U→V の写像、内部ラベル、任意の因子集合からの部分系取得、Regrouping・permutation、実質的に異なる ordinary / graded の混在を検討する。

積和から和積への因数分解、個別の簡約規則を選ぶ操作、一般の式変形・高度な簡約戦略、任意の式型の分解・再構成とユーザ独自の式木への接続も候補とする。通常の evaluate・基底展開に必要な分配・局所積・係数処理は既に提供する機構であり、これらの追加操作とは区別する。

Source の自動保持、式の構造比較、一般的な作用の等価性判定は未実装である。独立に構築した同形の式を自動共有する機構、評価結果のキャッシュ、並列評価・スレッド間共有は導入可否も含めて検討する。キャッシュでは対象・再利用条件・保持先・破棄・資源管理を、並列化では保持型の条件と実行方式を決める。

近似的な成分切り捨ては、基底・許容誤差・適用時点を利用者が指定する明示的な後処理として検討する。次元規格化した HS 正規直交基底では、捨てる成分 `δA=Σμ δaμ Bμ` の誤差は `||δA||²=Σμ |δaμ|²` で評価できる。非正規直交基底にはこの式をそのまま適用しない。現在の評価・座標・export に暗黙の切り捨ては追加しない。

### 2.3. 幾何・数値 backend・追加の量子計算機能

- 一次元 Chain 以外の標準 Geometry、任意 graph・座標・距離・plaquette・hyperedge。正方格子は現在 examples 内の実装に限る。
- 疎行列の出力時の CSR / CSC 選択、異なる index 整数型、追加 backend。
- Density operator・mixed state・reduced density operator、熱状態、固有値問題・基底状態探索、時間発展、測定・確率 sampling。
- Parameterized operator・記号係数と大規模な記号計算。
- MPS・MPO への変換、MPO の生成・最小化、tensor network の truncation・gauge・canonical form。
- GPU・分散計算・大規模計算向け backend。

後続機能のためだけの crate は先に作らず、独立した責任・依存関係・実行基盤が具体化した段階で検討する。Supertrace は通常の trace と異なる操作であり、現在の提供範囲には含めない。

## 3. 未実施の検証と性能

専用 no_std target での実ビルド、性能ベンチマーク・速度比較・追加の割り当て最適化は未実施である。現在の検証構成と限界は[verification](design/verification.md)を参照する。

演算子積に含まれる和の分配で、結果が疎でも項数が増大する場合がある。先に局所的な和をまとめる最適化、キャッシュと再生成の比較、区間 parity を未評価の構造として保持する方式は後続の検討対象とする。
