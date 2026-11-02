# ノルム・状態正規化・正規化期待値

更新日: 2026-11-02

平方根を使うノルムと正規化、および自己内積で割る正規化期待値は `syntaqs-normalize` が拡張 trait として提供する。係数能力そのものは[数値規約](numerics.md)、作用と内積は[局所式](primitive-expression.md)・[複合式](composite-expression.md)に定める。

## 1. Norm・正規化の API と feature

係数全体を C、`C: RealPart` の実数型を R とする。内積・期待値・norm squared・スカラー除算は `syntaqs-core`、以下の拡張 trait と `NormalizationError` は std を利用する `syntaqs-normalize` が提供する。

| 操作 | 提供元 | 局所状態の返り値 | 複合状態の返り値 |
| --- | --- | --- | --- |
| `norm_squared()` | core の状態式 | R | `Result<R, ExpressionError>` |
| `norm()` | `Norm` | R | `Result<R, ExpressionError>` |
| `normalized()` | `Normalize` | `Result<Self, NormalizationError>` | 同左 |
| `normalized_expectation(&operator)` | `NormalizedExpectation` | `Result<C, NormalizationError>` | 同左 |
| `normalized_expectation_in(&operator, &ambient)` | `NormalizedExpectationIn` | 提供しない | `Result<C, NormalizationError>` |

norm squared は自己内積の実部、norm はその平方根とする。`norm()`・`normalized()` だけが R に `num_traits::Float` を要求し、正規化には実数を C に埋め込む `From<R>` も要求する。正規化は入力を変更せず、位相を保つ実数の逆ノルムで式をスカラー倍する。通常の作用・内積と同じ評価経路を使い、基底展開・行列化は要求しない。

正規化と正規化期待値は、自己内積全体の `Zero::is_zero()` による厳密なゼロを `NormalizationError::ZeroNorm` として拒否する。明示的な zero に加え、非独立な primitive や項の相殺による zero も対象とする。`norm(zero)` は0を返す。負値の補正・近似ゼロ判定は導入せず、NaN・infinity などは数値型に委ねる。複合式の空間・評価エラーは `NormalizationError::Expression(ExpressionError)` に変換し、元のエラーを source として保持する。

正規化期待値は期待値を自己内積全体で除し、`Divide` を要求する。平方根・実部取得・実数の埋め込みは要求しない。複素除算の方式と数値能力は[数値規約](numerics.md)に定める。明示空間を渡す複合式では、その空間を演算子の既存束縛より優先する。

拡張 trait とエラーは `syntaqs-normalize` から直接利用する。core の libm feature は設けず、core の代数 API・norm squared・スカラー除算は正規化 crate がなくても利用できる。非零の自己内積による商を明示的に計算でき、平方根を使わない正規化期待値の拡張 trait も `syntaqs-normalize` に属する。

## 2. 数学的な意味と検証

通常の期待値は `⟨ψ,Aψ⟩` であり、未正規化状態でも分母を補わない。正規化期待値は `⟨ψ,Aψ⟩/⟨ψ,ψ⟩` である。正規化は `ψ/√Re⟨ψ,ψ⟩` とし、入力の位相を保つ。

複合式では ordinary / graded の共通評価を利用し、選択済みの空間を保存する。内積・作用のための数値行列や基底展開は不要である。一般の式で数学的に相殺する場合も、浮動小数点の残差が厳密零でなければ ZeroNorm になるとは限らない。

テストは [contracts.rs](../../crates/syntaqs-normalize/tests/contracts.rs)・[expectation.rs](../../crates/syntaqs-normalize/tests/expectation.rs)、[三準位の横断テスト](../../tests/tests/normalize_three_level.rs)・[graded の横断テスト](../../tests/tests/normalize_tensor_products.rs)に置く。位相保持・厳密零の拒否・独自係数の最小能力・基底によらない計算・両精度の参照行列との一致を対象とする。
