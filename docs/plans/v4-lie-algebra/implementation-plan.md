# Implementation plan

Status: all implementation steps pending. Execute in order against
[the specification](spec.md); no production code or manifest was changed while
preparing this plan. Each step includes its feature tests. Record actual
commands/results when completed, not as prospective successes.

## Step 1: Crate and numerical foundation

- [ ] Add syntaqs-lie to the workspace with crate documentation and private
  numerical/error modules. Use core, ndarray, and compatible ndarray-linalg;
  keep concrete algebras in development/example dependencies.
- [ ] Enable ndarray/ndarray-linalg BLAS features and retain linkage through a
  provider-neutral blas-src dependency. Centralize the development provider
  selection, initially system OpenBLAS from Nix/CI, and add a downstream fixture
  selecting it through blas-src. Expose no syntaqs-lie provider features.
- [ ] Implement tolerance settings/defaults, effective-setting diagnostics,
  linear solves, symmetric eigensolves, and real SVD/nullspace helpers.
- [ ] Set operation-specific scales, Gram-eigenvalue/conditioning rules, and
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

- [ ] Implement OperatorFamily construction, ordered access, shared expression
  retention, and common-context selection using existing local/placed APIs.
- [ ] Add Gram evaluation, rank/independence and Hermiticity diagnostics,
  reconstruction, projection coefficients, residuals, and span membership.
- [ ] Return errors for dependent/numerically singular bases where uniqueness
  is required. Retain report tolerances; allow explicit Gram/factorization reuse.
- [ ] Set family/result ownership and validation-report signatures. Context
  compatibility is checked even when expressions are zero.

**Completion gate:** empty/incompatible inputs fail construction; compatible
unvalidated candidates preserve order. Independently assembled matrices check
nonorthonormal Gram solves, reconstruction, and singular-basis errors at both
precisions. For `[Sx,Sz]` and `A=Sx+Sy`, obtain coefficients `[1,0]`, projection
Sx, residual 1, and failed membership at the default tolerance. Also cover an
in-span target, real basis changes, report diagnostics, and tiny coefficients
remaining in ordinary expressions.

## Step 3: Brackets and retained Lie representations

- [ ] Add the Lie bracket, pairwise closure diagnostics, bracket coefficients,
  and retained structure constants and adjoint matrices with explicit metadata.
- [ ] Add the classical Killing form, transformation/component selection,
  orthogonalization, and orthonormality checks for families.
- [ ] Ensure operations needing a Lie algebra check the required properties;
  do not turn projected nonclosed brackets into structure constants.
- [ ] Finalize supporting result names and array/metadata accessors within
  syntaqs-lie; do not introduce general map abstractions in core/export.

**Completion gate:** Pauli su(2) gives cyclic structure constants 2 and
`K=-8 I_3`; an abelian algebra gives zero Killing form. Check antisymmetry,
Jacobi, direct versus coefficient bracket actions, and adjoint column order.
For real invertible U, check `K_new=U^T K_old U`. Independent dense references
cover nonorthonormal inputs, transform reconstruction, closed/nonclosed cases,
and f32/f64. Compare a supported graded example with its ordinary action matrix
to verify signs while retaining the ordinary Lie bracket.

## Step 4: Adjoint commutant and center

- [ ] Form real linear constraints for the full adjoint commutant and common
  adjoint kernel. Return matrices in the original family coordinates.
- [ ] Construct internal orthonormal coordinates and their transforms. Compute
  the real symmetric commutant subspace there as a distinct internal object.
- [ ] Extract the full center and its orthogonal complement. Handle absent
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

No new-crate build or feature test has run yet. Record dated completed-step
results here, distinguishing pre-existing failures, regressions, and unverified
environments. The earlier foundation test is recorded in [the index](README.md#verification-record).
