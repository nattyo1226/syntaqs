# Stage 5: integrated acceptance

Date: 2026-11-16  
Status: complete; local integration gates passed.  
Starting revision: `7a0f552` (Stage 4).

## Integrated behavior and review

No additional production algorithm or public bound change is introduced in this
stage. The new [pipeline regression](../../../tests/tests/core_evaluation_pipeline.rs)
uses a three-site hopping Hamiltonian with a local potential. At f32/f64 in
ordinary and graded spaces, independent small matrices check grouping, repeated
action/collection, state inner products, squared-Hamiltonian HS products, and
both operator coordinates and reconstructed syntax. Matrix-unit and parity
bases exercise multi-component and singleton spectator identities respectively.
The numerical reference does not call production evaluation or export.

Static integration review follows grouping into intrinsic traversal, collector
identity/equality rules and evaluator continuations, contraction scheduling, and
the coordinate generator's key restoration and lazy payload creation. No new
actionable defect was identified in these inspected paths. Existing targeted
tests retain checks for graded signs, adjoints, binding/error precedence, deep
trees, non-Clone primitives, and operation-specific capability bounds. This is
targeted review and regression evidence, not a proof for arbitrary expressions.

## Measurement interpretation

The Stage 0–4 records retain their original samples and provenance. Their shared
cases use the same release harness and seven-sample median; allocation requests
include reallocations and cumulative requested bytes, not peak resident memory.
Result destruction is included. Additional grouping and output controls have no
Stage 0 baseline and are not assigned an overall speedup.

Stage 5 repeats all 102 current cases after integration checks, separately from
test/build processes. It does not attribute timing fluctuations to new production
work: Stage 4 and Stage 5 use identical production and probe sources.

| Case (median ms) | Stage 0 | Stage 1 | Stage 2 | Stage 3 | Stage 4 | Stage 5 | Overall speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Placed Hadamard n=8 | 251.076000 | 0.011048 | 0.010972 | 0.010947 | 0.009565 | 0.010840 | 23,162.5x |
| Rotation, 16 actions | 96.364458 | 107.029583 | 0.005378 | 0.005290 | 0.005474 | 0.005496 | 17,532.5x |
| Six-site H^4 on Neel | 55.237084 | 56.680000 | 0.895786 | 0.705893 | 0.695060 | 0.706271 | 78.2x |
| W64 norm squared | 99.744125 | 98.647625 | 60.936875 | 33.499375 | 33.966250 | 32.996125 | 3.0x |
| H32 HS self | 18.386458 | 18.418167 | 18.761333 | 7.150146 | 6.944312 | 7.282000 | 2.5x |
| H64 coordinates | 3.930281 | 3.979948 | 4.291927 | 3.957479 | 0.210430 | 0.208357 | 18.9x |
| H^2 64 coordinates | 752.744500 | 757.960917 | 774.564959 | 754.000542 | 36.955084 | 37.448167 | 20.1x |

| Allocation control | Stage 0 requests / bytes | Stage 5 requests / bytes |
| --- | ---: | ---: |
| Placed Hadamard n=8 | 10,555,044 / 1,094,632,312 | 444 / 24,920 |
| Rotation, 16 actions | 3,866,499 / 199,228,480 | 240 / 20,984 |
| Six-site H^4 on Neel | 1,788,119 / 108,566,400 | 23,697 / 2,167,128 |
| W64 norm squared | 2,438,984 / 133,681,016 | 1,309,691 / 78,216,064 |
| H32 HS self | 1,178,622 / 43,147,640 | 434,691 / 17,365,424 |
| H64 coordinates | 134,643 / 26,369,504 | 4,119 / 815,800 |
| H^2 64 coordinates | 25,135,284 / 4,896,265,824 | 798,111 / 90,049,744 |

| End-to-end regression control (median ms) | Stage 0 | Stage 5 | Interpretation |
| --- | ---: | ---: | --- |
| One-site action on Neel64 | 0.002105 | 0.023469 | Collection setup dominates a single action |
| Factored plus18 norm | 0.006088 | 0.019744 | Remains factored; preparation costs more |
| Plus18 coordinates | 152.033958 | 154.680459 | 262,144 genuine output entries |
| Construction, 128 placed sums | 0.045053 | 0.037246 | Cheap direct-child grouping |

Work-count acceptance is unchanged from Stage 4: W64 uses 91,456 local
inner-product calls (262,144 before stopping); H64 coordinates use 379 local
basis calls, including one spectator identity (12,096 / 11,718 before sharing).
H-squared64 uses 140,635 basis calls and 751 identity calls, of which 750 arise
from explicit local products and one from spectators. Its 17,395 output keys
are unchanged. Zero at positions 0/16/31 of a 32-factor state product requires
1/17/32 local contractions; HS requires 2/18/33 primitive traces including its
dimension trace. Other sum branches and no-zero controls still complete.

The [raw Stage 5 record](stage-5-results.txt) contains source/probe hashes,
compiler provenance, all samples, allocations, and call counts. Allocation
counts differ slightly in 22 of 102 cases: at most 20 requests and 9,488 cumulative
bytes per query. Address-keyed memo BTreeMaps can have different layouts between
runs; this is a plausible source of variation, not an independently isolated
cause. All work-count/output-size diagnostics match Stage 4. Timing/allocation
changes between these identical-source runs are not a new optimization.

Grouping removes the large representation penalty for placed local sums.
Per-action collection changes repeated rotations and Hamiltonian powers from
action-path growth to retained natural-state contributions. Incremental
contractions reduce local calls after exact zeros. Shared spectator expansion
and direct complete-key emission remove repeated identity work and prefix maps.

W-state contraction still enumerates every term pair and eagerly prepares tensor
group metadata. Its local call count remains cubic, with a smaller constant;
no subcubic claim is made. H-squared coordinates still enumerate intrinsic term
products. Final full-length keys and expanded syntax retain output-size costs.

Single-action and small factored-norm controls regress because query-local
normalization/interner setup now does additional work. These are accepted costs
of the agreed collection semantics, not hidden by the large repeated-action
speedups. The action-free duplicate-sum evaluator now performs collection;
comparison against the former no-op evaluator has different responsibilities.
The genuine 2^18-coordinate output remains output-bound. Further collection
overhead reduction and inner-product acceleration are recorded in
[future work](../../future-work.md#3-outstanding-verification-and-performance).

## Validation

The following checks passed locally on aarch64-apple-darwin. Tests include
unit/integration tests, doctests and compile-fail cases, with no test exclusions.

| Check | Rust 1.85.0 | Stable 1.99.1 |
| --- | --- | --- |
| Workspace tests: default / all-features / no-default-features | Pass / Pass / Pass | Pass / Pass / Pass |
| Workspace all-feature build | Pass | Pass |
| Workspace all-target no-default-feature check | Pass | Pass |
| Lie example: Lie-only / all features | Pass / Pass | Pass / Pass |
| Shared Lie-only integration tests | Pass | Pass |
| Downstream f32/f64 BLAS/LAPACK fixture | Pass | Pass |
| Provider-neutral / application-selected dependency features | Pass / Pass | Pass / Pass |
| Export and shared tests separately: Vec / ndarray / sprs | Not required by CI | Pass / Pass / Pass |
| Workspace and downstream formatting | Not required by CI | Pass |
| All-target/all-feature Clippy, warnings denied | Not required by CI | Pass |
| All-feature rustdoc, warnings denied | Not required by CI | Pass |

Commands are the matrix in [verification](../../design/verification.md#2-common-checks).
Provider selection checks also follow the CI dependency-tree expectations:
provider-neutral graphs have no native-provider package, while the application
selects system OpenBLAS without enabling ndarray-linalg/lax provider features.
The standalone performance probe's three semantic fixtures, formatting,
all-target Clippy with warnings denied, and release build passed on stable.

The MSRV shell already defined by the repository supplied Rust 1.85.0; no project
configuration or dependency was added for verification. Stable reports LLVM
23.1.1 and MSRV LLVM 19.1.7. Performance measurements use stable only. No
GitHub-hosted/Linux run, alternative native provider/platform, or dedicated
no_std target build is claimed by these local results. Dedicated-target and
cross-platform checks remain the existing verification limits.

## Documentation disposition

Current API/algorithm contracts live in the permanent expression, basis, and
verification designs. Deferred work lives in future work. Historical timings
and raw samples remain here for the requested stage-by-stage comparison. Per
the [repository documentation convention](../../README.md#1-document-responsibilities),
the task directory is retained until merge/closeout; no merge is performed by
this stage.
