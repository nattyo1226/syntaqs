# Verification structure and commands

Updated: 2026-11-17

## 1. Verification responsibilities

Use the commands below to check workspace behavior, optional backends, and
application-owned native linkage. The coverage inventory identifies where a
regression belongs; it describes test intent rather than a new execution result.

Feature designs own their mathematical contracts and boundary cases. Dated runs
and progress belong in active plans, following the
[documentation guide](../README.md#1-document-responsibilities).

Production-library tests live in each crate's `tests/` and unit tests. [syntaqs-tests](../../tests/README.md) holds shared three-level and graded two-state algebras, non-orthonormal bases, independent references, and core/normalize/export/Lie integration tests. [Examples](../../examples/README.md) include end-to-end model and Lie workflows. Production crates do not depend on test packages.

Place unit tests under the module they exercise. Shared integration-test helpers use `tests/common/mod.rs`, so `mod common` can import them without Cargo discovering an additional test target. Core unit tests may also reference the coefficient fixture there with `#[path = "../tests/common/mod.rs"]`.

## 2. Common checks

Run from the repository root with Cargo and the selected Rust toolchain available
on PATH. Linked workspace tests and examples use the internal
[BLAS provider](../../support/blas-provider/README.md) and require system OpenBLAS,
including LAPACK, and pkg-config. Downstream applications choose their own
provider as described in [native linkage](../../crates/syntaqs-lie/README.md#native-linkage).

```sh
cargo build --workspace --all-features
cargo test --workspace
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo check --workspace --all-targets --no-default-features
cargo fmt --all -- --check
cargo fmt --manifest-path tests/fixtures/lie-blas/Cargo.toml -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Match the configured CI toolchains when selecting checks:

| Checks | Toolchains |
| --- | --- |
| Workspace default/all/no-default-feature tests | Rust 1.85.0 and stable |
| All-feature build and all-target no-default-feature check | Rust 1.85.0 and stable |
| Formatting, Clippy, and rustdoc | Stable |

Run the Lie example and shared integration tests independently of export backends,
and verify the downstream application-owned native link:

```sh
cargo run -p examples --no-default-features --features lie --bin lie_algebra
cargo run -p examples --all-features --bin lie_algebra
cargo test -p syntaqs-tests --no-default-features --features lie --test lie_analysis
cargo run --manifest-path tests/fixtures/lie-blas/Cargo.toml
```

For independent optional-backend checks, use `cargo test -p syntaqs-export --no-default-features` and `cargo test -p syntaqs-tests --no-default-features` with no backend, `--features ndarray`, and `--features sprs`. Workspace all-features covers both together. Keep separate package checks because workspace runs may unify features from other packages.

Compare algebraic evaluation, expansion, and dense/sparse output at both precisions against independent small matrices, Fock-space sign rules, and Gram matrices. Do not construct reference values using the evaluator/export under test. Follow [numerics](numerics.md) for approximate comparison, exact zero, and cancellation residuals. Capability tests and compile_fail doctests also check bounds and incompatible type combinations.

### 2.1. Regression coverage

Maintain the following coverage alongside the detailed verification targets in each feature design. These describe the tests and their intended guarantees, not results of a new test run.

| Area | Coverage and references |
| --- | --- |
| Ordinary and graded composition | [core_tensor_products.rs](../../tests/tests/core_tensor_products.rs) compares two- and three-factor calculations against independent ordinary Kronecker matrices for f32 and f64. Cover kinds, identifications, mixed-kind rejection, subsystems, fallible input precedence, CAR, Act scope, local-embedding homomorphisms, three odd components, adjoints, inner products, and ordinary trace. |
| Basis and numerical integration | [core_basis_expansion.rs](../../tests/tests/core_basis_expansion.rs), [export_tensor_products.rs](../../tests/tests/export_tensor_products.rs), and [normalize_tensor_products.rs](../../tests/tests/normalize_tensor_products.rs) cover intrinsic coordinates, re-expansion, nonorthonormal bases, empty bases, metadata, all output backends, normalization, and expectations at both precisions. |
| Dimension-normalized HS inner products | [core_hs_inner_product.rs](../../tests/tests/core_hs_inner_product.rs) compares against explicit `Tr(A†B)/D`, including empty spaces, spectators, explicit-space binding overrides, extreme coefficient scales, and `⟨I,I⟩=1` on 2048 factors whose total dimension overflows. Verify no state type, Dimension, or integer conversion is required and ordinary trace gains no division bound. |
| Exact primitive collection | [core_collect_terms.rs](../../tests/tests/core_collect_terms.rs) checks exact primitive collection and ordinary/graded signs at f32/f64 without primitive Clone or Dimension. |
| Lie analysis | The shared package's `lie` feature enables [lie_analysis.rs](../../tests/tests/lie_analysis.rs) and [lie_decomposition.rs](../../tests/tests/lie_decomposition.rs). Component and integration verification targets are defined in [Lie verification](lie.md#8-verification); the independent Lie commands above check feature selection. |
| Sharing, capabilities, and large spaces | Capability tests cover borrowed and non-Clone primitives, operation-specific numeric bounds, and action without primitive Product. [core_tensor_products.rs](../../tests/tests/core_tensor_products.rs) includes separated states and local traces on 80 factors without dense intermediates, and evaluation and destruction through 20,000 graded wrappers on a 128 KiB stack. |
| Formatting | [formatting.rs](../../crates/syntaqs-core/tests/formatting.rs) covers all expression node kinds, parentheses, multiline output, complete Debug identifiers, output errors, shared-subexpression classification, and display and destruction at depth 20,000 on a 128 KiB stack. Standard-index and builtin-primitive tests check identities and labels. |
| Local algebras and physical models | Follow [local algebra](local-structure.md), [geometry](geometry.md), and [model](models.md) verification targets: independent local matrices, algebraic laws, normalized Gram matrices, physical coupling scales, constant shifts, many-body placement order, and model energies. Keep exact-zero rules and preservation of tiny nonzero coefficients separate from approximate comparisons. |

Reference values for graded tests come from the [test-only two-state algebra](../../tests/src/graded.rs) and independently assembled ordinary matrices. They must not depend on production fermion rules or the evaluator under test. Resource and depth regression sizes are specific test cases, not general performance bounds.

Core `expression::group::tests` covers structural grouping without coefficient
arithmetic or primitive evaluation capabilities, index equality across distinct
handles, binding/error preservation, sharing and idempotence, and iterative
grouping/destruction of deep single-factor, scalar, and multi-factor syntax.
The local-grouping cases in
[core_tensor_products.rs](../../tests/tests/core_tensor_products.rs) check
mixed parity, complex scales, scalar identity, product order, adjoints,
rebinding, and automatic basis/trace/HS preparation against independent ordinary
matrices in both precisions and tensor kinds. Formatting tests follow the
retained local structure after same-factor construction grouping.

Core `evaluate::tests::state` checks per-action collection and call counts,
shared-input memoization, coefficient/tensor normalization, exact cancellation,
first-occurrence order, nonreflexive PartialEq, retained factored sums, and
space preservation. [core_state_collection.rs](../../tests/tests/core_state_collection.rs)
checks 128 repeated rotations and eight six-site Heisenberg actions against
analytic/bit-state references at f32/f64. Capability fixtures distinguish local
coefficient equality from composite equality. Basis tests cover errors in
surviving terms and validate original operator placements before removing zero
terms.

Core `evaluate::tests::stopping` checks exact-zero stopping in product actions
and at the first, middle, and last factors of contractions. It covers nonzero
controls, underflow, tiny and NaN values, independent and overlapping tensor
groups, zero summands and partial sums, skipped coefficient operations,
ordinary and graded trace factors, and original space and reference checks.
The fixture exercises low-level contractions with equality-free coefficients
and composite operator contractions with non-Clone primitives. Call counts
also cover common interior blocks in operator sums. A 20,000-level action-free
contraction/destruction case runs on a small stack.
[core_collect_terms.rs](../../tests/tests/core_collect_terms.rs) checks
graded exchange signs when extracting common blocks against independent matrices.

Core `basis::expansion::operator::tests` checks once-per-query identity expansion,
queries with no spectators or no factors, placement in ambient factor order,
nonzero identity digits and non-unit coefficients, multi-component identity and
specified choices, working-key restoration, exact cancellation, original placement
validation, and basis errors in surviving terms.
Payload counters and node-sharing checks cover expanded syntax without primitive
cloning. Extreme spectator coefficients check ambient-order multiplication,
and a 20,000-axis enumeration case checks iterative traversal on a small stack.
Existing independent ordinary/graded matrix and nonorthogonal-basis tests verify
both precisions and reconstructed expression semantics.

Core `space::tests` covers index lookup with interleaved IDs, preservation of factor
order and effective tensor kind through flattening, and the agreement of
`tensor_flatten()` with `tensor().flatten()`, including error precedence.

[core_evaluation_pipeline.rs](../../tests/tests/core_evaluation_pipeline.rs)
combines local potential grouping, repeated hopping-model actions and collection,
state and operator HS contractions, and operator coordinates/expanded syntax.
Independent three-site matrices check ordinary and graded inputs at f32/f64,
including singleton and multi-component spectator identity bases. This complements
the workstream-specific call-count and capability tests above.

## 3. CI and verification limits

The [CI workflow](../../.github/workflows/ci.yml) runs on pushes to `main`, pull requests, and manual dispatch on 64-bit Linux. Rust 1.85.0 and stable each test the workspace with default, all, and no default features, including doctests and compile_fail cases. For each toolchain, only the default-configuration job runs the additional Lie-only example and shared Lie integration tests without export backends, downstream linkage fixture, and provider-selection checks. These commands select their own features independently of the workspace test configuration.

Stable also runs backend-specific export/shared tests, format, Clippy, and API documentation with warnings denied. Dependency caching and cancellation of older runs for the same PR/branch are configured.

Lie numerical tests, shared Lie integration tests, and Lie/export examples share the internal
[blas-provider package](../../support/blas-provider/README.md).
CI installs system OpenBLAS and pkg-config. The standalone application in
`tests/fixtures/lie-blas` executes f32/f64 BLAS/LAPACK calculations and checks
that provider features are selected by the application through blas-src, not
by syntaqs-lie or ndarray-linalg. CI checks both provider-neutral and
application-selected dependency features with `cargo tree`; the same shell
check can be run locally with the same tools and native dependencies.
Feature-specific numerical, representation, invariant, and decomposition
verification targets are maintained in the [Lie design](lie.md#8-verification).

Family analysis inherits the expression-contraction cancellation limits documented
in [Lie contracts](lie.md#3-projection-and-numerical-scales). Some cyclic
nonorthonormal inputs need explicit residual overrides; no validation stage
silently relaxes them. Dedicated-target no_std builds, cross-platform bitwise
seed reproducibility, and equivalence with the
[pinned Julia reference](../decisions/lie-analysis.md#context) are not established
by host test success. Workflow configuration alone does not establish a
GitHub-hosted run. Residual-accuracy investigation is tracked in
[future work](../future-work/evaluation-improvements.md#1-lie-residual-accuracy).

Dedicated no_std target builds for core, geometry, spin-half, and fermion remain unverified. Source/dependency-feature inspection and host builds do not establish dedicated-target success. Shared Arc requires pointer-width atomic operations on the target.

Tests for sharing, sparse output without dense intermediates, and iterative
deep-tree processing are not general speed/memory guarantees. Task-specific
timings, allocation counts, and skipped-call assertions do not establish
performance across all workloads, toolchains, or targets. Track further verification and optimization in [future work](../future-work.md#3-outstanding-verification-and-performance).

The [core evaluation performance decision](../decisions/core-evaluation-performance.md)
preserves reference workloads, measurement methodology, comparative results,
and Git provenance for the accepted grouping, collection, contraction, and
coordinate-generation changes. These dated observations are not CI thresholds.
