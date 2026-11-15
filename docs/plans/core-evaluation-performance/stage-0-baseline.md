# Stage 0 baseline and validation

Status: completed against production revision `a0700711b4320f60e745c0a3dc96a00e286252f8`.
Only the plan-local probe and documentation were modified for Stage 0.
No Stage 1–4 optimization is implemented or measured here.

## Reproduction and provenance

The [raw output](stage-0-baseline.txt) records the source revision, clean status
for `crates/`, `tests/`, and `examples/`, compiler/host, Cargo version, probe-source
SHA-256 digests, and all seven timing samples per case. The probe changes are
uncommitted Stage 0 changes on that revision; their digests identify the measured
harness. The [dependency snapshot](stage-0-probe.lock) matches the Cargo.lock hash
in the raw output. The probe remains outside the production workspace.

For the recorded dependency resolution, copy the snapshot to the ignored probe
lockfile, then build and run from the repository root:

```sh
cp docs/plans/core-evaluation-performance/stage-0-probe.lock docs/plans/core-evaluation-performance/probe/Cargo.lock
cargo build --release --locked --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml
cargo run --release --locked --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml -- all
```

Offline reproduction requires cached dependencies. The original execution used
the built binary under the repository's development environment. Test runs and
benchmark runs were sequential; no other test or benchmark was intentionally
run during this session. This does not assert isolation from all host activity.

The compiler was rustc 1.99.1 / LLVM 23.1.1 on aarch64-apple-darwin, with default
Cargo release optimization and the counting system-allocator wrapper. No custom
optimization flags were set by the measurement command.

Each case warms up once, calibrates a power-of-two batch toward 10 ms with a
65,536-iteration cap, and reports the median of seven per-call batch averages.
Long cases use one operation per sample. Destruction of each result is included.
An empty black-box control is recorded to expose the measurement floor. The
shortest ready-input controls hit the iteration cap before the target duration;
treat those nanosecond-scale results as rough baselines, not precise guarantees.

Allocation requests and cumulative requested bytes come from one separate call
with counting enabled. They are neither peak memory nor statistics over the
timing batches. The counter includes reallocation sizes and all destruction-time
allocations. Counts can vary slightly with allocation/layout details.

## Main cases

| Case | Median per call (ms) | Allocation requests | Requested bytes |
| --- | ---: | ---: | ---: |
| `Hadamard_layer_dagger_layer n=8 local_sum` | 0.010321248 | 481 | 31,520 |
| `Hadamard_layer_dagger_layer n=8 placed_sum` | 251.076000000 | 10,555,044 | 1,094,632,312 |
| `rotation k=16 evaluate` | 96.364458000 | 3,866,499 | 199,228,480 |
| `rotation k=16 coordinates` | 105.878250000 | 3,866,509 | 199,231,264 |
| `rotation k=16 expand_each_step` | 0.015389078 | 640 | 41,472 |
| `Hpow Neel n=6 k=4 evaluate` | 55.237084000 | 1,788,119 | 108,566,400 |
| `Hpow Neel n=6 k=4 coordinates` | 140.304417000 | 4,880,862 | 577,280,160 |
| `Hpow Neel n=6 k=4 expand_each_step` | 1.637250000 | 58,832 | 9,116,744 |
| `W n=64 norm_squared` | 99.744125000 | 2,438,984 | 133,681,016 |
| `W n=64 coordinates` | 1.195640625 | 41,056 | 8,850,912 |
| `H n=64 coordinates` | 3.930281250 | 134,643 | 26,369,504 |
| `H2 n=64 coordinates` | 752.744500000 | 25,135,284 | 4,896,265,824 |
| `H n=32 hs_self` | 18.386458000 | 1,178,622 | 43,147,640 |
| `plus n=18 norm_squared` | 0.006087911 | 266 | 21,720 |
| `plus n=18 coordinates` | 152.033958000 | 1,159,949 | 271,932,104 |

## Regression controls

Construction cases include building and binding the operator and destroying the
result. Spaces and site handles are prepared outside the timed closure. The
local-sum path builds a reusable local gate per call; the placed-sum path builds
its component operators at each site, so these are caller-level construction
controls rather than equal-operation-count microbenchmarks.

The duplicate-sum control currently just shares a ready tree; Stage 2 intentionally
adds collection work. The evaluated-Neel control measures a second evaluation of
an already evaluated state. The one-site-action control has no term proliferation.

| Case | Median per call (ms) | Allocation requests | Requested bytes |
| --- | ---: | ---: | ---: |
| `construction n=8 placed_sum=false` | 0.000767649 | 41 | 1,832 |
| `construction n=8 placed_sum=true` | 0.001974091 | 105 | 4,456 |
| `construction n=32 placed_sum=false` | 0.003002452 | 137 | 12,584 |
| `construction n=32 placed_sum=true` | 0.008220927 | 393 | 29,224 |
| `construction n=128 placed_sum=false` | 0.016528564 | 521 | 147,752 |
| `construction n=128 placed_sum=true` | 0.045052570 | 1,545 | 312,616 |
| `ready Neel n=64 evaluate` | 0.000031792 | 2 | 128 |
| `evaluated Neel n=64 evaluate_again` | 0.000031987 | 2 | 128 |
| `Neel n=64 one_site_action` | 0.002105225 | 35 | 22,472 |
| `ready local duplicate_sum n=128 evaluate` | 0.000029633 | 2 | 96 |
| `state n=32 inner zero_at=None` | 0.008623657 | 327 | 22,464 |
| `operator n=32 hs zero_at=None` | 0.013346517 | 626 | 44,216 |
| `state n=32 inner zero_at=Some(0)` | 0.008468282 | 327 | 22,464 |
| `operator n=32 hs zero_at=Some(0)` | 0.013286295 | 626 | 44,216 |
| `state n=32 inner zero_at=Some(16)` | 0.008485474 | 327 | 22,464 |
| `operator n=32 hs zero_at=Some(16)` | 0.013222087 | 626 | 44,216 |
| `state n=32 inner zero_at=Some(31)` | 0.008505127 | 327 | 22,464 |
| `operator n=32 hs zero_at=Some(31)` | 0.013143392 | 626 | 44,216 |

## Work counters

- W-state primitive inner-product calls at n = 8, 16, 32, 64 were respectively
  512, 4,096, 32,768, and 262,144.
- Each 32-factor product-state inner product used 32 primitive overlaps, whether
  there was no zero or a zero at factor 0, 16, or 31.
- Each corresponding operator HS product used 33 primitive traces, including
  the identity trace used for normalization. A zero at any tested position did
  not reduce the count.
- The 64-site H² case returned 17,395 coordinates, with 2,286,144 basis calls and
  2,146,260 identity calls. The 64-site H case returned 189 coordinates, with
  12,096 basis calls and 11,718 identity calls.

These are observations of the unchanged library. The harness no longer asserts
that inefficient n³ counts must persist. Later stages should add targeted tests
for their reduced counts without overwriting this baseline.

## Probe audit and semantic evidence

The audited probe retains the historical modes and adds `controls` and `all`.
It rejects unknown modes, checks both coordinate lengths before zipped rotation
comparison, removes sub-microsecond cached-coordinate norm timing as an unrelated
future-work path, and checks that coordinate norm numerically instead. Counter
wrappers implement primitive PartialEq in preparation for the accepted evaluator
bounds, without requiring primitive Clone.

The added [semantic fixtures](probe/tests/semantic_fixtures.rs) pass against the
baseline library:

1. Action-free duplicate terms, tensor regrouping and coefficient placement have
   the expected coordinates in ordinary and graded spaces; exact cancellation
   retains a small nonzero component.
2. Six-site Heisenberg powers through k=4 agree with independent computational-bit
   action, not a second call through the same evaluator.
3. Spectators expand correctly in a matrix-unit basis and in a basis with
   I = 2 B_3; expanded-expression round trips agree, and fully specified/unit-space
   queries do not request spectator identity expansion.

The fixture tests establish mathematical references, not the future structural
normal form or the availability of group_local. Existing graded matrix-reference
integration tests cover mixed parity, crossing signs, Act scope, and adjoints.

## Checks completed

- `cargo test --offline --manifest-path docs/plans/core-evaluation-performance/probe/Cargo.toml`:
  3 semantic tests passed; the binary has no unit tests.
- `cargo test --offline -p syntaqs-tests --test core_tensor_products --test core_basis_expansion --test core_hs_inner_product --test core_collect_terms`:
  19 tests passed, including basis-error visibility and graded reference cases.
- Probe release build, formatting, and Clippy with `--all-targets -- -D warnings`
  passed. The full `all` measurement completed with every value assertion passing.
- Plan-local links, tables, and whitespace were checked.

No workspace-wide feature/toolchain/backend matrix was rerun for this isolated
probe/documentation change. That matrix remains a later implementation gate.
The new seven-sample batched method differs from the historical five single-call
samples, so small timing differences do not establish a production regression
or improvement. No speedup threshold or confidence interval is claimed.
