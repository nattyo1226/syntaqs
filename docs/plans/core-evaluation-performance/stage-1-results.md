# Stage 1: local operator grouping

Status: implemented and validated; Stages 2–5 remain pending.
Starting revision: `aeda4685278baa00ba1bc4a0e8de87c0aae22b17`.

## Implemented changes

- Operator sum/product constructors group direct local operands on the same
  indexed factor after existing binding checks. Constructors gain no new bounds.
- `Operator::group_local(&self) -> Self` is available with `I: Eq`, `O: Identity`,
  and `C: Clone`. It preserves binding and accepts unbound operators without
  primitive equality, arithmetic, Product, Adjoint, grading, or Dimension bounds.
- Iterative whole-expression grouping classifies scalar-only, single-factor,
  and multi-factor subtrees. Scalars are converted to local syntax only when
  used in a single-factor context, sharing converted results across occurrences.
  Pure scalar roots and unchanged multi-factor paths retain their source nodes.
- Operator expand/coordinates, collect_terms_in, full trace, and HS preparation
  use the same grouping helper after validating original references.
- Same-factor construction now retains local syntax, so display can change from
  `A[i] + B[i]` to `(A + B)[i]`. Formatting assertions were updated accordingly.

The implementation lives in
[expr/group.rs](../../../crates/syntaqs-core/src/expr/group.rs).
No state normalization, early stopping, identity-expansion cache, persistent
support metadata, or change to public Coordinates was introduced.

## Avoiding unnecessary preparation work

Already-local and identity roots return directly. A bounded candidate scan avoids
full memo allocation for small expressions that cannot need a rewrite. After
256 node occurrences it conservatively falls back to memoized postorder, so
shared DAGs cannot cause exponential preflight traversal. Traversal uses borrowed
child iterators, reads leaf classifications directly, and stores replacements
only for changed containers. Memo keys are original node addresses, not user
primitive/index hashes.

An initial version allocated temporary child vectors and retained unchanged
handles in the memo. A simple 32-factor HS control exposed substantial overhead.
Those allocations and unnecessary clones were removed, and the bounded scan
reduces the final control to two additional allocation requests in the recorded
run. No persistent grouping flag was added to solve this regression.

## Correctness evidence

Five new core unit tests cover:

- Direct construction with opaque coefficients and primitives lacking Identity;
  same-index factors on distinct handles; operand order and original binding checks.
- Explicit grouping with Clone-only coefficients and non-Clone indices/primitives;
  scalar contexts, unbound/bound results, stable reapplication, and zero products
  retaining invalid references for validation.
- Shared scalar/single-factor subtrees: the coefficient is cloned once across
  shared uses, and grouped handles remain shared.
- Grouping and destruction of 20,000 nested local, scalar, and multi-factor nodes
  on a small thread stack.
- A shared product DAG with exponentially many occurrence paths: preparation
  remains bounded/memoized and returns the unchanged root.

Two new integration tests cover f32 and f64, each with 216 combinations of
ordinary/graded kind, placement position, primitive pairs, complex coefficients,
scalar identity, mixed parity, ordered products, and adjoints. Original, grouped,
and regrouped operators, automatic expansion, trace, and HS results agree with
independent ordinary embedding matrices. Additional cases check separated odd
factors, explicit rebinding to a different factor order, and invalid references.
The fixture primitives deliberately lack PartialEq, checking that basis/trace/HS
queries acquire no operator-equality requirement.

Existing collection, malformed-basis, graded scope, non-Clone capability,
normalization, and export integration tests also pass. State evaluation retains
its old bounds and duplicate-preserving behavior until Stage 2.

## Performance record

The [raw results](stage-1-results.txt) include source/probe hashes, dirty-tree
provenance on the starting revision, and all seven timing samples for 92 cases.
The same batched method as [Stage 0](stage-0-baseline.md) was used. Tests and
benchmarks were run sequentially; timings are host-specific and not confidence
intervals. Allocation bytes are cumulative requests, not peak memory.

| Case | Stage 0 ms | Stage 1 ms | Stage 0 allocations | Stage 1 allocations |
| --- | ---: | ---: | ---: | ---: |
| `Hadamard_layer_dagger_layer n=4 placed_sum` | 0.517667969 | 0.005566101 | 21,844 | 252 |
| `Hadamard_layer_dagger_layer n=6 placed_sum` | 12.077125000 | 0.008289896 | 504,166 | 371 |
| `Hadamard_layer_dagger_layer n=8 placed_sum` | 251.076000000 | 0.011048258 | 10,555,044 | 488 |
| `Hadamard_layer_dagger_layer n=8 local_sum` | 0.010321248 | 0.011053142 | 481 | 488 |
| `construction n=8 placed_sum=true` | 0.001974091 | 0.002059703 | 105 | 113 |
| `construction n=128 placed_sum=false` | 0.016528564 | 0.017422648 | 521 | 521 |
| `construction n=128 placed_sum=true` | 0.045052570 | 0.040526043 | 1,545 | 1,673 |
| `operator n=32 hs zero_at=None` | 0.013346517 | 0.014011189 | 626 | 628 |
| `H n=64 coordinates` | 3.930281250 | 3.979948000 | 134,643 | 134,715 |
| `H2 n=64 coordinates` | 752.744500000 | 757.960917000 | 25,135,284 | 25,135,358 |

The eight-site placed-sum case now has the same work scale as its local-sum
counterpart, instead of distributing 4^8 composite branches. The small control
changes expose preparation overhead and ordinary run variation; they do not
establish general speedups. In the 128-site placed-construction case, allocation
requests increase by one per grouped local sum, while retained graph size and
requested bytes decrease. Already-local construction has unchanged allocation
counts and a small factor-checking cost.

The added `grouping` probe mode separately tests direct versus deferred grouping:

| Style at n=8 | Coordinates ms | Explicit grouping ms |
| --- | ---: | ---: |
| local | 0.011050781 | 0.000464221 |
| direct | 0.011043131 | 0.000468825 |
| deferred | 0.017964559 | 0.003504507 |

`deferred` wraps each separately placed primitive in a composite unit scale,
so construction cannot use the direct-child rule. Query-time preparation still
recovers local sums. All three paths yield exactly one all-identity coordinate
with coefficient one. Explicit grouping time is shown separately from query time.

The existing rotation/H-power evaluation allocation counts, W-state local
inner-product counts, and H/H² identity basis-call counts remain at their Stage 0
levels: the other three optimizations have not been implemented.

Reproduce with the retained Stage 0 dependency snapshot and current source:

```sh
cargo test --offline -p syntaqs-core --lib group_tests
cargo test --offline -p syntaqs-tests --test core_tensor_products local_grouping
cargo build --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo run --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- all
```

`grouping` is available as a separate probe mode; `all` includes it after the
original Stage 0 cases. Historical raw baseline files remain unchanged.

## Validation completed and limits

- Core unit/integration tests passed across the targeted runs, including the
  unchanged slow local-expression stress test. The final repeated core command
  used `--tests -- --skip deep_action_product_scale_sum_and_adjoint_chains_use_work_stacks`
  because that test had already passed during the initial full run. Initial
  structure/format expectation failures were updated for the accepted local
  construction shape; their final regression runs pass.
- All default-feature syntaqs-tests integration tests and syntaqs-spin-half tests
  passed; core and spin-half doctests passed, including the new public API example.
- Workspace all-target checks passed with both no default features and all features.
- Workspace all-target no-default-feature Clippy passed with warnings denied.
- The plan-local probe's semantic tests and the full measurement assertions passed.
- Core rustdoc built with warnings denied. Formatting, local documentation links,
  source hashes/sample medians, and final diff checks passed.

Validation used Rust 1.99.1 on aarch64-apple-darwin. Rust 1.85 is not installed in
this environment, so a minimum-version build is not claimed. The new code avoids
let chains and preserves the declared minimum version. Full workspace runtime
checks for every optional/native backend remain Stage 5 work; successful
all-feature type checking is not a native-backend runtime test.
