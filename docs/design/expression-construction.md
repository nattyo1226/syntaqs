# Type selection and expression construction

Updated: 2026-11-02

This document specifies common construction and coefficient inputs. See [local expressions](primitive-expression.md) for evaluation and [composite expressions](composite-expression.md) for space-aware operations.

## 1. Type selection, placement, and collection

QuantumSystem associates state/operator primitives and coefficients through `State`, `Operator`, and `Coeff`. ExpressionFactory's inherent methods construct existing core expressions without registering/storing spaces or indices or uniformly requiring Clone/evaluation capabilities. Combining composite expressions requires `I: Eq`.

`ExpressionFactory<S,O,C>` stores no values and needs no instance; it implements QuantumSystem generically. `SpinHalf<C>`, `SpinOne<C>`, and `Fermion<C>` select concrete primitives through type aliases, without a trait import for construction. Custom primitives can also use a factory alias. The same primitive/coefficient types produce the same expression types; a system name adds no identity.

Use `LocalStateOf<Q>`, `LocalOperatorOf<Q>`, `StateOf<Q,I>`, and `OperatorOf<Q,I>` for annotations. Generic system APIs obtain types through `Q: QuantumSystem` and construct through expression entry points such as `LocalStateOf::<Q>::primitive(...)` or `ExpressionFactory<Q::State,Q::Operator,Q::Coeff>`. QuantumSystem itself has no constructors. Model-specific construction belongs in each primitive crate's public `models` module under the [model design](models.md).

| Entry point | Inputs and contract |
| --- | --- |
| state / operator | One state/operator primitive; returns an unplaced local expression without coefficient arithmetic or evaluation bounds. |
| product_state | Explicit space and state primitives, placing exactly one per leaf in space order; counts must match. |
| operator_product_at | Factor/primitive pairs multiplied in input order, allowing repeated indices; rightmost acts first; empty input is unbound identity. |
| state_sum / try_state_sum | Explicit space and states; empty input is zero on that space. |
| operator_sum / try_operator_sum | Operators; empty input is unbound zero. |
| tensor_in / try_tensor_in | Explicit space and placed states whose concatenated leaves exactly cover it in order, with compatible subsystem kinds. |
| operator_product / try_operator_product | Placed operators multiplied in input order; empty input is unbound identity. |

The `_at` in operator_product_at means it also places primitives: inputs are an IntoIterator of `(&factor, primitive)`, not bare primitives. Placement requires leaves without automatic tensor expansion. The product stays unbound until explicitly bound; its placements do not determine an ambient space including spectators. Shared expressions/spaces can outlive input slices and placement variables.

`try_` denotes Result inputs; ordinary composite constructors also return Result. Input errors use ExpressionError. Return the first input Err unchanged and stop calling next afterward. Composition is validated only after every input succeeds, so input errors precede earlier operands' space/duplicate errors. Already-built arrays/vectors and producer side effects are not rolled back. If final composition fails after successful inputs, all inputs have still been generated.

State::try_sum/try_tensor_in and Operator::try_sum/try_product are the underlying public APIs; ExpressionFactory delegates to them. Binding is decided from the entire operator sequence. A later unbound item makes the result unbound even if earlier bound items are incompatible; do not replace this with an early-rejecting binary fold.

These entry points only construct; they do not evaluate, simplify, expand, normalize, or reorder factors. Reuse construction and sharing without extra collection or copying just for try_ variants. Unprovided helpers belong in the [candidate backlog](../future-work/api-usability.md).

## 2. Coefficient inputs and inference

`scaled` on all four expression types and LinearCombination accepts `impl Into<C>` and converts exactly once at entry. Expressions only store the coefficient without requiring numeric capabilities or Clone; LinearCombination retains `C: Clone + Mul<Output=C>`. The source need not be Clone, and even an empty LinearCombination converts once. C's precision stays fixed without new casts/conversions. Mul, scalar, and term inputs are not broadened.

When inference is insufficient, specify C at construction or the type of a Default input, and remove redundant into() calls. Preserve the selected C and minimal bounds:

| Inference issue | Explicit form |
| --- | --- |
| Inferring C from scaled's argument | Select C through ExpressionFactory or annotate, for example `LocalOperator::<Op, Complex<f64>>`. |
| Conversion target inside `.scaled(value.into())` | Pass `.scaled(value)` directly. |
| Type produced by `.scaled(Default::default())` | Specify the input, for example `.scaled(Complex::<f64>::default())`. |

After selecting C, `.scaled(0.5)` accepts matching real precision. Conversions unavailable through Into, such as explicit f64 into `Complex<f32>`, remain rejected. See [basis expansion](../../examples/src/bin/basis_expansion.rs) and [two spins](../../examples/src/bin/two_spins.rs).

LocalState::try_sum and LocalOperator::try_sum collect `Result<Expression,ExpressionError>`, stop at the first Err, and return it unchanged. Successful inputs preserve sum's order, duplicates, zero coefficients, and empty/singleton behavior without primitive evaluation.

Named sums/products take sequences through `sum`/`product`; binary operations use standard `+`/`*`. Primitive `Product::product` remains the local rule. Local coefficient/operand construction uses `from_terms(impl IntoIterator<Item=Term<Self,C>>)` for arrays, iterators, and existing LinearCombination values without arithmetic, Clone, or evaluation, preserving order, duplicates, and zero coefficients.
