# Stage 0: implementation contracts

Status: contracts fixed for Stages 1–4; none of these new production behaviors
is implemented by Stage 0. The starting revision is `a070071`.
Section 1 was subsequently implemented in [Stage 1](stage-1-results.md);
Sections 2–4 remain planned.
The [integrated plan](implementation-plan.md) owns scope and sequencing; the
[baseline record](stage-0-baseline.md) owns the new measurements and checks.

## 1. Structural operator grouping

The Stage 1 public method will have the following bounds:

```rust
impl<I: Eq, O: Identity, C: Clone> Operator<I, O, C> {
    pub fn group_local(&self) -> Self;
}
```

This is a contract sketch using existing trait/type names, not compiled code.
`Identity` permits localizing scalar identity when a subtree also references
one indexed factor. `C: Clone` permits moving stored coefficient values from
shared composite Scale nodes into local syntax. No coefficient arithmetic,
primitive equality, Product, Adjoint, parity, Dimension, or Basis capability is
required. Creating local Product/Adjoint nodes does not evaluate their rules.
Index and operator primitives need not be Clone.

### Construction-time coverage

After existing binding compatibility checks, collapse sums whose immediate
children are all Local nodes on the same indexed factor. Collapse products when
both immediate operands are Local nodes on that factor. Named sequence
constructors use these same rules while consuming their operands. Empty/single
constructors retain current identity/zero/singleton behavior.

Compare indices by I::eq, not Arc identity. Preserve local operand order.
These rewrites share local handles, require no C::Clone or O::Identity, and
introduce no bounds on the existing constructors. Do not look through composite
Scale/Adjoint nodes, walk a previously built subtree, or commute/reassociate
across a different factor during construction. Such cases use explicit/query
preparation.

### Whole-expression grouping

Use iterative postorder traversal with three classifications: scalar-only,
one indexed factor, or multiple factors. Query-local node-identity memoization
avoids rescanning shared subtrees. Rebuild changed paths and share other handles.

For a one-factor subtree, translate Local, Scale, Sum, Product, and Adjoint into
the corresponding local syntax, preserving sum/product order and coefficient
placement. Translate scalar-only children into local identity/zero syntax in
that factor's context. Keep entirely scalar roots scalar: do not choose an
arbitrary ambient factor. Keep empty sums zero and empty products identity;
perform no scalar arithmetic or zero annihilation. Even a syntactically zero
product retains its original indexed references.

For a multi-factor subtree, retain its composite operation and grouped children.
Do not gather separated occurrences through another factor or attempt common
factor extraction. Reapplying grouping must leave the resulting shape stable.
The original binding, including None, is copied unchanged.

No ambient space is required by this structural API. Automatic preparation runs
only after validation of the original expression against the selected space.
Start with operator expand/coordinates and collect_terms_in, whose current
bounds already permit grouping; use the same helper for trace/HS preparation
where compatible. Do not route state action through an operator Product rule.
The existing homogeneous decomposition and graded sign algorithms remain owners
of signs. No persistent grouped flag or public support metadata is added.

## 2. State evaluation capabilities and normalized form

The Stage 2 evaluator bounds are:

| Operation | Required bounds |
| --- | --- |
| LocalState::evaluate | C: Clone + Zero + One + Mul<Output=C> + Conjugate<Output=C>; S: PartialEq + Act<C, Operator=O, Term=S>; O: Adjoint<C, Term=O> |
| State::evaluate | The local bounds, plus I: Eq, C: PartialEq, O: HomogeneousParity + ParityOperator |

Zero already requires addition and One already requires multiplication; explicit
Mul in the table records the operation rather than an additional capability.
Local state collection does not compare coefficients to identify primitive
terms. Composite collection compares coefficients inside retained local
polynomials and factored sums, hence its additional C::PartialEq.

State methods that invoke evaluate inherit these bounds. Local/composite basis
queries, inner products, norms, expectations, normalize adapters, and state/matrix
export must propagate them with existing operation-specific bounds. Construction
does not inherit evaluation bounds. Operator-only queries do not acquire state
comparison bounds. No Eq/Ord/Hash, primitive/index Clone, or operator Product
requirement is added to action evaluation.

### Normalization rules

The internal collector represents each additive contribution as an outer
coefficient plus an action-free shape. Shapes contain unit, placed local
polynomial, ordered tensor factors, or retained additive subexpressions.
All primitive payloads remain in shared handles.

1. Local polynomials flatten additive nesting, combine stored scales, and add
   coefficients of PartialEq-matching primitives. Keep first-occurrence order.
   Remove exact-zero coefficients. Preserve an implicit singleton coefficient
   when possible; a stored one coefficient need not be removed without C::PartialEq.
2. Composite tensors flatten tensor parentheses, remove scalar unit factors,
   pull child outer coefficients into the tensor's outer coefficient, and retain
   leaf order and the original containing space. Never distribute child sums.
3. Composite sums flatten additive nesting and propagate surrounding scales
   while collecting their contributions. Single-factor sums use the same local
   polynomial representation, whether originally inside or outside placement.
   Retained multi-factor sums remain structural children of enclosing tensors.
4. A one-term local polynomial exposes its coefficient to the enclosing
   contribution. This makes `(2 a) tensor b` and `2 (a tensor b)` match without
   comparing coefficient locations. Do not divide by a coefficient or extract
   numeric common factors from multi-term polynomials.
5. Matching uses the resulting ordered shape, primitive PartialEq, and coefficient
   PartialEq inside retained shapes. Sum operand order after collection is
   first-occurrence order; do not permute retained sums to search for additional
   equivalences. Do not identify factored and distributed forms.
6. A zero tensor factor makes the tensor zero after reference/space validation.
   Preserve the selected containing space for zero; never change the Act scope.

A complete global sum-of-products list is not the normalized form.
The collector must retain `tensor_i(a_i+b_i)` as a product of sums.
Natural primitive equality need not resolve linear dependence between distinct
primitives; basis expansion remains responsible for that additional aggregation.

### Matching index and readiness

Use a query-local index of normalized shapes, keeping first-occurrence output
order separately from lookup order. Equality-only primitive/coefficient
interning assigns scratch integer IDs; BTreeMap keys over structural
descriptors then avoid repeated full product comparisons without requiring
Ord/Hash on user values. Local primitive collection uses a linear list.
These IDs never escape the evaluation and are not physical basis indices.
Do not add a persistent hashed canonical form or automatic global sharing.

Use iterative work stacks and a per-call node memo plus an internal normalized
result wrapper carried by evaluator tasks. Normalize each original shared
subtree at most once per call, and pass already normalized action results to
subsequent tasks without recollection. No persistent collected flag is introduced
in the initial implementation; cross-call rescanning remains a measured cost.
Primitive/unit inputs can use their intrinsically trivial normal form directly.

Action-free inputs are normalized too, including duplicate sums. Re-evaluation
has stable structure but is not promised O(1). The existing has_action and
pending_action flags remain action-presence facts, not collection certificates.
Pointer memoization permits reuse of processed subtrees; it must not override
value PartialEq when comparing terms, including nonreflexive values.
If measurements justify persistent normalization metadata later, revisit that
choice explicitly instead of silently turning action flags into caches.

### Action boundaries

For A+B, send each branch's contributions from the same normalized input to one
collector; do not accumulate a separate uncollected result tree. For AB, apply B,
collect, then apply A. Enforce the same boundaries inside local actions.
Adjoints retain reversal/conjugation, and graded action uses the original Act
space and actual local parity-operator action.

## 3. Exact-zero inner-product stopping

Stage 3 changes product scheduling, not the state or operator inner-product
definition. Preserve independent tensor boundaries and conjugate linearity.
Maintain a continuation containing the accumulated value and remaining factors,
schedule one contraction, and resume only if the result is not exactly zero.
Skip subsequent multiplication/scaling work made unnecessary by zero.

Never terminate a sum because one pair or a partial sum is zero. Keep other
summands. On HS paths, keep the existing normalized local trace calculation and
ordinary lowering with parity strings; no basis or state type is introduced.
Inner-product-only helpers need Zero but no new coefficient PartialEq beyond
state evaluation's inherited requirements. Original reference validation precedes
short-circuiting. Use numerical conventions for underflow and nonfinite values;
no approximate threshold or finite-value rejection is added.

## 4. Coordinate generation and error boundaries

Stage 4 keeps existing operator-query bounds and Coordinates output. Reuse a
lazy query-local expansion of identity, create a working full-length key, and
enumerate complete local-choice combinations directly into the result map.
Single-component identity I=gamma B_q supplies default index q and coefficient
factors gamma. Use multiplication, not division. Multi-component identities
share choices, not a precomputed spectator tensor map. Expand output additionally
shares local basis-expression handles and constructs the required placed syntax.

Preserve existing basis-error visibility. In particular,
[core_basis_expansion](../../../tests/tests/core_basis_expansion.rs) checks that
zero coefficients or a zero factor do not hide malformed expansion in another
factor. Expand/validate all specified local factors and required spectator
choices of a reached intrinsic term before suppressing its numeric combinations.
A zero partial product must not skip a later basis validation that the existing
query would perform. Never request identity expansion for a query with no
spectators or for the zero-factor unit space.

Keep selected-space validation ahead of grouping and generation, and preserve
index membership and local dimension checks. The current basis traversal may
eliminate an empty intrinsic branch before reaching the generator; this is not
permission to hide errors on later factors of a reached branch.

Identity-call counters include both spectators and I produced by explicit local
products. Stage 4 must reduce repeated spectator expansion, not force every
identity-related basis call in H² to disappear. Use the unsquared Pauli H case
to isolate the spectator count, alongside total H² measurements.

## 5. Stage 0 evidence and handoff

The probe's added semantic fixtures run against the unchanged library. They
provide expected mathematical outputs for normalization cases, independent
Heisenberg bit-state action, and multi-component/scaled-identity bases. They do
not claim structural collection, grouping API availability, or early stopping.
Those structural/call-count assertions must be added when each later stage is
implemented, with exact capability tests and non-Clone types.

The audited probe adds construction, action-free, low-duplicate, no-zero, and
operator-HS controls, seven batched timing samples, complete coordinate-length
checks, explicit mode validation, source/toolchain provenance, and separate
operation counts. Baseline-only n^3 assertions were removed: reduced call counts
are intended later. Raw counts and baseline expectations remain recorded as data.
No production runtime code is changed in Stage 0.
