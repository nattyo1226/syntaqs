# syntaqs workspace tests

See the [project README](../README.md) for an overview and development commands.

`syntaqs-tests` is a workspace package containing shared test algebras, independent numerical reference calculations, and public API and integration tests that use them. Like `examples`, it inherits version, edition, and rust-version from the workspace and sets `publish = false`.

`src/graded.rs` provides a two-state algebra with parity; `src/three_level.rs` provides a three-state algebra with support for non-orthonormal bases. Tests under `tests/` import this library to exercise the public APIs of core, normalize, export, and (with the optional `lie` feature) Lie analysis. Filename prefixes `core_`, `normalize_`, `export_`, and `lie_` identify the main component under test. Tests of library internals and fixtures specific to a crate remain in that crate.

Production crates do not depend on `syntaqs-tests`. Keeping tests that use shared fixtures in this package avoids importing other crates' `tests/` directories through relative paths and removes the need to publish a test-support library. Testing an individual library with `cargo test` is distinct from validating the entire workspace.

Run these commands from the repository root:

```sh
# Test core, normalize, and Vec export
cargo test -p syntaqs-tests

# Test all backends and Lie analysis (requires BLAS/LAPACK)
cargo test -p syntaqs-tests --all-features

# Test each optional backend separately
cargo test -p syntaqs-tests --no-default-features --features ndarray
cargo test -p syntaqs-tests --no-default-features --features sprs

# Test Lie without enabling either export backend
cargo test -p syntaqs-tests --no-default-features --features lie --test lie_analysis

# Test all crates, examples, and workspace integration tests
cargo test --workspace --all-features
```

`syntaqs-tests` has no default features. Its `ndarray` and `sprs` features enable the corresponding features in `syntaqs-export`. Use `-p syntaqs-tests` to test individual backend configurations. When testing the entire workspace, dependency settings in other packages may enable additional backends.

The `lie` feature enables the unpublished test package's optional `syntaqs-lie`
dependency and the centralized development BLAS provider. It requires system
OpenBLAS (including LAPACK) and pkg-config; the default and backend-only test
configurations do not select this provider. Production libraries still do not
depend on this package.

[lie_analysis.rs](tests/lie_analysis.rs) exercises family validation, projection,
retained coefficients, closure, adjoint/Killing representations, commutants,
center extraction, decomposition, and component reconstruction as a single
public workflow at f32/f64. It compares spin/fermion and custom three-level/graded
algebras with independently assembled matrices. Operator-only adapters provide
no primitive Clone, Dimension, Basis, state, or action capabilities. An 80-factor
case verifies that the full workflow needs no full physical action matrix.
