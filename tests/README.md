# syntaqs workspace tests

`syntaqs-tests` supplies shared test algebras, independent numerical references,
and public-API integration tests. Test an individual library for its own
contracts, or this package for workflows that cross library boundaries.
The package is unpublished; production crates do not depend on it.

## Run the shared tests

Run from the repository root:

```sh
cargo test -p syntaqs-tests
cargo test --workspace --all-features
```

The first command covers core, normalize, and Vec export. The second includes
all libraries, example tests, export backends, and Lie analysis, requiring system
OpenBLAS with LAPACK and pkg-config. Full commands and toolchain selection are
specified in [verification](../docs/design/verification.md#2-common-checks).

## Select a backend or Lie analysis

This package has no default features. Test optional backends separately to avoid
workspace dependency-feature unification:

```sh
cargo test -p syntaqs-tests --no-default-features --features ndarray
cargo test -p syntaqs-tests --no-default-features --features sprs
cargo test -p syntaqs-tests --all-features
```

`ndarray` and `sprs` enable the matching export APIs. The optional `lie` feature
adds syntaqs-lie and the internal native provider; default and backend-only
configurations do not select that provider. With native dependencies installed,
run either shared Lie target independently:

```sh
cargo test -p syntaqs-tests --no-default-features --features lie --test lie_analysis
cargo test -p syntaqs-tests --no-default-features --features lie --test lie_decomposition
```

## Fixtures and coverage

- `src/three_level.rs` defines a three-state algebra and nonorthonormal bases.
- `src/graded.rs` defines a two-state algebra with parity for independent sign checks.
- Tests under `tests/` use these fixtures with core, normalize, export, and Lie.
  Filename prefixes identify the main component under test.

Reference calculations use independent matrices and Fock-space rules rather
than production evaluation/export. Lie workflows cover f32/f64, operator-only
adapters, large spectators, and su(3)/mixed decomposition; their detailed targets
are maintained in the [Lie design](../docs/design/lie.md#8-verification).

Crate-specific fixtures and internal tests stay in their library crate. Shared
fixtures belong here so tests can import them through an ordinary library,
without reaching into another crate's `tests/` directory. For a released version,
use the tests and documentation from the [same tag](../docs/installation.md).
