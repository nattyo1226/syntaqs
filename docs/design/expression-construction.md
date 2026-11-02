# 型選択と式の構築

更新日: 2026-11-02

共通の構築入口と係数入力の契約を定める。局所式の評価は[primitive-expression](primitive-expression.md)、空間を伴う式は[composite-expression](composite-expression.md)を参照する。

## 1. 型選択・配置・集約

QuantumSystem は関連型 `State`・`Operator`・`Coeff` で状態・演算子 primitive と係数型を関連付ける。構築は ExpressionFactory の以下の inherent method から既存の core 式を生成する。空間や Index を登録・保持せず、係数・primitive・Index の Clone や評価能力を一律に要求しない。複数式の合成は I: Eq を要求する。

`ExpressionFactory<S,O,C>` は値を保持せず、インスタンス生成を必要としない構築入口であり、QuantumSystem を一括実装する。`SpinHalf<C>`・`SpinOne<C>`・`Fermion<C>` は具体的な primitive を選ぶ型エイリアスで、構築に trait の import は不要である。独自の primitive も ExpressionFactory の型エイリアスから利用できる。同じ primitive と係数型からは同じ式型を構築し、システム名を式の同一性へ追加しない。

`LocalStateOf<Q>`・`LocalOperatorOf<Q>`・`StateOf<Q,I>`・`OperatorOf<Q,I>` は型注釈に使う。システム単位のジェネリック API では `Q: QuantumSystem` から型情報を取得し、構築には `LocalStateOf::<Q>::primitive(...)` などの式型の入口、または `ExpressionFactory<Q::State,Q::Operator,Q::Coeff>` を使う。QuantumSystem 自体に構築メソッドは置かない。モデル固有の構築は各 primitive crate の公開 `models` module に分離し、[モデルの設計](models.md)に従う。

| 入口 | 入力・契約 |
| --- | --- |
| state / operator | 状態／演算子 primitive を一つ受け取り、未配置の局所式を返す。係数の算術や評価能力は要求しない |
| product_state | 明示空間と状態 primitive の列。空間の葉の順に一つずつ配置し、個数の完全一致を検証する |
| operator_product_at | 葉と演算子 primitive の組の列。同一 Index の再登場を許可し、入力順に積を構築。右端が先に作用。空入力は未束縛 identity |
| state_sum / try_state_sum | 明示空間と状態列。空入力はその空間の zero |
| operator_sum / try_operator_sum | 演算子列。空入力は未束縛 zero |
| tensor_in / try_tensor_in | 明示空間と配置済み状態列。葉の連結が指定空間を順序も含めて完全に覆い、部分系の種類も互換であることを検証する |
| operator_product / try_operator_product | 配置済み演算子列の入力順の積。空入力は未束縛 identity |

operator_product_at の _at は配置も行う入口を意味し、裸の primitive の列ではなく (&factor, primitive) の IntoIterator を受け取る。配置先は葉のみで、Tensor の自動展開はしない。演算子は明示的に bind するまで未束縛であり、spectator を含む全作用空間を積の配置先から自動構築しない。入力スライスや配置先の変数の生存期間を越えて、共有した式・空間を保持できる。

try_ は Result 入力を表し、通常版も複合式では Result を返す。エラー型は ExpressionError に固定する。最初の入力 Err をそのまま返し、その後は Iterator の next を呼ばない。全入力が Ok の場合だけ合成を検証するため、入力エラーは先の項の空間不整合・重複より優先する。事前に配列・Vec を作った場合の生成処理や副作用は巻き戻さない。全入力成功後の合成検証が失敗する場合も、入力生成は最後まで行う。

State::try_sum・try_tensor_in、Operator::try_sum・try_product を基礎 API として公開し、ExpressionFactory はこれらへ委譲する。演算子の束縛は全入力を見て決め、後続に未束縛項があれば、それ以前の束縛が互換でなくても未束縛として構築する。逐次二項 fold で早期に拒否する実装にはしない。

各入口は構築のみを行い、評価・簡約・基底展開・自動正規化・暗黙の因子順序変更は行わない。構築の共有処理を使い、try 版のためだけの再収集や式の複製を避ける。未提供の補助 API は[将来の候補](../future-work/api-usability.md)で管理する。

## 2. 係数入力と型推論

局所・複合の4式型と LinearCombination の `scaled` は `impl Into<C>` を受け取り、入口で一度だけ C に変換する。式型は係数を保持するだけで数値能力・Clone を要求せず、LinearCombination は従来の `C: Clone + Mul<Output=C>` を維持する。変換元の Clone は不要で、空の LinearCombination でも変換は一度行う。C の精度は維持し、新たなキャストや精度変換は導入しない。Mul・scalar・term の引数は広げない。

入力型の推論が不足する場合は、構築入口の係数型や Default の入力型を明示し、不要な into() を外す。選択済みの C と最小能力の契約を維持する。型推論の指定方法は次のとおりである。

| 推論が不足する箇所 | 現在の書き方 |
| --- | --- |
| scaled の引数から式の C を推論 | ExpressionFactory で先に C を選ぶか、`LocalOperator::<Op, Complex<f64>>` などで明示する |
| `.scaled(value.into())` の内側の変換先 | `.scaled(value)` と入力を直接渡す |
| `.scaled(Default::default())` の生成型 | `.scaled(Complex::<f64>::default())` など入力型を明示する |

係数型を選択済みなら `.scaled(0.5)` と同じ精度の実数を渡せる。標準の `Complex<f32>` へ明示的な f64 を渡すなど、既存の Into が提供しない変換は受け入れない。利用例は[基底展開](../../examples/src/bin/basis_expansion.rs)と[2スピンの計算](../../examples/src/bin/two_spins.rs)を参照する。

LocalState::try_sum・LocalOperator::try_sum は `Result<式,ExpressionError>` の列を集約し、最初の入力 Err 以降を取り出さずに、そのまま返す。全成功時は sum と同じ項順・重複・零係数・空／単項の扱いを保ち、primitive を評価しない。

名前付きの和・積の構築は式型の列版 `sum` / `product` に絞り、二項は標準の `+` / `*` を使う。primitive の `Product::product` は局所規則として維持する。局所式の係数付き構築は `from_terms(impl IntoIterator<Item=Term<Self,C>>)` とし、配列・iterator・既存の LinearCombination を受け取る。数値演算・Clone・評価を要求せず、入力順・重複・ゼロ係数を保つ。
