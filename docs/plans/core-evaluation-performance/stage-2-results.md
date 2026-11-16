# Stage 2: natural-state collection between actions

Date: 2026-11-16  
Status: implemented and validated; Stages 3–5 remain planned.  
Starting revision: `a7e8d452c27089308b7c6b462fe20f30b90775c2` (Stage 1).

## Implemented behavior

Local and composite `evaluate()` now collect matching natural-state terms
between successive actions and normalize action-free inputs. Products still act
right to left, including adjoint reversal. Operator-sum branches feed one
incremental accumulator from the same normalized input.

[Local evaluation](../../../crates/syntaqs-core/src/evaluate/local.rs) carries
shared normalized primitive polynomials rather than rebuilding an uncollected
state tree between actions. Primitive matching uses a linear PartialEq list,
retains first-occurrence order, combines coefficients, and removes exact zeros.
Scaling a normalized result does not repeat primitive matching.

[Composite shapes](../../../crates/syntaqs-core/src/evaluate/state_terms.rs)
separate an outer coefficient from unit, placed local polynomial, ordered tensor,
and retained-sum structure. Tensor parentheses/units and singleton coefficient
positions are normalized. Single-factor sums share the local polynomial form;
scales of multi-term local polynomials are absorbed into their coefficients.
A zero factor annihilates a tensor while the public state keeps its selected
space. There is no numerical common-factor extraction or distributive matching:
products of sums remain factored.

Equality-only primitive/coefficient interning supplies scratch IDs for structural
BTreeMap lookup. Output order is independent of lookup order. Nonreflexive user
values do not gain equality from shared pointers or IDs. A per-call source-node
memo avoids repeated normalization of original shared subtrees, and evaluator
tasks pass normalized wrappers onward without recollecting action results.
Local memo entries retain their source handles to prevent pointer-address reuse
when several placed actions share one evaluator context. Traversals and final
node destruction remain iterative.

Primitive/unit roots and flat products of placed primitives have an intrinsically
trivial normal form and can share their original expression after inspecting
their direct children. There is no persistent collected flag, nor reinterpretation
of has_action/pending_action as such a flag. Other action-free inputs are scanned
again on each query; no O(1) re-evaluation guarantee is made. Composite basis
expansion reuses already evaluated local states without a second local evaluation.

## Capabilities and validation boundary

Local evaluation requires `S: PartialEq + Act`, `O: Adjoint`, and coefficient
Clone/Zero/One/conjugation. Composite evaluation also needs `I: Eq`, coefficient
PartialEq, and the existing operator parity capabilities. Local coefficients need
no PartialEq. No primitive/index Clone, primitive Eq/Ord/Hash, coefficient ordering,
or operator Product was added. Storage/construction bounds are unchanged.

These intentional evaluation bounds propagate through core inner products,
norms, expectations, state basis queries, normalize extension traits, and
vector/action-matrix export. Operator-only basis, trace, and HS queries retain
their bounds. The lasting contract is in
[natural-state collection](../../design/composite-expression.md#92-natural-state-collection-between-actions).

Implementation exposed one consequence of the accepted `evaluate -> basis
expansion -> accumulation` sequence: a primitive removed by exact-zero collection
no longer reaches the basis and cannot report a malformed expansion. This also
applies when a zero state factor annihilates a tensor containing another primitive.
The historical state-zero assertions in core/export basis tests were changed
accordingly; retained-state expansion, operator-zero expansion, enumeration,
and original space/reference checks remain tested. This explicitly clarifies
the earlier broad Stage 0 basis-error wording. Stage 4 must still preserve
validation of reached **operator** intrinsic terms.

Floating-point summation order can change. The old large-superposition norm
fixture now has an exactly real denominator after primitive collection, so its
assertion of a nonzero imaginary rounding residual was removed. Finite normalized
expectations and the independent extreme complex-division regressions remain
covered; no numerical tolerance or special-value rejection was added.

## Correctness evidence

Six new [core tests](../../../crates/syntaqs-core/src/evaluate/state_tests.rs) cover:

- 100 successive branching local actions: two retained primitives and exactly
  199 primitive actions; a shared delayed input runs its rule once.
- Tensor grouping, scalar placement, units, same-factor placement/sum equivalence,
  first-occurrence order, exact cancellation, tiny nonzero terms, and space retention.
- A 32-factor product of two-term local sums retaining 64 primitive leaves;
  factored and distributed multi-factor states remain distinct shapes.
- Shared nonreflexive primitives and retained NaN coefficients remaining distinct.
- Local and composite 64-level shared additive DAGs with 2^64 occurrence paths:
  memoized normalization and one subsequent primitive action, without expansion.

Two new [integration tests](../../../tests/tests/core_state_collection.rs),
one per precision, compare up to 128 rotations with analytic sine/cosine values
and local ordered-product application. Eight successive six-site Heisenberg
applications match an independent bit-state reference; the half-filled sector
has at most 20 nonzero coordinates. Existing independent ordinary/graded matrix,
Act-scope, nonorthogonal-basis, export, normalization, and deep-tree tests pass.
Capability fixtures now exercise the approved bounds, including local
coefficients lacking equality and operators lacking Product.

## Performance

The [raw record](stage-2-results.txt) contains all 92 cases, seven batched timing
samples per case, source/harness hashes, starting-revision/dirty-tree metadata,
and toolchain information. The retained dependency snapshot matches the probe
lock. Tests and measurements ran sequentially. Compare with
[Stage 1](stage-1-results.md); historical records remain unchanged.

| Case | Stage 1 ms | Stage 2 ms | Stage 1 allocations | Stage 2 allocations |
| --- | ---: | ---: | ---: | ---: |
| `rotation k=16 evaluate` | 107.029583000 | 0.005378153 | 3,866,499 | 240 |
| `rotation k=16 coordinates` | 113.720708000 | 0.005581645 | 3,866,509 | 247 |
| `Hpow Neel n=6 k=1 evaluate` | 0.014161743 | 0.021149982 | 452 | 699 |
| `Hpow Neel n=6 k=4 evaluate` | 56.680000000 | 0.895786437 | 1,788,119 | 23,697 |
| `Hpow Neel n=6 k=4 coordinates` | 141.952791000 | 0.754755187 | 4,880,862 | 24,595 |
| `ready Neel n=64 evaluate` | 0.000032301 | 0.000044094 | 2 | 0 |
| `Neel n=64 one_site_action` | 0.002161067 | 0.023010580 | 35 | 681 |
| `ready local duplicate_sum n=128 evaluate` | 0.000029302 | 0.008677510 | 2 | 289 |
| `H n=64 evaluate_on_Neel` | 0.630682312 | 0.855838563 | 6,839 | 11,063 |
| `plus n=18 norm_squared` | 0.006061727 | 0.020349529 | 266 | 737 |
| `W n=32 norm_squared` | 11.663291000 | 7.738125000 | 312,006 | 329,895 |
| `Hadamard_layer_dagger_layer n=8 placed_sum` | 0.011048258 | 0.010972168 | 488 | 488 |
| `H2 n=64 coordinates` | 757.960917000 | 774.564959000 | 25,135,358 | 25,135,351 |

The repeated-action cases avoid exponential growth in contribution paths. At
16 rotations, allocation requests fall from 3,866,499 to 240 and cumulative
requested bytes from 199,228,480 to 20,984. Four Heisenberg actions use 23,697
allocation requests rather than 1,788,119. No basis is used for this collection.

The H^4 evaluate-only sample has noticeable host variation (roughly 0.72–1.09 ms),
so its recorded median exceeds the separate coordinates median. These are
host-specific samples, not confidence intervals; no faster run was substituted.

Controls expose real costs. A single action on a 64-site product state still
prepares a query-local matching context and is about ten times slower in this
run (roughly 2.2 to 23 microseconds). One Hamiltonian action and factored-state
norms can also become slower. A ready 128-term duplicate sum now actually collects
its terms, whereas the old evaluator returned it unchanged. The primitive-product
fast path removes allocations for ready Neel inputs, but does not certify general
ready inputs or eliminate preparation for pending actions. Retaining the agreed
per-call normalization contract has these costs; this stage does not claim a
universal speedup.

W-state primitive inner-product calls remain n^3 (262,144 at n=64): exact-zero
contraction stopping is still Stage 3. H/H² basis/identity call counts also retain
the old work pattern; shared spectator expansion is still Stage 4. Small changes
in their elapsed times/allocation requests do not establish those optimizations.

## Validation and reproduction

The following completed on Rust 1.99.1, aarch64-apple-darwin:

```sh
cargo test --offline --workspace
cargo test --offline --workspace --all-features
cargo test --offline --workspace --no-default-features
cargo check --offline --workspace --all-targets --all-features
cargo check --offline --workspace --all-targets --no-default-features
cargo clippy --offline --workspace --all-targets --all-features -- -D warnings
cargo build --offline --workspace --all-features
env RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --all-features --no-deps
cargo fmt --all -- --check
cargo test --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo build --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo run --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- all
```

The final all/no-default-feature runs include all 40 core unit tests, with no
stress-test skips. The default run passed before the last shared-DAG test was
added; that test passed in both later configurations. The probe's three semantic
fixtures and every measurement assertion passed. Formatting, documentation links,
source hashes, recorded medians, and final diffs were checked separately.

Rust 1.85 was not tested. Host feature configurations do not establish dedicated
no_std-target, other-platform, alternative native-provider, or every independent
backend-feature combination. Those broader checks and combined-stage acceptance
remain Stage 5 work. No changes for Stages 3 or 4 are claimed here.
