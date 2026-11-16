# Stage 3: incremental local contractions and exact-zero stopping

Date: 2026-11-16  
Status: implemented and validated; Stages 4–5 remain planned.  
Starting revision: `5264dbcd1c4f0f5edb7d4df7a8708c3803620522` (Stage 2).

## Implemented changes

The [state contractor](../../../crates/syntaqs-core/src/evaluate/inner_product.rs)
now carries the running product and remaining independent groups in a
continuation. It schedules one group at a time and discards the remaining
contractions when a local value or the running product is exactly zero.
Pending coefficient scales skip cloning/conjugation/multiplication after a zero;
explicit zero scales can stop contraction before primitive overlaps. Sum
reduction omits zero contributions but always considers remaining summands,
even when the partial sum is zero. Existing tensor alignment and distribution
inside inseparable groups are retained, including overlapping subsystem boundaries.

Local primitive overlaps skip downstream coefficient multiplication, left
conjugation, and addition on zero. A multiplication that underflows to zero also
skips the remaining arithmetic. This uses only Zero and existing operations;
inner-product helpers acquire no coefficient PartialEq requirement.

[Trace and HS](../../../crates/syntaqs-core/src/evaluate/trace.rs) contract local
factors in selected ambient order and stop the current tensor term on an exact
zero. Other tensor terms continue. Zero primitive traces skip HS normalization
division, coefficient multiplication and addition; nonzero primitive traces
still normalize by Tr(I) before coefficient multiplication/summation. The separate
identity trace establishing the local dimension is retained. Ordinary lowering
with graded parity strings and all original reference/space checks are unchanged.

No public API, capability-bound, basis/coordinate-format, primitive algebra,
state collection, or basis-query spectator-identity generation changes were introduced.
There is no orthogonality metadata, term-pair prefilter, or numeric result cache.
The current permanent contract is
[incremental scalar contractions](../../design/composite-expression.md#93-incremental-scalar-contractions).

## Correctness evidence

Six new [core tests](../../../crates/syntaqs-core/src/evaluate/stopping_tests.rs)
cover:

- State products with zero at the first/middle/last factor and nonzero controls;
  underflowed running products stop, tiny nonzero products and NaN-only products
  continue. Factor order differs from numeric index order.
- Zero summands, partial sums that cancel then become nonzero, and an inseparable
  two-site sum whose zero result suppresses an independent trailing contraction.
  Distinct natural primitives have nonzero overlaps, so unequal primitives are
  never assumed orthogonal.
- Counters proving zero contractions skip remaining coefficient multiplication,
  conjugation and addition, using a coefficient without PartialEq/Copy/Num/Float.
- Ordinary and graded HS products with early/middle/late zeros, counters for
  visited local traces and normalization divisions, and operators without
  Clone/PartialEq/Dimension. The graded fixture uses homogeneous real 2x2
  operators with matching parities so the intended zero position survives
  ordinary lowering; independent matrix tests cover general parity patterns.
- Trace/HS sums continuing through zero tensor terms and zero partial sums;
  invalid original references and incompatible state spaces remain errors.
- Direct contraction and destruction of a 20,000-level action-free source on a
  128 KiB stack. This stress input is separate from the realistic performance cases.

Existing independent ordinary/graded matrices, nonorthogonal Gram matrices,
normalization, extreme normalized-HS scales, Act scope, 2048-factor identities,
allocation and deep-expression tests pass at both precisions. All 46 core unit
tests pass with no skips. No existing test expectations were weakened.

## Performance measurements

The [raw record](stage-3-results.txt) retains the unchanged 92-case probe with
seven batched timing samples, cumulative allocation requests/bytes, source and
harness hashes, revision/dirty-tree metadata, and compiler information. No tests
or other benchmarks ran concurrently. The dependency snapshot still matches the
probe lock. Every measurement assertion passed.

The [Stage 2 record](stage-2-results.md) is the comparison baseline. Its 31 hashed
file snapshots were verified against the Stage 3 starting commit, so the compared
source/harness is identified independently of the elapsed-time samples.

| Case | Stage 2 ms | Stage 3 ms | Stage 2 allocations | Stage 3 allocations |
| --- | ---: | ---: | ---: | ---: |
| `W n=8 norm_squared` | 0.145900711 | 0.110920250 | 6,684 | 4,659 |
| `W n=16 norm_squared` | 1.039919250 | 0.673987000 | 45,202 | 27,583 |
| `W n=32 norm_squared` | 7.738125000 | 4.417062500 | 329,895 | 183,050 |
| `W n=64 norm_squared` | 60.936875000 | 33.499375000 | 2,508,595 | 1,309,685 |
| `H n=16 hs_self` | 2.547625000 | 1.053130250 | 149,043 | 60,204 |
| `H n=32 hs_self` | 18.761333000 | 7.150146000 | 1,178,657 | 434,691 |
| `state n=32 inner zero_at=None` | 0.008656087 | 0.008837748 | 323 | 318 |
| `state n=32 inner zero_at=Some(0)` | 0.008656270 | 0.004176656 | 323 | 101 |
| `state n=32 inner zero_at=Some(16)` | 0.008871724 | 0.006581014 | 322 | 213 |
| `state n=32 inner zero_at=Some(31)` | 0.008800883 | 0.008724670 | 322 | 318 |
| `operator n=32 hs zero_at=None` | 0.014198690 | 0.013958293 | 628 | 628 |
| `operator n=32 hs zero_at=Some(0)` | 0.014173706 | 0.006681519 | 628 | 256 |
| `operator n=32 hs zero_at=Some(16)` | 0.014151896 | 0.010547444 | 628 | 448 |
| `operator n=32 hs zero_at=Some(31)` | 0.013843872 | 0.013849771 | 628 | 628 |

The 64-site W norm falls from about 60.9 to 33.5 ms; allocations drop from
2,508,595 to 1,309,685 and cumulative requested bytes from 139,758,288 to 78,213,936.
The 32-site Heisenberg HS self-inner-product falls from about 18.8 to 7.15 ms,
with allocations reduced from 1,178,657 to 434,691. These cases retain the
original realistic operator/state definitions and mathematical outputs.

Rule-call counts directly establish avoided work:

| Sites | Stage 2 W local overlaps | Stage 3 W local overlaps |
| ---: | ---: | ---: |
| 8 | 512 | 232 |
| 16 | 4,096 | 1,616 |
| 32 | 32,768 | 11,936 |
| 64 | 262,144 | 91,456 |

For the 32-factor product controls, state overlaps now visit 1, 17, or 32 factors
when zero is at index 0, 16, or 31; the no-zero case still visits all 32. HS
primitive traces number 2, 18, or 33 instead of 33, including the one identity
trace establishing the local dimension. Unit-test division counters additionally
show that zero primitive traces do not run normalization division.

In the no-zero controls, the state inner product is about 2% slower in this run
(8.66 to 8.84 microseconds), while HS is about 2% faster (14.20 to 13.96
microseconds). The late-zero controls also stay close to baseline. These small
changes do not establish a general regression or speedup; all measurements are
host-specific medians, not confidence intervals. No extra tuning or alternative
sample was substituted for these controls.

## Limits and remaining work

For this W representation and factor order, diagonal term pairs visit n factors
and off-diagonal pairs visit min(i,j)+1. The resulting call count is
(n^3 + 3n^2 - n)/3, still cubic. All n^2 pairs are considered. End-to-end norm
timing also includes Stage 2 operand evaluation/collection, structural group
metadata and allocation work. Group discovery and graded lowering can prepare
later factors even when their numeric contraction is subsequently skipped.
This stage does not claim to avoid all preparation or remove cubic scaling.

Exact-zero propagation follows mathematical structure: it can omit operations
that the old IEEE sequence would execute after zero, including signed-zero or
NaN/inf interactions. There is no approximate threshold, new finite-value
rejection, or change to the documented coefficient algebra. NaN alone does not
satisfy is_zero. Validation still precedes contraction shortcuts; basis queries
are unchanged by this stage.

Basis-aware inner products, orthogonality prefilters and partial-contraction
reuse remain [future work](../../future-work.md#32-inner-product-acceleration).
H/H² operator-coordinate identity/basis-call counts retain their previous work
pattern; sparse coordinate generation remains Stage 4.

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
cargo test --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo build --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo run --release --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- all
```

The final default workspace run followed the last zero-addition shortcut; all
three feature configurations cover the complete final implementation. The probe's
three semantic fixtures pass. Source hashes, seven-sample medians, expected
call counts, local documentation links/anchors/tables, formatting and final diffs
were checked separately.

Rust 1.85, dedicated no_std targets, alternative platforms/native providers,
and every standalone backend feature combination were not verified. Broader
integration acceptance remains Stage 5; Stage 4 has not been implemented.
