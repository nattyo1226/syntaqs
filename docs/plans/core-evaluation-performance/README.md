# Core evaluation and basis-expansion performance investigation

Date: 2026-11-15  
Baseline: `8b18d8b`  
Status: investigation complete; the local-grouping direction below is agreed.
Implementation has not started; the other optimization proposals remain unadopted.

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
| Agree on the local-operator grouping direction | Complete; design details remain open |

Completion criteria were met: examples have explicit definitions, measurements
have a recorded method, source paths explain the findings, and proposals are
distinguished from verified behavior. Implementation requires a separate task.

This record does not change expression contracts or provide performance
guarantees. Ordinary evaluation intentionally retains duplicate and zero terms.
See [local expressions](../../design/primitive-expression.md),
[composite expressions](../../design/composite-expression.md), and
[basis expansion](../../design/basis.md).

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

#### Intended explicit API and open details

The API should return a grouped expression without changing its input, retain
the original binding, preserve factor order, and make no further structural
changes when reapplied to an already grouped result. `group_local()` is a
working name, not an accepted public signature.

The remaining design choices are:

- Exact construction-time coverage beyond direct local sums/products, especially
  scales, adjoints, scalar identities, and empty expressions.
- Public API name, ownership convention, capability bounds, and support for
  unbound expressions. If unbound expressions are supported, ambient validation
  remains deferred and original factor references must be preserved.
- How much of the preparation helper to reuse in trace and `collect_terms_in()`.
  State action has different capability requirements and needs separate analysis.
- Whether an internal grouped marker is worthwhile to avoid rescanning explicit
  API results. Measure repeated traversal before adding persistent metadata.

Verification before accepting an implementation should compare grouped and
ungrouped coordinates, actions, traces, and adjoints in ordinary and graded
spaces; include noncommuting same-factor products, odd crossings, mixed parity,
scalar identities, rebinding, invalid references, non-Clone primitives/indices,
and deep/shared trees. Measure construction time, evaluation time, allocation
counts, and already-local inputs separately. Re-run the Hadamard case to check
that the placed-sum penalty is removed without material construction overhead.

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

Candidate improvement: interleave basis-aware actions and coordinate aggregation,
preserving the basis-free `evaluate()` contract and capability bounds.
Per-step expansion already provides a caller-side option, with the cost of
materializing basis syntax between actions.

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

Candidates include a basis-enabled sparse-coordinate inner product and partial
contraction reuse. Zero short-circuiting reduces unnecessary work but alone does
not generally remove cubic scaling for this representation. Basis-free
contraction must remain available.

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
expands spectator identities. Its `tensor` helper repeatedly allocates digit
vectors and intermediate `BTreeMap` values when appending factors.

There is a related action cost:
[`Task::Local`](../../../crates/syntaqs-core/src/evaluate/composite.rs) visits
spectator state factors for each local action. Applying H once to the Neel state
and evaluating took 0.062708, 0.198417, 0.613792, and 2.118667 ms at 16, 32, 64,
and 128 sites. Spectators are shared in the result but still traversed.

Candidates: reuse identity expansion within a query, avoid intermediate maps for
single-entry factors, reduce coordinate-prefix copying, and route local actions
to affected subtrees. Identity need not be one basis element; the generic basis
contract allows a multi-term expansion.

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

## Candidate priorities and limitations

The measured cases suggest the following tentative investigation order. The
local-grouping direction in finding 1 is agreed, but this overall order and an
integrated implementation plan have not been approved:

1. Aggregate basis-aware state actions before intermediate terms proliferate.
2. Preserve or recover same-factor structure before composite distribution.
3. Improve sparse-sum contraction, with optional basis-aware paths.
4. Reduce identity expansion, intermediate maps, and spectator traversal.

Comparisons and operation counts identify costly paths, not percentages of CPU
time in individual functions. Results depend on representation, basis, compiler,
allocator, and host. Graded fermion evaluation, Hubbard models, other scalars,
and production application workloads were not timed.

## Verification record

- `cargo test --offline -p syntaqs-core -p syntaqs-spin-half` passed at the
  investigated baseline, including doctests.
- Probe assertions checked exact Hadamard cancellation, exact Hamiltonian-power
  agreement, rotation agreement within `1e-10`, and W/product norms within `1e-12`.
- W primitive calls and Hamiltonian basis calls were counted separately from timing.
- After relocating the probe into this directory, its release build and `count`
  mode passed again. Formatting, local links, and dependency paths were checked.
- No production fixes, API changes, or optimization regression tests were introduced.
