# syntaqs workspace tests

`syntaqs-tests` is a workspace package containing shared test algebras, independent numerical reference calculations, and public API and integration tests that use them. Like `examples`, it inherits version, edition, and rust-version from the workspace and sets `publish = false`.

`src/graded.rs` provides a two-state algebra with parity; `src/three_level.rs` provides a three-state algebra with support for non-orthonormal bases. Tests under `tests/` import this library to exercise the public APIs of core, normalize, and export. Filename prefixes `core_`, `normalize_`, and `export_` identify the main component under test. Tests of library internals and fixtures specific to a crate remain in that crate.

Production crates do not depend on `syntaqs-tests`. Keeping tests that use shared fixtures in this package avoids importing other crates' `tests/` directories through relative paths and removes the need to publish a test-support library. Testing an individual library with `cargo test` is distinct from validating the entire workspace.

Run these commands from the repository root:

```sh
# Test core, normalize, and Vec export
cargo test -p syntaqs-tests

# Test all backends, including ndarray and sprs
cargo test -p syntaqs-tests --all-features

# Test each optional backend separately
cargo test -p syntaqs-tests --no-default-features --features ndarray
cargo test -p syntaqs-tests --no-default-features --features sprs

# Test all crates, examples, and workspace integration tests
cargo test --workspace --all-features
```

`syntaqs-tests` has no default features. Its `ndarray` and `sprs` features enable the corresponding features in `syntaqs-export`. Use `-p syntaqs-tests` to test individual backend configurations. When testing the entire workspace, dependency settings in other packages may enable additional backends.
