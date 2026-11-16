# Integrated core evaluation performance plan

Status: Stages 0–2 complete; Stages 3–5 remain planned. Stage 0 fixed the
contracts and recorded an audited pre-optimization baseline. Stage 1 implemented
local grouping; its [results and validation](stage-1-results.md) are recorded.
Stage 2 [results and validation](stage-2-results.md) cover per-action collection
and its compatibility/performance costs.

## 1. Objective and boundaries

Reduce intermediate expression growth and redundant local work without changing
the mathematical meanings of actions, inner products, or basis coordinates.
The [investigation](README.md) owns baseline measurements and the detailed agreed
scope. This document owns the combined change map, dependencies, stages, and
completion criteria.

The four workstreams are complementary:

| Workstream | Required change | Intended effect |
| --- | --- | --- |
| Local-operator grouping | Group direct same-factor operands during construction; recover deeper single-factor subtrees before evaluation; expose an explicit grouping API using the same rules | Avoid distributing local sums across the composite expression |
| State-term collection | Normalize and collect matching natural-state terms inside `evaluate()` between actions | Bound repeated contributions by retained state structure rather than action paths |
| Incremental inner products | Evaluate local product factors on demand and stop branches on exact zeros | Avoid remaining local contractions and arithmetic after a zero is established |
| Sparse coordinate generation | Share spectator identity expansion and emit completed keys directly from sparse factor data | Remove repeated spectator expansion and factor-by-factor intermediate maps |

No general distributive-equivalence search, multi-factor common-factor
extraction, approximate pruning, new basis-aware inner-product API, orthogonality
metadata, persistent result cache, or public coordinate-format change is in
scope. Further inner-product acceleration remains
[future work](../../future-work.md#32-inner-product-acceleration).

## 2. Processing boundaries and shared responsibilities

### Operator construction and basis queries

```text
construction:
    binding checks -> cheap same-factor grouping of direct local children

explicit grouping API:
    original expression -> iterative single-factor subtree grouping -> expression

operator basis query:
    select space and validate original references
    -> single-factor subtree grouping
    -> existing intrinsic traversal and graded crossing signs
    -> expand specified local expressions; reuse spectator identity expansion
    -> emit completed coordinate contributions into one result accumulator
    -> Coordinates, or reconstruct expanded expression syntax
```

Grouping owns representation changes only. It does not run primitive product
rules, select a basis, or compute graded crossing signs. Share its local rewrite
rules between construction and whole-expression traversal. Evaluate-time use
must not create a second normalizer with different semantics.

Coordinate generation consumes the signed intrinsic factors. It must not insert
parity strings, recompute crossing signs, or reorder local products.
Query-local identity expansion is a basis-query detail, separate from general
expression caching.

### State evaluation and inner products

```text
state evaluation:
    action -> normalize/collect natural-state terms -> next action -> ...

state coordinates:
    evaluate -> requested basis expansion -> basis-index accumulation

state inner product:
    validate compatible spaces -> evaluate operands
    -> align independent tensor groups
    -> incrementally contract groups, distributing sums only where needed

operator HS inner product:
    validate space -> existing ordinary lowering with parity strings
    -> incrementally multiply normalized local trace contributions
```

Share natural-state normalization and term matching between local/composite
evaluators where their semantics coincide. Do not force state collection,
operator grouping, and basis-index aggregation into one universal expression
representation: they have different equality rules and capability requirements.
The existing exact-zero basis accumulator can remain a common numerical helper.

## 3. Change map and compatibility

| Area | Main source owners | Planned work |
| --- | --- | --- |
| Operator structure | [composite expressions](../../../crates/syntaqs-core/src/expr/composite.rs), [local expressions](../../../crates/syntaqs-core/src/expr/local.rs), [arithmetic](../../../crates/syntaqs-core/src/expr/arithmetic.rs) | Shared same-factor rewrite rules, cheap construction integration, explicit grouping API, iterative preparation |
| Traversal and grading | [traversal](../../../crates/syntaqs-core/src/expr/traversal.rs) | Feed prepared expressions into existing traversal; preserve ungrouped graded placement order until signs are resolved |
| State evaluation | [local evaluator](../../../crates/syntaqs-core/src/evaluate/local.rs), [composite evaluator](../../../crates/syntaqs-core/src/evaluate/composite.rs) | Per-action collection, incremental sum accumulation, ordered product continuations, normalized-result reuse |
| Scalar contractions | [inner products](../../../crates/syntaqs-core/src/evaluate/inner_product.rs), [trace and HS](../../../crates/syntaqs-core/src/evaluate/trace.rs) | Replace eager product scheduling with on-demand continuations; preserve sum branches and HS normalization |
| Basis queries | [composite expansion](../../../crates/syntaqs-core/src/basis/expansion/composite.rs), [local expansion](../../../crates/syntaqs-core/src/basis/expansion/local.rs), [accumulation](../../../crates/syntaqs-core/src/basis/expansion/mod.rs) | Query-local identity expansion, working keys and local-choice traversal, direct result accumulation, expression-output payload sharing |
| Existing exact operator collection | [collect](../../../crates/syntaqs-core/src/evaluate/collect.rs) | Keep primitive collection distinct from structural grouping; reuse preparation where compatible without expanding its agreed scope |

New private helper modules are acceptable if needed, but module names and exact
placement should follow the final responsibilities rather than duplicate these
operations across callers.

### Capability changes

State evaluation will require primitive `PartialEq` and coefficient operations
needed for collection. Coefficient `PartialEq` is accepted where evaluation
compares retained factored structures, but not on construction or unrelated
operations. Stage 0 fixed the evaluator bounds, implicit-one treatment,
normalization rules, and query-local collection index in
[the implementation contracts](stage-0-contracts.md).
Local collection has fewer bounds than composite structural matching. Stage 2
implements these bounds and propagates them through core queries, normalize,
and vector/action-matrix export.
Do not impose Eq/Ord/Hash or primitive/index Clone merely to implement collection.

Propagate intentional changes through state inner products, norms, expectations,
basis queries, and callers in `syntaqs-normalize`, `syntaqs-export`, examples,
and tests. Review `syntaqs-lie` for effects of operator grouping and HS changes.
Keep action evaluation independent of primitive operator Product: ordered
products act by successive applications.

Construction must retain its lightweight storage capabilities. Gate rewrites
requiring unavailable capabilities, especially scalar-identity handling, to
evaluation preparation or the explicit API rather than strengthening all
constructors. The operator coordinate optimization itself must not add
primitive equality or scalar division requirements.

### Invariants across all workstreams

- Preserve same-factor product order, ambient factor order, adjoint reversal and
  conjugation, delayed Act scope, and ordinary/graded mathematical meaning.
- Validate original spaces/references before numeric zero shortcuts can hide
  them. Construction grouping must preserve references needed by later validation.
- Remove numerical contributions only through exact `Zero::is_zero()`. Preserve
  tiny nonzero values; floating-point rounding and nonfinite propagation follow
  [numeric conventions](../../design/numerics.md), not bitwise baseline equality.
- State basis queries expand the evaluated result. Exact-zero terms removed by
  Stage 2 no longer run basis rules. Retained state primitives, original space
  references, matrix-basis enumeration, and reached operator-coordinate terms
  still undergo their corresponding validation.
- Keep independent products of sums factored. Normalize tensor parentheses and
  coefficient placement internally without permuting factors or replacing the
  selected space.
- Use iterative traversals and shared immutable handles. Preserve deep-tree
  processing, final-owner destruction, and non-Clone primitive/index support.
- Keep `Coordinates` as full-length basis-index keys and coefficients. Materialize
  only the expression syntax required by `expand()`.

## 4. Implementation stages

The suggested order is grouping, state collection, incremental contraction,
then coordinate generation. Grouping establishes the operator preparation
boundary used by coordinate generation. Collection establishes the state
structure consumed by contractions. Contraction and coordinate generation are
otherwise independent; the order supports review and measurement attribution.

| Stage | Deliverable | Completion gate | Current status |
| --- | --- | --- | --- |
| 0. Contracts and baseline | Exact API/bounds and collector rules; repeatable benchmark inputs | Contracts fixed; 74 measured cases and 19 existing integration tests plus 3 new semantic fixtures recorded at a070071 | Complete |
| 1. Local grouping | Shared rewrites, construction integration, explicit API, operator-query preparation | Same-factor equivalence and graded-order tests pass; Hadamard placed sums avoid the old global distribution | Complete; [results](stage-1-results.md) |
| 2. State collection | Local/composite per-action collection and caller-bound migration | Rotations and Hamiltonian powers agree with references; matching normalization and factored-input tests pass | Complete; [results](stage-2-results.md) |
| 3. Inner-product stopping | On-demand product contraction and exact-zero propagation | Call-count tests prove later factors are skipped while other sum branches run; graded HS remains correct | Planned |
| 4. Coordinate generation | Shared spectator expansion and direct key emission for both outputs | Single/multi-component identity cases pass; spectator basis calls no longer scale with terms times spectators | Planned |
| 5. Integration and documentation | Combined regressions, performance comparison, current API docs | Required workspace checks pass or pre-existing/environment failures are identified; acceptance criteria below are met | Planned |

### Stage 0: completed contracts and baseline

The [Stage 0 contracts](stage-0-contracts.md) fix API/bounds, direct-child
construction rewrites, scalar/empty-expression handling, the normalized
state-term representation, first-occurrence matching order, and query-local
interning/memoization. No persistent collected/grouped flag is introduced in the
initial implementation. Action-free inputs are collected too; re-evaluation
has stable shape but is not promised O(1).

The [baseline record](stage-0-baseline.md), [raw samples](stage-0-baseline.txt),
and [dependency snapshot](stage-0-probe.lock) identify the production starting
revision and the audited harness. The new controls measure construction, ready
inputs, low-duplicate actions, nonzero contractions, and operator HS products.
Independent semantic fixtures cover normalization examples, Heisenberg powers,
and nontrivial identity expansions. Existing graded and malformed-basis
integration tests passed.

Stages 3–4 must implement the remaining fixed contracts and add structural/call-count
assertions that require those implementations. Stage 0 does not claim new
production behavior. Historical and Stage 0 timings are machine-specific
references, not universal thresholds.

### Stages 1–2: prevent avoidable intermediate terms

Group only single-factor subtrees and keep construction rewrites cheap. Apply
automatic preparation once per operator query after validation. Reuse the same
rules in the explicit API, preserving binding and making repeated grouping
structurally stable.

For state sums, use one collector for contributions from the same input.
For products AB, collect B's result before applying A. Enforce this inside
local actions as well as composite actions. Normalize scales and tensor grouping
before comparing matching terms; retain product-of-sums nodes instead of
distributing solely for equality. Update capability tests and dependent APIs
in the same stage as their implementation changes.

### Stages 3–4: reduce per-branch work

Represent contraction products with a running value and the remaining work.
Schedule the next factor only after the current result is known to be nonzero.
Do not stop a sum when one summand or its partial sum is zero.
Preserve the existing separation of intrinsic graded coordinates from ordinary
parity-string lowering for traces and HS products.

For coordinates, keep sparse specified-factor data after intrinsic ordering.
Initialize spectator expansion lazily once per query, preserving index checks.
For a single-component identity I = gamma B_q, prepare default digits q and
account for gamma at unspecified factors without requiring division. Enumerate
specified local choices into working key storage, emitting each complete
contribution directly into the result accumulator. For multi-component identity,
share its choices and enumerate required combinations incrementally without
building a full spectator map. Reuse local expression handles for `expand()`.

## 5. Verification and performance acceptance

Use independent small matrices and analytic values for semantics, and operation
counts for work avoidance. The existing [verification guide](../../design/verification.md)
owns the required command matrix; this plan adds targeted checks, not alternative
project-wide requirements.

| Workstream | Required targeted evidence |
| --- | --- |
| Grouping | Same-factor noncommuting products; mixed parity and three odd crossings; adjoints; explicit rebinding; invalid references; grouping idempotence; opaque construction coefficients |
| State collection | Local and composite repeated actions; action-free duplicate sums and re-evaluation stability; exact cancellation and tiny coefficients; equivalent tensor grouping/scale placement; products of sums remaining factored; no primitive Product requirement; ready/shared/deep inputs |
| Inner products | Zero at early/middle/late factors; counters for skipped work; nonzero and other sum branches; complex conjugation; graded Act scope; parity-string HS products; normalization without total-dimension conversion |
| Coordinates | Identity at index 0 and other indices; non-unit single-component coefficients; multi-component identity; no spectators/unit space; basis errors; graded products/adjoints; coordinates and expression outputs with non-Clone primitives |

After each stage, run appropriate core tests and directly affected integration
tests. Broaden checks when bounds or behavior reach consumers. The final gate
uses the workspace default/all/no-default-feature tests, all-feature build,
all-target no-default-feature check, formatting, Clippy and rustdoc checks, and
the supported toolchains/backends specified in the verification guide.
Record any unavailable toolchain/native-provider checks explicitly; never infer
them from a smaller passing test run.

Repeat the benchmark cases separately from tests and other benchmark processes:

| Case | Improvement to demonstrate | Regression control |
| --- | --- | --- |
| Hadamard U†U, local versus placed sums | Placed construction avoids composite sum proliferation | Construction time and already-local expressions |
| Repeated rotations and six-site H powers | Intermediate matching terms are collected before the next action | Low-duplicate states and evaluation with no pending actions |
| W-state norm and operator HS products | Exact-zero branches skip later local calls | Include operand evaluation/collection cost and no-zero products |
| H/H² operator coordinates | One spectator identity expansion per query when needed; fewer allocations and no prefix maps | Multi-component identities and `expand()` output reconstruction |
| Product-state sums | Factored norm evaluation stays inexpensive | Genuine 2^n coordinate output remains correct |

Primitive PartialEq matching can itself be costly when most terms are distinct.
W-state end-to-end timing must include normalization costs, not just contraction
after pre-processing. Full-length coordinate keys and final expression output
also retain unavoidable costs. Do not claim that these changes remove all
exponential growth or make W-state contraction asymptotically subcubic.

Record elapsed time, allocation requests/cumulative bytes, and relevant local
rule counts. Show each stage's results and the combined result. Require the
predicted work reductions and investigate material regressions in control cases;
numeric speedup targets should be established from the repeated current baseline,
not invented from the historical sample.

## 6. Completion and documentation

The work is complete when all four stages are implemented within their agreed
scope, capability changes are reflected in consumers, semantic/call-count
regressions pass, and integrated performance results are recorded with limits.

During implementation, update the public rustdoc and relevant permanent designs
for [local expressions](../../design/primitive-expression.md),
[composite expressions](../../design/composite-expression.md),
[basis expansion](../../design/basis.md), and
[verification](../../design/verification.md) to reflect the implemented contracts.
Keep deferred optimizations in future work and dated measurements in this task.
Close out the plan only after implementation, validation, and review are complete.

Stage 0 test and measurement outcomes are recorded separately in its baseline
record. Stage 1 implements the grouping API, shared construction rules, and
operator-query preparation; its results include semantic checks and 92 measured
cases. Stage 2 implements per-action natural-state collection and caller-bound
migration; see its [implementation record](stage-2-results.md). Stages 3–5 remain pending.
