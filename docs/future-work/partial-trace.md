# Partial trace の検討事項

更新日: 2026-11-02

未実装の機能に関する数学的契約を記録する。実施時期・公開 API は未定であり、現在の full trace・基底処理・export はこの機能に依存しない。

## 1. 入力・残存空間・評価の契約

partial trace の計算時には、次の二つを確定する。

- `ambient`: 明示した空間があればそれを優先し、なければ入力演算子の束縛先を用いる全 Hilbert 空間
- `trace_out`: trace out する空間。空間の基本因子から縮約対象の自由度集合を取得する

`trace_out` は、[空間の部分系条件](../design/hilbert-space.md#6-部分系の判定)で `ambient` の部分系でなければならない。構築時に指定した内部自由度の空間を利用でき、site 全体の指定はその構成因子の空間に対応づける。演算子式が参照するすべての自由度も `ambient` に存在しなければならない。未束縛式でも空間引数を与えれば処理でき、先に別の束縛操作を呼ぶ必要はない。束縛も明示空間もなければエラーとし、縮約対象から ambient を推論しない。束縛済みでも `trace_out` は別途指定する。

結果は残存空間 `R` に束縛された演算子 `B: R→R` とし、残存因子の Index・局所構造・順序を保存する。結果が zero または scalar identity でも `R→R` を保持する。呼び出し側に結果の再埋め込みを要求しない。式には今回選択した縮約前の `ambient` と縮約対象を記録した遅延節点を保持できる。結果の束縛先を後から上書きしても、既存節点の縮約前空間は変更しない。

演算子が trace out する自由度上で恒等的に作用する場合、その局所空間の次元を係数へ掛ける。特に、

```math
\operatorname{Tr}_{\mathcal H_1}
\left(A_0;\mathcal H_0\otimes\mathcal H_1\right)
= \dim(\mathcal H_1)A_0
```

とする。空集合を trace out する場合は元の演算子と等価な式を返す。すべての自由度を trace out する場合、partial trace の結果は 0-site Hilbert 空間上の scalar operator expression とする。full trace は同じ数学的結果を複素 scalar として返す。

Fermion を含むすべての系で、full trace と partial trace は通常の Hilbert 空間上の非正規化 trace とする。partial trace は、入力の埋め込みで確定した因子順序と通常 tensor 表現に対して定義する。残存因子の基底 index を `r,r'`、trace-out 因子の完全正規直交基底 index を `t` とすると、結果の行列要素は `B_{r,r'} = Σ_t A_{r,t;r',t}` である。ここで複合 index は元の因子順序に従って配置し、この定義のために行列を生成する必要はない。

Fermion の場合は入力の ordering と grading を参照し、parity string を含む実際の作用に従って処理する。単に trace-out した自由度のラベルを消して元の式を再埋め込みしてはならない。supertrace は現時点では扱わず、graded な縮約によって通常の trace の意味を変更しない。

構文上の参照自由度を、通常 tensor 表現での作用因子と同一視しない。例えば二 mode の `c_1=P_0⊗a_1` について、通常の行列上の partial trace は

```math
\operatorname{Tr}_0(c_1)=\operatorname{Tr}(P_0)a_1=0
```

となる。mode 0 を identity spectator として `2a_1` を返すのは誤りであり、結果は残存空間 `H_1→H_1` の zero operator となる。これは通常の partial trace の規約に従う。

Spinful fermion で up mode だけを partial trace する場合は内部 mode の参照を、site 全体を指定する場合は両因子を用いる。

Partial trace は線型性と随伴を保存し、残存空間でさらに full trace を取ると元の full trace に一致する。互いに重ならない因子集合の逐次縮約は、同じ対象をまとめた縮約と一致する。これらと埋め込みの整合性も、実装時の検証対象とする。

## 2. 検証対象と未決事項

公開 API、遅延節点、残存空間の構築、共通評価器への接続は未実装である。意味の定義には `B[r,r′]=Σ_t A[r,t;r′,t]` を用い、複合 index は元の因子順序で配置する。全行列の生成を実装上の必須手順にはしない。

実装時には、空集合・真部分集合・全因子の縮約、線型性・随伴・full trace・互いに重ならない集合の逐次縮約、spectator の次元、zero・scalar 結果の空間保持を検証する。Spinful fermion の内部 mode と Site 全体の指定、graded な式と独立な通常行列の一致、不正な参照・部分系も対象とする。
