# Stage 4: shared spectator expansion and direct operator coordinates

Date: 2026-11-16  
Status: implemented and validated; Stage 5 remains planned.  
Starting revision: `d318227eae34a105470fd01a0e8bedd06022a85a` (Stage 3).

## Implemented changes

The [operator generator](../../../crates/syntaqs-core/src/basis/expansion/operator.rs)
now separates sparse specified-factor expansions from a query-local spectator
identity expansion. The latter is initialized lazily once, shared across missing
positions/terms, and never requested for fully specified terms or the zero-factor
unit space. A zero expression with no reached intrinsic term also needs no identity.
Explicit local products that yield identity are expanded independently, without
primitive equality, identity recognition, or a general result cache.

Single-component identity I = gamma B_q initializes a reusable full-length key
with q. Specified positions are overwritten for each complete combination and
restored afterward. Gamma multiplies at each missing position in ambient order;
it is not divided out or regrouped into a precomputed power. This preserves
representable extreme coefficients that power regrouping could overflow or
underflow. Multi-component identities share their choices across spectator axes
and enumerate the necessary combinations with an iterative work stack. Products
whose choices are all singletons have a direct path without enumeration frames.

Complete contributions enter one final composite-coordinate accumulator instead
of constructing a coordinate-prefix map at every site. Local expansions still
aggregate their own basis indices. Coordinates carry no primitive payload;
expand joins shared local basis-expression handles only when a key first enters
the accumulator and then reconstructs factor-ordered syntax on the selected space.
The [shared accumulator](../../../crates/syntaqs-core/src/basis/expansion/mod.rs)
now accepts lazy payload construction; state expansion retains its original
numerical behavior and generation path.

Ordinary intrinsic terms retain encounter order, while graded terms have already
resolved crossing signs. Both are mapped to ambient positions before generating
keys. Same-factor products keep their existing order; the generator adds no
parity strings or new crossing signs. Original space/reference validation remains
before preparation. All specified local expansions and the first needed spectator
identity of a reached term are validated in ambient order before numeric zero
pruning, including empty/zero-factor contributions. Error visibility is unchanged.

No public API, coordinate format, dependency, coefficient equality/division,
primitive/index cloning, or user Ord/Hash requirement was added. The lasting
contract is [operator coordinate generation](../../design/basis.md#operator-coordinate-generation).

## Correctness evidence

Eight new [core tests](../../../crates/syntaqs-core/src/basis/expansion/operator_tests.rs)
cover:

- One spectator identity expansion per coordinates/expand query, no cross-query
  cache, and no spectator expansion for fully specified or unit-space queries.
  A fully specified term remains valid even after an earlier term cached an
  empty identity expansion.
- Identity at digit 3 with coefficient 2, differently supported terms, fully
  specified terms before/after lazy initialization, exact cancellation, and
  correct restoration of working-key digits.
- Multiple specified choices with single-component identity, multi-component
  identity in a matrix-unit basis, and fully specified branching terms mixed
  with spectator terms. Coordinates and expanded-expression round trips agree.
- Original invalid references, non-basis/out-of-bounds identity outputs, zero
  coefficients, empty explicit factors, empty identity expansion, and errors
  in later specified factors. Both public outputs retain the expected errors.
- Coefficients without PartialEq/division/Float, primitives without Clone/PartialEq,
  and indices without Clone/Hash/Ord; public bounds remain unchanged.
- Payload construction once for duplicate complete keys and shared identity
  primitive handles across expanded terms, without cloning primitive payloads.
- Ambient-order multiplication with gamma = 1e-200 or 1e200 and compensating
  large/small term coefficients, avoiding a precomputed-power numerical failure.
- Iterative enumeration of 20,000 axes on a 128 KiB stack with only two actual
  output combinations. This stress case is separate from the performance examples.

Existing independent f32/f64 ordinary/graded matrices, matrix-unit/parity bases,
nonorthogonal bases, rebindings, mixed parity, adjoints, invalid-basis and deep
expression tests pass. All 54 core unit tests pass with no skips. No existing
semantic assertions were weakened. During implementation, existing graded/matrix
coverage caught the ordinary encounter-order assumption; explicit position
mapping and reverse-order regression coverage resolved it before final checks.

## Performance comparison

The [raw record](stage-4-results.txt) includes the original 92 comparison cases
plus ten new output controls, seven batched samples per case, allocation requests
and cumulative requested bytes, source/harness hashes, starting revision and
dirty-tree status, and compiler provenance. Tests and benchmarks ran sequentially.
The Stage 0 dependency snapshot still matches the probe lock. Every measurement
and reconstruction assertion passed.

The [Stage 3 record](stage-3-results.md) is the comparison baseline. Its hashed
file snapshots match the starting commit. The original case definitions are
unchanged; the new `coordinates` mode appends output controls to `all`.

| Case | Stage 3 ms | Stage 4 ms | Stage 3 allocations | Stage 4 allocations |
| --- | ---: | ---: | ---: | ---: |
| `H n=16 coordinates` | 0.217804688 | 0.031704672 | 8,331 | 999 |
| `H n=32 coordinates` | 0.902684938 | 0.077291180 | 33,560 | 2,039 |
| `H n=64 coordinates` | 3.957479250 | 0.210429688 | 134,710 | 4,117 |
| `H n=128 coordinates` | 15.495417000 | 0.550606781 | 539,762 | 8,258 |
| `H2 n=16 coordinates` | 9.157333500 | 1.344359375 | 359,039 | 46,610 |
| `H2 n=32 coordinates` | 82.707708000 | 6.583770500 | 3,046,136 | 195,131 |
| `H2 n=64 coordinates` | 754.000542000 | 36.955084000 | 25,135,353 | 798,108 |
| `Hadamard_layer_dagger_layer n=8 local_sum` | 0.010945761 | 0.009573955 | 488 | 444 |
| `Hadamard_layer_dagger_layer n=8 placed_sum` | 0.010946818 | 0.009564738 | 488 | 444 |

At 64 sites, H coordinates improve from about 3.96 to 0.210 ms (about 18.8x),
and H² coordinates from 754 to 37.0 ms (about 20.4x). H² allocation requests drop
from 25,135,353 to 798,108; cumulative requested bytes drop from 4,896,315,840 to
90,047,752. These are cumulative allocation requests, not peak retained memory.
The coordinate counts and mathematical outputs retain the baseline definitions.

Basis-call counts directly isolate the spectator work:

| Query | Stage 3 basis calls | Stage 4 basis calls | Stage 3 identity calls | Stage 4 identity calls |
| --- | ---: | ---: | ---: | ---: |
| `H n=16` | 720 | 91 | 630 | 1 |
| `H n=32` | 2,976 | 187 | 2,790 | 1 |
| `H n=64` | 12,096 | 379 | 11,718 | 1 |
| `H n=128` | 48,768 | 763 | 48,006 | 1 |
| `H2 n=64` | 2,286,144 | 140,635 | 2,146,260 | 751 |

For unsquared H, every reached term specifies two non-identity local operators:
there are 6(n-1) explicit calls and one shared spectator call. At n=64, the old
11,718 repeated spectator calls become one. For H² at n=64, 751 identity calls
remain: one spectator expansion and 750 explicit local identities produced by
Pauli multiplication. The optimization does not classify those products as
spectators or require operator equality to reuse them. Final H² output still
contains 17,395 coordinate keys.

Already-local/placed Hadamard controls also improve slightly in this run, with
fewer prefix-map allocations. State evaluation and inner-product paths were not
changed. Their small timing/allocation variations remain host-dependent controls,
not evidence of additional optimization. Timings are host-specific medians, not
confidence intervals or universal speedup guarantees.

## Additional output controls

The new controls measure H expand at 16/32/64 sites, fully specified Pauli strings
for both outputs, and 16-site H² expand. They verify reconstruction outside the
timed region and include output destruction in measurements. Expand's unsquared
H basis-call counts match coordinates (91, 187, 379; one identity call each).

| Additional Stage 4 control | Median ms | Allocations | Cumulative requested bytes |
| --- | ---: | ---: | ---: |
| `H n=16 expand` | 0.071263832 | 2,761 | 215,920 |
| `full Pauli string n=16 coordinates` | 0.003823985 | 154 | 13,936 |
| `full Pauli string n=16 expand` | 0.005057495 | 221 | 20,920 |
| `H n=32 expand` | 0.226060547 | 8,736 | 670,064 |
| `full Pauli string n=32 coordinates` | 0.007990865 | 285 | 35,056 |
| `full Pauli string n=32 expand` | 0.010258505 | 417 | 49,208 |
| `H n=64 expand` | 0.802093750 | 30,000 | 2,279,856 |
| `full Pauli string n=64 coordinates` | 0.022212320 | 544 | 101,872 |
| `full Pauli string n=64 expand` | 0.026258709 | 805 | 130,360 |
| `H2 n=16 expand` | 2.368453125 | 93,691 | 8,432,656 |

These controls were first introduced in Stage 4 and have no prior timing baseline;
no comparative expand speedup is claimed from them. Identity choices with multiple
components are covered by semantic tests, not a separate large performance study.

## Limits and remaining work

Input term products still enumerate their intrinsic contributions. Full-length
keys and the requested expanded syntax retain unavoidable output-size costs.
A multi-component identity may genuinely require exponentially many complete
keys; sharing its local choices does not remove that output. Intrinsic traversal,
explicit primitive evaluation/local basis expansion, ambient-position lookup,
and final key copies still cost work. No general primitive/result cache, common
factor extraction, sparse public-key format, or change to state coordinates was
introduced. Stage 3's remaining inner-product costs are unaffected.

## Validation and reproduction

Completed on Rust 1.99.1, aarch64-apple-darwin:

```sh
cargo test --offline --workspace
cargo test --offline --workspace --all-features
cargo test --offline --workspace --no-default-features
cargo check --offline --workspace --all-targets --no-default-features
cargo clippy --offline --workspace --all-targets --all-features -- -D warnings
cargo build --offline --workspace --all-features
env RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --all-features --no-deps
cargo fmt --all -- --check
cargo fmt --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- --check
cargo test --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo clippy --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml --all-targets -- -D warnings
cargo build --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo run --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- all
```

All three workspace feature configurations include the final implementation and
new tests. The probe's three semantic fixtures pass. Final rustdoc includes the
updated public query documentation. Source hashes, seven-sample medians, expected
call counts, metadata/coordinate lengths, local documentation links/anchors/tables,
formatting and final diffs were checked separately.

Rust 1.85, dedicated no_std targets, alternative platforms/native providers and
every standalone backend feature combination were not verified. Stage 5 still
owns combined acceptance and any remaining supported-toolchain/provider checks;
this record does not mark that stage complete.
