# Core evaluation and basis-expansion performance investigation

Date: 2026-11-15  
Baseline: `8b18d8b`  
Status: investigation complete; directions for all four findings are agreed.
Stage 0 is complete: [implementation contracts](stage-0-contracts.md) and an
[audited baseline](stage-0-baseline.md) are recorded. Stage 1 local grouping is
implemented; see its [results and validation](stage-1-results.md). The change map, remaining stage order, and
acceptance criteria are in the [integrated implementation plan](implementation-plan.md).

## Objective, scope, and progress

Identify bottlenecks in `syntaqs-core` using operators and states users can
reasonably construct. Separate intermediate expression growth from the cost of
producing an intrinsically large result.

| Stage | Progress |
| --- | --- |
| Inspect evaluation, traversal, contraction, and basis expansion | Complete |
| Measure realistic cases and allocation requests | Complete |
| Check equivalent representations, analytic results, and operation counts | Complete |
| Run relevant existing tests and record findings | Complete |
| Agree on the local-operator grouping direction | Complete; Stage 1 implemented |
| Agree on repeated-action aggregation and its initial equivalence scope | Complete; implementation details remain open |
| Bound the inner-product improvement to local contraction and exact-zero short-circuiting | Complete; further acceleration deferred |
| Agree on sparse coordinate generation with shared identity expansion | Complete; implementation details remain open |
| Consolidate changes, dependencies, stages, and acceptance criteria | Plan recorded; Stages 0 and 1 complete |
| Stage 0: contracts, probe audit, semantic fixtures, and current-revision baseline | Complete |
| Stage 1: local construction/grouping API and query preparation | Complete; Stages 2–5 not started |

Completion criteria were met: examples have explicit definitions, measurements
have a recorded method, source paths explain the findings, and proposals are
distinguished from verified behavior. Subsequent implementation is tracked by stage.

The historical investigation below provides no performance guarantees. At the investigated baseline, ordinary evaluation
intentionally retains duplicate and zero terms. The agreed direction in finding
2 would change that behavior and its capability requirements.
See [local expressions](../../design/primitive-expression.md),
[composite expressions](../../design/composite-expression.md), and
[basis expansion](../../design/basis.md).

### Agreed directions at a glance

The following sections retain the investigation and initial design discussion.
Where they identify implementation choices, the later
[Stage 0 contracts](stage-0-contracts.md) now resolve those choices for the first
implementation. Section 1 is now implemented by Stage 1; Sections 2–4 remain
planned.

| Finding | Adopted scope |
| --- | --- |
| 1. Composite sum distribution | Same-factor local grouping at construction, before evaluation, and through an explicit API, using shared rules |
| 2. Repeated state actions | Collect matching natural-state terms between actions in `evaluate()`, normalizing tensor grouping and coefficient placement without distributive-equivalence checks |
| 3. Inner products | Preserve local factorization and short-circuit unnecessary work on exact zeros; further acceleration is deferred |
| 4. Operator basis coordinates | Share the identity's local expansion within a query and emit complete coordinates directly from sparse factor data |

Direction 1 is implemented; directions 2–4 remain planned. The sections below
retain baseline measurements and describe each adopted scope. The implementation
plan tracks the combined sequence and current completion status.

## Method and reproduction

Measurements used an aarch64 Apple Silicon host, Rust 1.99.1, release optimization,
`Complex<f64>`, `num-complex 0.4.6`, and `num-traits 0.2.19`. The initial timing
runs preceded the existing test-suite run.

Each case warms up once and reports the median of five executions, including
destruction of its result. Input construction is outside the timed closure
except for per-step action/expansion loops and the action node in the Heisenberg
action case. Allocation requests are counted in a separate execution using a
system-allocator wrapper. Timed runs disable counting but retain that wrapper.

Allocated bytes are cumulative requested allocation and reallocation sizes,
**not peak live memory or resident memory**. CPU sampling, confidence intervals,
and cross-machine comparisons were not collected.

The retained [probe source](probe/src/main.rs) and [manifest](probe/Cargo.toml)
are an isolated package outside the production Cargo workspace. Run from the
repository root:

```sh
cargo run --release --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo run --release --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- extra
cargo run --release --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- count
cargo run --release --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- h2-count
```

The default run measures rotations, Heisenberg chains, Hamiltonian powers, and
product states. `extra` measures Hadamard representations, per-step Hamiltonian
expansion, and W states. `count` checks W-state primitive inner-product counts;
`h2-count` counts basis calls for the 64-site squared Hamiltonian.
Add `--offline` when dependencies are cached, as in the original investigation.
The package resolves dependencies separately from the root lockfile.

The five-single-call-sample method and tables in this investigation are historical.
Stage 0 subsequently audited the probe to use seven batched samples and added
regression controls. See its [baseline record](stage-0-baseline.md) for the
current measurement method, dependency snapshot, raw samples, and reproduction.

The tables preserve original timing runs. Analytic assertions for rotations
and product-state norms were added afterward and passed on a rerun.
This probe is an investigation tool, not a stable benchmark suite.

## 1. Global sums distribute before local cancellation

Let X, Y, and Z denote the Pauli-normalized operators `Sx`, `Sy`, and `Sz`.
For a Hadamard layer U = tensor_i((X_i + Z_i)/sqrt(2)), U†U should expand to
identity alone in `CartesianBasis`. The probe computes the equivalent V†V
scaled by 2^(-n), with V = product_i(X_i + Z_i). Normalization outside the
product avoids rounding artifacts in this cancellation experiment.

Compare constructing X + Z as a `LocalOperator` before placement with adding
separately placed `Operator` values:

| Sites | Local-sum coordinates | Placed-sum coordinates |
| ---: | ---: | ---: |
| 4 | 0.008125 ms | 0.758042 ms |
| 6 | 0.009292 ms | 12.025667 ms |
| 8 | 0.010708 ms | 254.308750 ms |

Both returned exactly one coordinate with coefficient one. At eight sites,
placed sums caused 10,555,044 allocation requests and 1,094,632,312 requested
bytes; local sums caused 481 requests and 31,520 bytes.

Mechanism: [`OpTerms::next`](../../../crates/syntaqs-core/src/expr/traversal.rs)
clones branches at composite sums, distributing into 4^n branches before local
multiplication and basis aggregation cancel them. Local sums allow cancellation
per factor before the global tensor product grows.

Agreed direction: group compatible same-factor sums/products before global
distribution, using the three entry points below. Callers can already avoid
this cost by completing local gates before placement.

### Agreed local-grouping direction

Use limited grouping during construction, a more comprehensive traversal before
evaluation, and an explicit API that lets callers retain and reuse the grouped
expression. This agreement concerns finding 1 only; it does not approve
implementation or settle the overall optimization plan.

“Same factor” means the same single indexed leaf, not merely the same ambient
space. Operators on different factors of one bound space are not eligible for
direct local grouping.

| Entry point | Responsibility | Initial scope |
| --- | --- | --- |
| Construction | Preserve immediately visible local structure at low cost | Group direct local operands on the same factor, especially sums and products; do not scan arbitrary subtrees |
| Before evaluation | Recover local structure missed during construction | Traverse subtrees before composite distribution; start with operator basis expansion and coordinates |
| Explicit API | Let callers obtain and reuse the grouped expression | Apply the same subtree-grouping rules as the evaluation preparation |

Share the eligibility and transformation rules across these entry points.
Construction applies them to immediate children; evaluation preparation and the
explicit API apply them throughout the expression. Avoid three independent
normalizers with different semantics.

Grouping changes representation only: move sums, ordered products, scales, and
adjoints into a `LocalOperator` where supported. Do not invoke primitive product
rules, expand a basis, collect equal primitives, remove zero contributions, or
extract common factors from multi-factor sums. The explicit API is distinct from
`collect_terms_in()`, which evaluates primitive products and collects terms.

#### Ordering, grading, and validation

Only group a subtree whose factor references all belong to one factor.
Preserve left-to-right product order within that factor. Do not gather separated
occurrences across another factor: transforming B_1 A_0 C_1 into A_0 (BC)_1
requires a minus sign when all three operands are odd.

Same-factor embedding preserves sums, products, and adjoints in both ordinary
and graded spaces, including mixed-parity local expressions. Grouping therefore
adds no exchange sign and needs no construction-time parity classification.
Keep the existing evaluation sequence afterward: `HomogeneousTerms` splits local
expressions into even/odd parts, and `IntrinsicOpTerms` computes crossing signs
using the selected space's factor order. Do not enable unrestricted same-factor
grouping in the raw graded traversal or apply exchange signs twice.

For automatic evaluation preparation, select the ambient space and validate the
original expression before grouping, then run the grouped expression through
`OpTerms` and the existing grading path. Keep the original expression unchanged
and share unchanged subtrees. Use an iterative traversal and perform preparation
once per query, not once per iterator item.

Preserve construction-time binding compatibility checks. Grouping must not erase
factor references that later validation needs, including references under zero
coefficients. Handling scalar identity within a single-factor subtree must
respect existing capability bounds and must not introduce a new indexed factor.

#### Accepted explicit API and resolved details

The accepted API is `Operator::group_local(&self) -> Self`, subject to the
capability bounds needed for structural grouping. It returns a grouped
expression without changing its input, retains the original binding, preserves
factor order, and makes no further structural changes when reapplied to an
already grouped result. It also accepts unbound operators and invokes no
primitive evaluation. Ambient-space validation remains at binding/evaluation
time; original factor references must be preserved.

Stage 0 fixed these details, and Stage 1 now implements them:

- Construction groups direct local sums/products only, with unchanged bounds.
  Scales, adjoints, and scalar identities are handled by whole-expression
  grouping; pure scalar roots and empty scalar sums remain scalar.
- The explicit API requires only `I: Eq`, `O: Identity`, and `C: Clone`.
- Operator basis queries, trace/HS, and `collect_terms_in()` share query
  preparation after original-reference validation. State action retains its
  existing evaluation path and capability requirements.
- There is no persistent marker. A bounded candidate scan avoids unnecessary
  memo setup on small unchanged expressions, with query-local memoized postorder
  traversal for larger or potentially groupable trees.

The [Stage 1 record](stage-1-results.md) reports verification comparing grouped and
ungrouped coordinates, actions, traces, and adjoints in ordinary and graded
spaces, including noncommuting same-factor products, odd crossings, mixed parity,
scalar identities, rebinding, invalid references, non-Clone primitives/indices,
and deep/shared trees. Construction time, query time, allocation counts, and
already-local controls are measured separately. The Hadamard comparison shows
the placed-sum penalty is removed, with construction costs recorded explicitly.

## 2. Repeated actions retain many contributions to small states

Apply R = cos(0.1) I - i sin(0.1) Y repeatedly to `Up`. After k rotations the
state has two coordinates: cos(0.1k) Up + sin(0.1k) Down.

| Rotations | Evaluate only | Coordinates after all actions | Expand after every action |
| ---: | ---: | ---: | ---: |
| 8 | 0.317000 ms | 0.304750 ms | 0.010750 ms |
| 12 | 4.787000 ms | 5.032500 ms | 0.011542 ms |
| 16 | 97.865375 ms | 107.578250 ms | 0.015666 ms |

At 16 rotations, final coordinates caused 3,866,509 allocation requests and
199,231,264 requested bytes. Per-step expansion caused 640 requests and
41,472 bytes. Both paths and the analytic formula agree within `1e-10`.

The same behavior occurs with the existing open-chain Heisenberg model at
coupling one,
H = -(1/4) sum_i(X_i X_(i+1) + Y_i Y_(i+1) + Z_i Z_(i+1)),
acting repeatedly on an alternating Up/Down Neel state. Powers H^k |Neel>
occur in Krylov constructions and moment calculations.

| Sites | Power | Final nonzero coordinates | Evaluate only | Coordinates after all actions | Expand after every action |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 6 | 1 | 6 | 0.014459 ms | 0.039875 ms | Not measured |
| 6 | 2 | 12 | 0.234083 ms | 0.615500 ms | 0.329291 ms |
| 6 | 3 | 16 | 3.516333 ms | 9.154500 ms | 0.914333 ms |
| 6 | 4 | 18 | 56.287209 ms | 142.004250 ms | 1.631000 ms |

At power four, final coordinates caused 4,880,862 allocation requests and
577,280,160 requested bytes. Per-step expansion caused 58,832 requests and
9,116,744 bytes. Both paths produced exactly equal coordinates. The six-site
zero-magnetization sector has only 20 computational-basis states.

Mechanism: [local action evaluation](../../../crates/syntaqs-core/src/evaluate/local.rs)
distributes sums and retains duplicates.
[Composite state expansion](../../../crates/syntaqs-core/src/basis/expansion/composite.rs)
completes `state.evaluate()` before aggregating coordinates. Final aggregation
does not bound the intermediate evaluated expression.

Agreed direction: make `evaluate()` aggregate repeated contributions between
actions without selecting a basis. Then let `coordinates(basis)` expand the
evaluated result in the requested basis and aggregate basis indices. This
replaces the earlier candidate of requiring a basis-aware evaluation path for
intermediate aggregation while preserving the old `evaluate()` bounds.
Per-step `expand()` remains the measured caller-side workaround; those timings
do not measure the agreed evaluator design.

### Agreed repeated-action aggregation direction

The evaluator should resolve an action, normalize its result within the scope
below, and combine matching terms before applying the next operator. Inputs
without pending actions are also normalized and collected: `Up + Up` evaluates
to `2 Up`. Re-evaluating a normalized result should not change its structure.
This is collection in the natural primitive representation, not basis expansion.

```text
evaluate():
    action -> normalize and collect -> action -> normalize and collect -> ...

coordinates(basis):
    evaluate() -> expand in the selected basis -> collect equal basis indices
```

For example, two action paths producing the same primitive state combine
`a |Down> + b |Down>` into `(a+b) |Down>`. Distinct primitives need not be
recognized as linearly dependent: the selected basis can resolve those relations
later. The number of genuinely distinct nonzero states may still grow.

#### Collection boundaries and capability requirements

For an operator sum A+B, apply both operands to the same input state and add
their contributions to a common result collector. Do not feed A's output into
B. Collect contributions as they arrive rather than retaining the entire
uncollected result tree until the end of the sum.

For an ordered product AB, apply B first, collect its result, and then apply A.
Apply the same principle to an explicit chain of delayed actions. Local and
composite evaluation must agree on this behavior; collection at the composite
boundary alone must not leave local repeated actions expanding unchecked.

Requiring `PartialEq` on state primitives for `evaluate()` is accepted as a
natural requirement of this responsibility. Coefficient `PartialEq` is also
accepted on evaluation paths that need to compare coefficients inside retained
factored structures; do not add it to construction or operations that need no
such comparison. Coefficient capabilities must also
support collection, including coefficient addition, multiplication when
normalizing nested scales, and exact-zero detection. The exact Rust bounds and
their propagation to callers remain to be specified. This is an intended
capability change, not a promise to preserve the baseline's weaker bounds.
Do not silently replace `PartialEq` with an `Eq`, `Ord`, or `Hash` requirement.

#### Initial scope of matching terms

Normalize tensor grouping and coefficient placement while preserving factor
order, then collect terms whose remaining structure matches using primitive
equality. Writing a, b, and c for local states, the initial scope includes:

| Case | Intended treatment |
| --- | --- |
| `2 a + 3 a` | Combine into `5 a` |
| `2 (a tensor b) + 3 (a tensor b)` | Combine coefficients of the matching product state |
| `(a tensor b) tensor c` versus `a tensor (b tensor c)` | Recognize the same ordered factors after internal tensor regrouping |
| `(2 a) tensor b` versus `2 (a tensor b)` | Normalize the coefficient position before comparison |

Internal regrouping does not permute factors, add a graded sign, or replace the
state's containing space and its grouping. Existing compatibility validation
still applies.

Do not search for distributive equivalence or factorize multi-term expressions
to determine equality. In particular, the initial collector need not identify
`(|a> + |b>) tensor |c>` with
`|a> tensor |c> + |b> tensor |c>`. Preserve existing product-of-sums structure;
do not distribute it solely to obtain a uniform sum-of-products representation.
This avoids expanding `tensor_i(|a_i> + |b_i>)` into 2^n terms just to compare
states. General distributive matching and common-factor extraction are outside
the agreed initial scope.

#### Zero handling, grading, and remaining design work

Remove coefficients only when they are exactly zero. Retain small nonzero
coefficients; tolerance-based pruning would be a separate, explicitly approximate
feature. Changed addition order can change floating-point rounding, so numerical
verification must not require bitwise agreement with the old evaluation path.

Keep the existing action order, each delayed Act's selected space, and parity
operator actions on preceding factors. Structural collection must not move an
operator across another factor or replace a parity action by an assumed scalar
sign. Preserve validation of original references even if collected contributions
cancel to zero.

The remaining design work includes:

- The collector representation and matching algorithm under `PartialEq`,
  including comparison of retained factored sums and their coefficients.
- Normalization boundaries for local versus composite sums, including equivalent
  local-state content stored inside or outside a placement.
- Required coefficient bounds and effects on dependent APIs and capability tests.
- Avoiding repeated normalization of already collected subtrees while also
  collecting action-free inputs. The existing ready-subtree shortcut does not
  by itself establish collected form.
- Reuse of normalized/shared subtrees and control of comparison costs without
  requiring full expansion or persistent caches.

Before accepting an implementation, verify repeated rotations and Heisenberg
powers against analytic or independent small-system results. Test the matching
cases above, cancellation, retained tiny coefficients, noncommuting operator
order, graded Act scope and parity actions, and deep/shared trees. Confirm that
product-of-sums inputs stay factored. Measure local and composite evaluation,
coordinate extraction, allocation counts, and cases with few duplicates; the
effect of comparison overhead remains unmeasured.

## 3. W-state norms evaluate all pairs and all factors

The normalized W state is the sum of n product states with one Down spin and all
other spins Up, scaled by 1/sqrt(n). The probe uses n terms of n local factors.

| Sites | `norm_squared()` | Generate `SzBasis` coordinates |
| ---: | ---: | ---: |
| 8 | 0.135083 ms | 0.019208 ms |
| 16 | 1.313166 ms | 0.065584 ms |
| 32 | 11.611000 ms | 0.273458 ms |
| 64 | 98.498083 ms | 1.212250 ms |

The norm agrees with one within `1e-12`; there are n coordinates. A primitive
wrapper confirmed exactly n^3 local inner-product calls: 512, 4,096, 32,768,
and 262,144 respectively. The 64-site direct norm caused 2,439,023 allocation
requests and 133,692,128 requested bytes.

Mechanism: [composite contraction](../../../crates/syntaqs-core/src/evaluate/inner_product.rs)
distributes into all n^2 pairs and contracts all n factors before multiplying
their values. Off-diagonal pairs are orthogonal, but a zero overlap does not
stop the remaining factor tasks.

Agreed direction: preserve local contraction structure and stop a product branch
as soon as an exact zero makes its remaining work unnecessary. Apply this to
state inner products and operator Hilbert–Schmidt inner products. More advanced
acceleration is deferred to [future work](../../future-work.md#32-inner-product-acceleration).

### Agreed local-contraction and early-stopping scope

Use conjugate linearity on the left and linearity on the right to decompose only
the sums that a contraction needs. Contract independent tensor factors first;
do not globally distribute products of sums merely to expose term pairs.
Existing factorization and local rules remain the basis of evaluation.

Evaluate product factors incrementally. If a local overlap, local trace, or
partial product satisfies `Zero::is_zero()`, return zero for that product branch
without evaluating its remaining factors. Also skip coefficient arithmetic and
downstream contractions made unnecessary by that zero. A zero term pair does
not terminate the other branches of a sum; cancellation of a partial sum to zero
does not justify skipping its remaining summands.

The current state contractor schedules all factor contractions before multiplying
their results. Adding a zero check only to the final product reduction would
not save those contractions. The implementation must delay remaining work using
continuations or an equivalent incremental traversal. Operator HS contractions
should likewise avoid computing all local traces before deciding to stop.

For graded operators, use the existing lowering to the ordinary representation,
including parity strings, before testing local trace factors. Preserve HS
normalization and existing local multiplication order; do not infer orthogonality
from unequal primitive values or add grading signs to state inner products.
Validate spaces and original references before skipping numeric work.

Use exact zero only, without tolerance-based pruning or a new finite-value
restriction. Numerical behavior follows the existing
[numeric conventions](../../design/numerics.md); changed evaluation order does
not promise identical floating-point rounding or nonfinite-value propagation.

The initial scope excludes basis-aware sparse inner products, prefiltering term
pairs with orthogonality metadata, and partial-contraction caches. All-pairs
enumeration may remain, and zero short-circuiting alone does not generally remove
cubic scaling in the W-state case. Each W-state summand is distinct, so the
repeated-action collection in finding 2 does not itself resolve this bottleneck.

Before accepting an implementation, count local rule calls to verify that a
zero factor suppresses later factors and that other sum branches still run.
Cover state and HS products, zero factors at different positions, no-zero inputs,
retained tiny nonzero values, preserved independent tensor sums, graded parity
strings, and invalid references even when another factor is zero. Compare values
with analytic or independent small-system results and measure both zero-heavy
and no-zero cases. These checks and performance gains remain unimplemented and
unmeasured.

## 4. Sparse Hamiltonians repeatedly process spectator identities

Use the same open-chain Heisenberg model and `CartesianBasis`.

| Sites | Nonzero coordinates of H | H coordinates | Nonzero coordinates of H² | H² coordinates |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 45 | 0.213250 ms | 907 | 9.297416 ms |
| 32 | 93 | 0.931750 ms | 4,099 | 83.587791 ms |
| 64 | 189 | 3.923833 ms | 17,395 | 779.563167 ms |
| 128 | 381 | 15.413250 ms | Not measured | Not measured |

Instrumented basis-rule calls identify spectator processing:

| Expression | Sites | All basis-rule calls | Identity calls |
| --- | ---: | ---: | ---: |
| H | 16 | 720 | 630 |
| H | 32 | 2,976 | 2,790 |
| H | 64 | 12,096 | 11,718 |
| H | 128 | 48,768 | 48,006 |
| H² | 64 | 2,286,144 | 2,146,260 |

The 64-site H² coordinates caused 25,135,284 allocation requests and
4,896,265,824 requested bytes.

Mechanism: [`operator_expansion`](../../../crates/syntaqs-core/src/basis/expansion/composite.rs)
visits every ambient factor per intrinsic term and repeatedly constructs and
expands spectator identities. The source Hamiltonian does not contain those
spectator identity nodes: they are temporary local expressions created by the
`None` branch when an intrinsic term has no operator for the current factor.
Its `tensor` helper repeatedly allocates digit vectors and intermediate
`BTreeMap` values when appending factors.

The multiplicity comes from processing many intrinsic terms, not from identities
growing inside the source tree. At 64 sites H has 189 two-factor terms, each
with 62 spectators, hence 11,718 spectator identity expansions. H² has 35,721
term pairs before collection, each referring to at most four factors. The
identity-call count also includes identities obtained from local products such
as XX = I.

There is a related action cost:
[`Task::Local`](../../../crates/syntaqs-core/src/evaluate/composite.rs) visits
spectator state factors for each local action. Applying H once to the Neel state
and evaluating took 0.062708, 0.198417, 0.613792, and 2.118667 ms at 16, 32, 64,
and 128 sites. Spectators are shared in the result but still traversed.

Agreed direction: retain sparse factor data, share the identity's local expansion
within a query, and generate completed basis coordinates without factor-by-factor
intermediate maps. Routing state actions directly to affected subtrees is a
related candidate, not part of this adopted coordinate-generation scope.

### Agreed sparse coordinate-generation direction

Preserve the public output format. `Coordinates` stores a map from one local
basis-index digit per ambient factor to a coefficient:
`BTreeMap<Vec<usize>, C>`. The index tuple is the coordinate key; these are not
two separately stored index representations. This query represents an operator
in a tensor-product operator basis, rather than applying it to a state.

#### Sparse inputs and shared spectator information

`IntrinsicOpTerms` already returns only specified factors after resolving the
ordinary/graded product semantics. Preserve that sparse representation:

```text
Ambient factors: 0, 1, 2, 3
Term coefficient: 2
Specified factors: {1: X, 3: Z}
Unspecified factors: use the shared identity expansion
```

When a spectator is first needed, compute the identity's expansion and validate
its basis indices using the existing local expansion rules. Reuse that result
for the rest of the query. Do not construct an identity or invoke its basis
expansion separately for each spectator of each term. Do not eagerly expand it
when there are no spectators, including the zero-factor unit space.

Expand the specified local expressions normally. The shared spectator expansion
is private query-local data, not a persistent cache or a new public sparse
operator type.

#### Direct coordinate generation

For CartesianBasis = [I, X, Y, Z], the identity expansion is index 0 with
coefficient 1. Initialize a working key with zeros, then place the indices of
specified factors at their positions. The example emits:

```text
Default key:  [0, 0, 0, 0]
Factor 1:    [0, 1, 0, 0]
Factor 3:    [0, 1, 0, 3]
Contribution: [0, 1, 0, 3] -> 2
```

These intermediate key states are updates to working storage, not successive
maps. For multi-component specified factors, enumerate their local choices and
emit each completed key and coefficient directly into the shared result
accumulator. Combine contributions with equal keys and remove exact zeros.
Avoid a separate full-space expansion map per intrinsic term.

The single-component case must be determined from the identity's actual
expansion. Do not assume its index is zero or its coefficient is one. If
I = gamma B_q, fill spectator positions with q and include the spectator
coefficient factors in the emitted coefficient. Handling this case must not
introduce a scalar division requirement.

If identity has multiple components, such as I = E_00 + E_11 in a matrix-unit
basis, reuse the same local choices at every spectator and enumerate their
combinations incrementally with those of specified factors. Do not precompute a
full spectator tensor-product map. The final coordinate count can still grow
exponentially in the number of spectators; that is required by this explicit
output basis, not removable identity-generation overhead.

#### Output, grading, and remaining work

Run this generation after the existing intrinsic ordering and exchange-sign
processing. Preserve factor order and the signed coefficient, and do not insert
parity strings or apply crossing signs again. Spectator identity is even.

`coordinates()` emits only keys and coefficients, not identity expression nodes.
`expand()` can use the same generation scheme with shared local basis-expression
handles, then construct the required result syntax. In CartesianBasis that
syntax still contains explicit identity factors. Preserve support for other
bases where identity is represented by several basis elements, and avoid adding
Clone/Eq/Ord requirements on primitives merely for this optimization.

Remaining work is to choose the working-key and local-choice traversal layout,
resolve and reuse ambient factor positions efficiently, account for spectator
coefficients, and share generation between coordinate-only and expression
outputs. The current full-length public keys, intrinsic term count, map
comparisons, and final expression-node creation still have costs. This design
does not itself solve composite term proliferation or guarantee a particular
speedup.

Verify single-component identity at a nonzero index and with a non-unit
coefficient, multi-component identity, no-spectator inputs, unit spaces, exact
cancellation, basis errors, explicit rebinding, non-Clone primitives/indices,
and graded products/adjoints. Compare coordinates and expanded expressions with
independent small-system results. Re-run the H and H² cases with basis-call and
allocation counts to establish that repeated spectator expansion and prefix-map
construction are removed. These implementation checks and gains are not yet
measured.

## Control: output-size growth can be necessary

The product state |+>^n, with |+> = (Up + Down)/sqrt(2), has exactly 2^n nonzero
coordinates in `SzBasis`.

| Sites | Nonzero coordinates | `norm_squared()` | `coordinates()` | `expand()` |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 1,024 | 0.004000 ms | 0.246917 ms | 0.684625 ms |
| 14 | 16,384 | 0.005458 ms | 5.604708 ms | 16.361209 ms |
| 18 | 262,144 | 0.006250 ms | 150.175958 ms | Not measured |

At 18 sites, coordinates caused 1,159,949 allocation requests and 271,932,104
requested bytes. The norm agrees with one within `1e-12`.

Exponential output size is inherent here. Independent tensor-factor sums can
already be contracted without global expansion, as the fast norm illustrates.
`expand()` additionally rebuilds syntax; `coordinates()` avoids that result
tree when only coefficients are needed. Allocation improvements do not remove
the output-size lower bound.

## Integration status and limitations

All four directions in the overview are agreed, and local grouping is implemented.
The [implementation plan](implementation-plan.md) proposes their implementation
stages, order, compatibility work, and verification gates. Local grouping should
feed the existing grading path before the coordinate generator; state-term collection and inner
products must agree on the normalized state representation and preserve Act
scope. Query-local identity reuse is separate from the deferred general result
and partial-contraction caching proposals.

Further inner-product acceleration remains in
[future work](../../future-work.md#32-inner-product-acceleration).

Comparisons and operation counts identify costly paths, not percentages of CPU
time in individual functions. Results depend on representation, basis, compiler,
allocator, and host. Graded fermion evaluation, Hubbard models, other scalars,
and production application workloads were not timed.

## Historical investigation verification record

- `cargo test --offline -p syntaqs-core -p syntaqs-spin-half` passed at the
  investigated baseline, including doctests.
- Probe assertions checked exact Hadamard cancellation, exact Hamiltonian-power
  agreement, rotation agreement within `1e-10`, and W/product norms within `1e-12`.
- W primitive calls and Hamiltonian basis calls were counted separately from timing.
- After relocating the probe into this directory, its release build and `count`
  mode passed again. Formatting, local links, and dependency paths were checked.
- The initial investigation introduced no production fixes, API changes, or
  optimization regression tests. Later Stage 1 changes and checks are recorded
  [separately](stage-1-results.md).
