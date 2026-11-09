# Verification structure and commands

Updated: 2026-11-09

## 1. Verification responsibilities

Feature designs define mathematical contracts and boundary cases. This document describes test placement, common commands, regression coverage, and environment limitations. Dated execution results and progress belong in active implementation plans; completed plans are removed after lasting specifications are recorded as described in the [documentation guide](../README.md#1-document-responsibilities).

Production-library tests live in each crate's `tests/` and unit tests. [syntaqs-tests](../../tests/README.md) holds shared three-level and graded two-state algebras, non-orthonormal bases, independent references, and core/normalize/export integration tests. [Examples](../../examples/README.md) include end-to-end model tests. Production crates do not depend on test packages.

## 2. Common checks

Run from the repository root. Prefix commands with `direnv exec .` when development tools are not on PATH.

```sh
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo check --workspace --all-targets --no-default-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
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
| Operator families and exact collection | [operator_family.rs](../../crates/syntaqs-lie/tests/operator_family.rs) checks ordered contexts, Hermiticity, real independence, nonorthonormal projection/reconstruction, residuals, singularity, and numerical overrides against small matrices. [core_collect_terms.rs](../../tests/tests/core_collect_terms.rs) checks exact primitive collection and ordinary/graded signs without primitive Clone or Dimension. Both cover f32/f64; see [Lie contracts](lie.md). |
| Sharing, capabilities, and large spaces | Capability tests cover borrowed and non-Clone primitives, operation-specific numeric bounds, and action without primitive Product. [core_tensor_products.rs](../../tests/tests/core_tensor_products.rs) includes separated states and local traces on 80 factors without dense intermediates, and evaluation and destruction through 20,000 graded wrappers on a 128 KiB stack. |
| Formatting | [formatting.rs](../../crates/syntaqs-core/tests/formatting.rs) covers all expression node kinds, parentheses, multiline output, complete Debug identifiers, output errors, shared-subexpression classification, and display and destruction at depth 20,000 on a 128 KiB stack. Standard-index and builtin-primitive tests check identities and labels. |
| Local algebras and physical models | Follow [local algebra](local-structure.md), [geometry](geometry.md), and [model](models.md) verification targets: independent local matrices, algebraic laws, normalized Gram matrices, physical coupling scales, constant shifts, many-body placement order, and model energies. Keep exact-zero rules and preservation of tiny nonzero coefficients separate from approximate comparisons. |

Reference values for graded tests come from the [test-only two-state algebra](../../tests/src/graded.rs) and independently assembled ordinary matrices. They must not depend on production fermion rules or the evaluator under test. Resource and depth regression sizes are specific test cases, not general performance bounds.

## 3. CI and verification limits

The [CI workflow](../../.github/workflows/ci.yml) runs on pushes to `main`, pull requests, and manual dispatch on 64-bit Linux. Rust 1.85.0 and stable each test the workspace with default, all, and no default features, including doctests and compile_fail cases.

Stable also runs backend-specific export/shared tests, format, Clippy, and API documentation with warnings denied. Dependency caching and cancellation of older runs for the same PR/branch are configured.

Lie numerical tests and export examples share the internal
[blas-provider package](../../support/blas-provider/README.md).
CI installs system OpenBLAS and pkg-config. The standalone application in
`tests/fixtures/lie-blas` executes f32/f64 BLAS/LAPACK calculations and checks
that provider features are selected by the application through blas-src, not
by syntaqs-lie or ndarray-linalg. CI checks both provider-neutral and
application-selected dependency features with `cargo tree`; the same shell
check can be run locally under the development environment. The crate's unit
tests cover solves, complete right nullspaces, symmetric spectra, Gram cutoffs,
tolerance overrides, and invalid
numerical data. Family tests cover validation/projection.
[Representation tests](../../crates/syntaqs-lie/tests/representations.rs) cover
brackets, closure, adjoint matrices, Killing forms, and family transformations
against independent dense references at both precisions, including an ordinary
bracket of odd operators on a graded space.
[Invariant tests](../../crates/syntaqs-lie/tests/invariants.rs) compare adjoint
commutants and centers with independent real block matrices, including complete
spans, original-coordinate residuals, basis-change covariance, absent components,
and rank/residual overrides. Unit tests distinguish internal symmetric commuting
subspaces from the public full real commutant. Simple-ideal decomposition remains
a subsequent step.

Dedicated no_std target builds for core, geometry, spin-half, and fermion remain unverified. Source/dependency-feature inspection and host builds do not establish dedicated-target success. Shared Arc requires pointer-width atomic operations on the target.

Benchmarks, speed comparisons, and additional allocation optimization are also unverified. Tests for sharing, sparse output without dense intermediates, and iterative deep-tree processing are not general speed/memory guarantees. Track further verification and optimization in [future work](../future-work.md#3-outstanding-verification-and-performance).
