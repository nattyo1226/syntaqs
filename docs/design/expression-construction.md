# Type selection and expression construction

Updated: 2026-11-17

Choose primitive and coefficient types once, then construct local or placed
expressions through the same factory. Construction stores shared syntax;
[local evaluation](primitive-expression.md) and [space-aware evaluation](composite-expression.md)
apply the mathematical rules when a result is requested.

## 1. Type selection, placement, and collection

`QuantumSystem` associates state/operator primitives and coefficients through
`State`, `Operator`, and `Coeff`. `ExpressionFactory` provides inherent
constructors for the corresponding core expressions. It stores no spaces or
indices and requires no uniform Clone or evaluation capabilities; combining
composite expressions requires `I: Eq`.

`ExpressionFactory<S,O,C>` stores no values and needs no instance; it implements QuantumSystem generically. `SpinHalf<C>`, `SpinOne<C>`, and `Fermion<C>` select concrete primitives through type aliases, without a trait import for construction. Custom primitives can also use a factory alias. The same primitive/coefficient types produce the same expression types; a system name adds no identity.

Use `LocalStateOf<Q>`, `LocalOperatorOf<Q>`, `StateOf<Q,I>`, and `OperatorOf<Q,I>` for annotations. Generic system APIs obtain types through `Q: QuantumSystem` and construct through expression entry points such as `LocalStateOf::<Q>::primitive(...)` or `ExpressionFactory<Q::State,Q::Operator,Q::Coeff>`. QuantumSystem itself has no constructors. Model-specific construction belongs in each primitive crate's public `models` module under the [model design](models.md).

| Entry point | Inputs and contract |
| --- | --- |
| state / operator | One state/operator primitive; returns an unplaced local expression without coefficient arithmetic or evaluation bounds. |
| `state_tensor_from(primitives, &space)` | Places exactly one state primitive per leaf in space order. Counts must match. |
| `operator_product_from(assignments)` | Primitive/factor pairs multiplied in input order, allowing repeated indices. The rightmost operator acts first. Empty input is unbound identity. |
| `state_sum(states, &space)` / `try_state_sum(states, &space)` | Sums states on the explicit space. Empty input is zero on that space. |
| operator_sum / try_operator_sum | Operators; empty input is unbound zero. |
| `state_tensor(states, &space)` / `try_state_tensor(states, &space)` | Tensors placed states whose concatenated leaves exactly cover the explicit space in order, with compatible subsystem kinds. |
| operator_product / try_operator_product | Placed operators multiplied in input order; empty input is unbound identity. |

`state_tensor_from` and `operator_product_from` build expressions from
primitives. Each item passed to `operator_product_from` has the form
`(primitive, &factor)`. Placement requires a leaf. The result is unbound,
and `bind` selects the full ambient space including spectator factors.
Shared expressions and spaces can outlive input slices and placement variables.

`try_` denotes Result inputs; ordinary composite constructors also return Result.
Input errors use `ExpressionError`. Fallible collection follows this order:

1. Consume inputs until the first Err, returning that error unchanged and making
   no further calls to next.
2. If every input succeeds, validate the complete composition. Input errors
   therefore precede space or duplicate errors in earlier successful operands.

Already-built arrays/vectors and producer side effects are not rolled back. If
composition fails after successful input generation, all inputs have been produced.

State::try_sum/try_tensor_in and Operator::try_sum/try_product are the underlying public APIs; ExpressionFactory delegates to them. Binding is decided from the entire operator sequence. A later unbound item makes the result unbound even if earlier bound items are incompatible; do not replace this with an early-rejecting binary fold.

These entry points do not evaluate primitive rules, perform coefficient
arithmetic, expand bases, normalize states, or reorder factors. They preserve
duplicates and zero coefficients. After binding checks, operator sums whose
immediate children are all local to the same factor become one local sum.
An operator product groups leading same-factor local operands into one local
product. Each remaining composite product stores its operands in one ordered
sequence. Both operations use `Operands` for storage and a common path for
operand collection and binding checks, including `+`, `*`, and the named
constructors. Existing product operands keep their shared subtrees. Factor
matching uses index equality.
This structural grouping adds no coefficient
Clone/arithmetic or primitive Identity/evaluation bounds and does not inspect
through composite scales, adjoints, or other previously built subtrees.
See [local grouping](composite-expression.md#113-structural-local-operator-grouping)
for explicit and query-time grouping.

Reuse construction and sharing without extra collection or copying just for
try_ variants. Unprovided helpers belong in the
[candidate backlog](../future-work/api-usability.md).

## 2. Coefficient inputs and inference

`scaled` on all four expression types and LinearCombination accepts `impl Into<C>`
and converts exactly once at entry, including for an empty LinearCombination.
The input need not be Clone. Expressions only store the resulting coefficient,
while LinearCombination requires `C: Clone + Mul<Output=C>`.

The selected C fixes precision; this conversion adds no implicit casts.
`Mul`, `scalar`, and term constructors take their declared coefficient inputs
rather than this broader Into interface.

When inference is insufficient, specify C at construction or the type of a Default input, and remove redundant into() calls. Preserve the selected C and minimal bounds:

| Inference issue | Explicit form |
| --- | --- |
| Inferring C from scaled's argument | Select C through ExpressionFactory or annotate, for example `LocalOperator::<Op, Complex<f64>>`. |
| Conversion target inside `.scaled(value.into())` | Pass `.scaled(value)` directly. |
| Type produced by `.scaled(Default::default())` | Specify the input, for example `.scaled(Complex::<f64>::default())`. |

After selecting C, `.scaled(0.5)` accepts matching real precision. Conversions unavailable through Into, such as explicit f64 into `Complex<f32>`, remain rejected. See [basis expansion](../../examples/src/bin/basis_expansion.rs) and [two spins](../../examples/src/bin/two_spins.rs).

LocalState::try_sum and LocalOperator::try_sum collect `Result<Expression,ExpressionError>`, stop at the first Err, and return it unchanged. Successful inputs preserve sum's order, duplicates, zero coefficients, and empty/singleton behavior without primitive evaluation.

Named sums/products take sequences through `sum`/`product`; binary operations use standard `+`/`*`. Primitive `Product::product` remains the local rule. Local coefficient/operand construction uses `from_terms(impl IntoIterator<Item=Term<Self,C>>)` for arrays, iterators, and existing LinearCombination values without arithmetic, Clone, or evaluation, preserving order, duplicates, and zero coefficients.
