# Implementation plan

Status: Steps 1–4 complete; Steps 5–6 pending. Continue in order against
[the specification](spec.md). Each step includes its feature tests. Record
actual commands/results when completed, not as prospective successes.

## Step 1: Crate and numerical foundation

- [x] Add syntaqs-lie to the workspace with crate documentation and private
  numerical/error modules. Use core, ndarray, and compatible ndarray-linalg;
  keep concrete algebras in development/example dependencies.
- [x] Enable ndarray/ndarray-linalg BLAS features and retain linkage through a
  provider-neutral blas-src dependency. Centralize the development provider
  selection in the unpublished internal `support/blas-provider` package,
  initially system OpenBLAS from Nix/CI, and add a downstream fixture
  selecting it through blas-src. Expose no syntaqs-lie provider features.
- [x] Implement tolerance settings/defaults, effective-setting diagnostics,
  linear solves, symmetric eigensolves, and real SVD/nullspace helpers.
- [x] Set operation-specific scales, Gram-eigenvalue/conditioning rules, and
  roundoff policy for real coordinates and squared norms. Define invalid
  settings/nonfinite numerical-data errors without changing core evaluation.

**Completion gate:** f32/f64 solve/eigen/nullspace smoke tests execute with the
test provider; zero/rank-deficient rectangular matrices yield expected nullspaces.
Verify the dependency pair with Rust 1.85 and stable. The downstream fixture
links through its own blas-src provider selection without ndarray-linalg
provider features. Production feature inspection shows no library-selected
provider. Check native-link propagation in workspace feature combinations and
that the selected source supplies LAPACK too. Cover tolerance overrides and
boundary cases.

Resolve concrete options/error signatures here. If the dependency pair fails
the existing minimum Rust version, establish a compatible configuration before
dependent steps; do not silently raise the workspace minimum.

## Step 2: Operator families, validation, and projection

- [x] Implement OperatorFamily construction, ordered access, shared expression
  retention, and common-context selection using existing local/placed APIs.
- [x] Add Gram evaluation, rank/independence and Hermiticity diagnostics,
  reconstruction, projection coefficients, residuals, and span membership.
- [x] Return errors for dependent/numerically singular bases where uniqueness
  is required. Retain report tolerances; allow explicit Gram/factorization reuse.
- [x] Set family/result ownership and validation-report signatures. Context
  compatibility is checked even when expressions are zero.

**Completion gate:** empty/incompatible inputs fail construction; compatible
unvalidated candidates preserve order. Independently assembled matrices check
nonorthonormal Gram solves, reconstruction, and singular-basis errors at both
precisions. For `[Sx,Sz]` and `A=Sx+Sy`, obtain coefficients `[1,0]`, projection
Sx, residual 1, and failed membership at the default tolerance. Also cover an
in-span target, real basis changes, report diagnostics, and tiny coefficients
remaining in ordinary expressions.

## Step 3: Brackets and retained Lie representations

- [x] Add the Lie bracket, pairwise closure diagnostics, bracket coefficients,
  and retained structure constants and adjoint matrices with explicit metadata.
- [x] Add the classical Killing form, transformation/component selection,
  orthogonalization, and orthonormality checks for families.
- [x] Ensure operations needing a Lie algebra check the required properties;
  do not turn projected nonclosed brackets into structure constants.
- [x] Finalize supporting result names and array/metadata accessors within
  syntaqs-lie; do not introduce general map abstractions in core/export.

**Completion gate:** Pauli su(2) gives cyclic structure constants 2 and
`K=-8 I_3`; an abelian algebra gives zero Killing form. Check antisymmetry,
Jacobi, direct versus coefficient bracket actions, and adjoint column order.
For real invertible U, check `K_new=U^T K_old U`. Independent dense references
cover nonorthonormal inputs, transform reconstruction, closed/nonclosed cases,
and f32/f64. Compare a supported graded example with its ordinary action matrix
to verify signs while retaining the ordinary Lie bracket.

## Step 4: Adjoint commutant and center

- [x] Form real linear constraints for the full adjoint commutant and common
  adjoint kernel. Return matrices in the original family coordinates.
- [x] Construct internal orthonormal coordinates and their transforms. Compute
  the real symmetric commutant subspace there as a distinct internal object.
- [x] Extract the full center and its orthogonal complement. Handle absent
  center and absent semisimple part without constructing an empty OperatorFamily.

**Completion gate:** returned commutant matrices commute with every adjoint
matrix, and central vectors are annihilated by all adjoints. Expected full
real commutant dimensions are 1 for su(2), 2 for su(2) plus u(1), and n² for
an n-dimensional abelian algebra. Verify center dimensions, real basis-change
covariance, and independent constraint residuals at both precisions. Symmetric
subspace results must not be mislabeled as the full commutant.

## Step 5: Complete center and simple-ideal decomposition

- [ ] Implement the specified center-first Julia-style sampling/eigenspace
  algorithm and reconstruct results as LieDecomposition.
- [ ] Add caller-supplied seeds, finite retry settings, eigenvalue grouping,
  exhaustion errors, and effective-setting diagnostics.
- [ ] Validate the full center, direct-sum span, ideal invariance, cross-ideal
  commutation, and complete splitting. For the compact centerless blocks,
  check that no further symmetric commuting projector can split a block;
  a restricted symmetric commutant larger than scalar identity signals an
  incomplete split rather than a simple ideal.
- [ ] Support orthogonalizing decomposition components. Resolve zero-center
  layout and result accessor signatures without adding validated-basis types.

**Completion gate:** recover su(2), two commuting su(2) factors, su(2) plus
u(1), a purely abelian algebra, and a multidimensional center with a simple
factor. Test orthonormal and nonorthonormal input transformations, both
precisions, same-environment seed reproducibility, and retry/exhaustion paths.
Compare spans/projectors and invariants, not component order or individual
eigenvectors. Deliberately insufficient spectral separation must not produce
successful complete decomposition.

## Step 6: Public integration and full verification

- [ ] Add a runnable syntaqs-lie example covering family construction,
  validation, projection, retained coefficients, and center/ideal decomposition.
  Show blas-src linkage, the required native libraries, and tolerance/seed
  overrides; avoid ndarray-linalg provider-selection instructions.
- [ ] Add public-API integration coverage using existing spin/fermion and shared
  test algebras as appropriate; confirm Gram/bracket evaluation needs neither
  full physical matrices nor a state basis.
- [ ] Update crate/root READMEs, workspace ordering, examples documentation,
  permanent requirements/architecture/designs, and CI to reflect implemented
  behavior. Preserve existing crate/backend coverage and no_std boundaries.
- [ ] Run the applicable complete repository checks and record results below.
  Release numbering, commits, publication, and merging are separate tasks.

**Completion gate:** independent small-system references agree with direct
syntaqs operations and dense coefficient calculations. All stages are usable
through public APIs, provider choice remains with the application, and the
declared toolchain/CI configurations are verified. After completion and merge,
transfer any remaining lasting contracts and retire these temporary documents.

## Verification procedure

Run targeted crate tests after each feature step; widen checks when shared
code, manifests, or infrastructure change. Use `direnv exec .` when needed.
Once the test backend is configured, the final checks include:

```sh
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo check --workspace --all-targets --no-default-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Also run the existing default/no-default-feature CI tests, Rust 1.85 checks,
and independent export/shared backend checks from
[verification](../../design/verification.md). Use the centralized development
blas-src provider for linked test/example binaries and verify feature-unified
workspace builds. syntaqs-lie has no mutually exclusive provider features to
combine under all-features.

## Step 1 results: 2026-11-04

Implemented `NumericalOptions`, `Tolerance`, `ToleranceCheck`, and
`NumericalError`, with private real solve/eigen/SVD/nullspace/Gram kernels.
Independent numerical tests cover both precisions, wide/tall/empty matrices,
noncontiguous layouts, overrides, finite-data checks, and roundoff boundaries.

| Check | Result |
| --- | --- |
| `cargo test -p syntaqs-lie` in stable and Rust 1.85.0 devShells | 22 tests passed in each. |
| `cargo test --workspace` with default, all, and no default features | Passed, including doctests. |
| `cargo build --workspace --all-features`; `cargo check --workspace --all-targets --no-default-features` | Passed. |
| Workspace format, Clippy all-targets/all-features with `-D warnings`, and rustdoc with `-D warnings` | Passed; standalone fixture formatting passed too. |
| Standalone `lie-blas` application in stable and Rust 1.85.0 devShells | f32/f64 BLAS matrix products and LAPACK solve/eigen calls executed successfully. |
| Provider-isolation CI shell check using `cargo tree` | Provider-neutral graph has no native provider, and application selection uses blas-src without ndarray-linalg provider features. |
| Independent syntaqs-export and syntaqs-tests runs for Vec, ndarray, and sprs | All six package/backend combinations passed. |

Verification ran locally on aarch64 macOS with system OpenBLAS from Nix.
CI now installs OpenBLAS/pkg-config and executes the downstream/graph checks;
GitHub-hosted Linux jobs have not been run in this session. Existing manifest
warnings about both license fields remain. Enabling ndarray BLAS exposed a
workspace feature-unification link failure in the existing Heisenberg example
and its integration test; both now import the central development provider
only with the export feature, and all workspace configurations pass.

The shared provider is an unpublished internal package at
`support/blas-provider`. After its relocation and rename, Lie and example tests
with all features, standalone native-link execution, provider-graph checks,
workspace formatting, and all-target/all-feature Clippy passed again.

The existing local `.sideflake/flake.nix` devShell was extended with OpenBLAS,
pkg-config, and an MSRV shell. It is globally Git-ignored, as is the local
Cargo.lock; these local environment changes are not tracked source changes.
Production library dependencies do not select OpenBLAS. Step 1 provided the
numerical foundations only.

## Step 2 results: 2026-11-04

Implemented ordered local/placed `OperatorFamily<E>`, Hermiticity and
real-independence reports, complex Gram matrices, reusable real LU
factorizations, real coordinates, reconstruction, projection, residuals, and
span checks. Results borrow their originating family; numerical settings remain
explicit. Concrete API contracts are in [the current design](../../design/lie.md).

Core provides explicit exact term collection to reduce cancellation before
squaring residuals. It requires primitive equality, shares handles without
primitive cloning, retains local sums in factored form, and removes exact zeros
only. A 32-factor regression protects against global Cartesian expansion.
Ordinary evaluation and core's no_std/dependency direction remain unchanged.

| Check | Result |
| --- | --- |
| `cargo test -p syntaqs-lie` in stable and Rust 1.85.0 devShells | 22 numerical tests, 27 public API tests, and one doctest passed in each. |
| `cargo test -p syntaqs-tests --test core_collect_terms` | Two independent ordinary/graded reference tests passed. |
| `cargo test --workspace` with default, all, and no default features | Passed, including doctests; the final all-feature rerun after collection refinement passed too. |
| `cargo build --workspace --all-features`; `cargo check --workspace --all-targets --no-default-features` | Passed. |
| Workspace format, all-target/all-feature Clippy and rustdoc with `-D warnings` | Passed. |
| Standalone `lie-blas` application and Cargo provider-isolation check | Passed. |

Public tests cover independent small-matrix references, nonorthonormal and
changed bases, in/out-of-span targets, complex-coefficient Hermitian expressions,
invalid contexts including zero inputs, singularity, tolerance overrides,
nonfinite data, tiny positive residuals, and non-Clone primitives/indices without
state/basis/Dimension capabilities. Verification ran locally on aarch64 macOS
with Nix OpenBLAS. GitHub-hosted Linux CI has not run in this session. Closure,
brackets, retained Lie representations, and decomposition remain later steps.

## Step 3 results: 2026-11-09

Implemented ordinary `lie_bracket`, family bracket expressions and projection
coordinates, ordered pairwise closure diagnostics, retained `StructureConstants`,
`AdjointRepresentation`, and classical `KillingForm`. Closure/coefficients reuse
explicit Gram factorizations. Constants require Hermiticity, independence, and
passing closure; failed closure remains a report or `NonClosedPair` error.
Arrays borrow the exact source family and retain numerical settings.

Added real rectangular family transformations and selections retaining column
coordinates, Gram-eigenvector orthogonalization/orthonormalization, and explicit
orthonormality reports. Concrete API contracts and numerical scales are in
[the current design](../../design/lie.md). No general core/export map abstraction
or production dependency was added. Fermion is a development dependency for
the graded reference test.

Sixteen new public tests cover f32/f64 Pauli su(2), `K=-8I`, abelian zero
Killing forms, nonorthonormal dense references, antisymmetry, Jacobi, adjoint
column order and action, Killing covariance, transforms and reconstruction,
orthogonalization, closure failures, setting overrides, bad inputs, and graded
odd-Majorana brackets compared with independent Jordan–Wigner matrices.
Existing capability tests now exercise constants and orthonormalization without
primitive/index Clone, Dimension, or state traits.

| Check | Result |
| --- | --- |
| `cargo test -p syntaqs-lie` under stable and Rust 1.85.0 | 22 numerical tests, 27 family tests, 16 representation tests, and one doctest passed in each. |
| `cargo test --workspace` with default, all, and no default features | Passed, including doctests. |
| `cargo build --workspace --all-features` | Passed. |
| `cargo check --workspace --all-targets --no-default-features` | Passed. |
| Workspace formatting, all-target/all-feature Clippy with `-D warnings`, rustdoc with `-D warnings` | Passed. |

Verification ran locally on aarch64 macOS with Nix OpenBLAS. GitHub-hosted
Linux CI was not executed in this session. Existing warnings about both license
manifest fields remain. Center, commutant, and decomposition are subsequent steps.


## Step 4 results: 2026-11-09

Implemented the full real `AdjointCommutant` from row-major homogeneous
commuting constraints in original coordinates. `CenterSplit` extracts the
full common adjoint kernel in internal HS-orthonormal coordinates, retains
forward/inverse transforms, maps center and complementary SVD directions back
to original coordinates, and reconstructs optional operator families. Zero
center/complement dimensions use `(n,0)` arrays and `None`, never empty families.
Methods on retained adjoints reuse coefficients/settings; family convenience
methods first enforce Hermiticity, independence, and closure.

`ConstraintDiagnostics` retains SVD shape, rank, singular values, cutoffs, and
residual checks. Candidates classified as null must also pass the residual
bound; otherwise `ConstraintResidual` identifies the failed candidate. Center
columns are additionally checked against the original adjoints. The SVD kernel
now retains its complementary right singular vectors, preserving a complete
orthogonal split under a single rank decision. The distinct internal
`SymmetricCommutant` uses normalized symmetric parameters in orthonormal
coordinates; it is tested and staged for Step 5's sampling algorithm.
No dependencies or core/export APIs changed. Contracts are in
[the current design](../../design/lie.md#6-adjoint-commutant-and-center).

Twelve new public tests and four internal tests cover f32/f64 independent su(2)
block constraints, full commutant spans/dimensions, purely abelian and multiple
center directions, two simple factors, absent components, reconstruction,
HS orthogonality and complete span, basis-change covariance, graded generators,
precondition failures, and independent rank/residual settings. Internal tests
explicitly distinguish dimensions six versus nine for the symmetric versus full
commutant of a three-dimensional abelian algebra, and four versus five for
su(2) plus a two-dimensional center. Existing numerical/capability tests also
cover SVD complements and APIs without primitive/index Clone, Dimension, or states.

| Check | Result |
| --- | --- |
| `cargo test -p syntaqs-lie` under stable and Rust 1.85.0 | 26 unit tests, 12 invariant tests, 27 family tests, 16 representation tests, and one doctest passed in each. |
| `cargo test --workspace --all-features` | Passed, including doctests. |
| `cargo build --workspace --all-features`; `cargo check --workspace --all-targets --no-default-features` | Passed. |
| Workspace formatting, all-target/all-feature Clippy and rustdoc with `-D warnings` | Passed. |

Verification ran locally on aarch64 macOS with Nix OpenBLAS. GitHub-hosted
Linux CI was not executed in this session. Existing warnings about both license
manifest fields remain. Full default/no-default workspace test runs and independent
backend/provider checks were not repeated for this crate-local change; the
complete integration/release verification remains Step 6. Simple-ideal
decomposition remains Step 5.
