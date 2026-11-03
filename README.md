# syntaqs

**syntaqs** /ˈsɪn.tæks/ (pronounced like [syntax](https://dictionary.cambridge.org/pronunciation/english/syntax)) is a Rust library for constructing and evaluating quantum states and operators algebraically.

The name stands for **synta**x for **q**uantum **s**ystems. Build local expressions, place them on identified Hilbert-space factors, and combine them into larger systems. Evaluate actions, inner products, expectations, and traces using local algebra rules, then choose a basis when you need coordinates or numerical vectors and matrices.

## What it provides

- Shared, immutable expressions with delayed actions, products, and adjoints, evaluated without constructing full matrices.
- Ordinary and graded tensor spaces with explicit factor identity and ordering.
- Spin-1/2, spin-1, and single-mode fermion algebras, plus traits for defining your own local algebra.
- Basis expansion, re-expansion, and sparse coordinates for states and operators.
- Chain geometries and helpers for XYZ, Heisenberg, BLBQ, and Hubbard models.
- State normalization and dense/sparse export to Vec, ndarray, and sprs containers.

Each system currently uses one pair of state/operator primitive types across its factors. Partial trace, heterogeneous local algebras, and further extensions are tracked in [future work](docs/future-work.md).

## Installation

The workspace declares Rust **1.85** as its minimum version and uses edition **2024**. Install the crates you need directly from Git:

```toml
[dependencies]
syntaqs-core = { git = "https://github.com/nattyo1226/syntaqs" }
syntaqs-spin-half = { git = "https://github.com/nattyo1226/syntaqs" }

num-complex = "0.4.6"
```

These dependencies follow the repository's default branch. To use a fixed revision, add the same `rev` value to each syntaqs Git dependency. Publishing to crates.io is planned; the setup above uses Git.

Normalization and numerical export are separate crates. Add these entries to the same dependency table when needed:

```toml
syntaqs-normalize = { git = "https://github.com/nattyo1226/syntaqs" }
syntaqs-export = { git = "https://github.com/nattyo1226/syntaqs", features = ["ndarray", "sprs"] }
```

`syntaqs-export` always supports Vec output. Its default feature set is empty; `ndarray` and `sprs` independently enable the corresponding external containers. See its [crate README](crates/syntaqs-export/README.md) for the output types.

## Quick start

This example constructs a two-spin product state and measures a product of Pauli Z operators. Put it in `src/main.rs` with the dependencies above.

```rust
use num_complex::Complex;
use syntaqs_core::HilbertSpace;
use syntaqs_spin_half::{SpinHalf, SpinHalfOperator as Op, SpinHalfState as Ket, SzBasis};

type Spin = SpinHalf<Complex<f64>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = HilbertSpace::factor("a");
    let b = HilbertSpace::factor("b");
    let space = a.tensor(&b)?;

    let state = Spin::product_state(&space, [Ket::Up, Ket::Down])?;
    let zz = Spin::operator_product_at([(&a, Op::Sz), (&b, Op::Sz)])?.bind(&space)?;

    assert_eq!(state.expectation(&zz)?, Complex::new(-1.0, 0.0));

    let coordinates = state.coordinates(&SzBasis)?;
    for entry in coordinates.flat_entries() {
        let (index, amplitude) = entry?;
        println!("{index}: {amplitude}");
    }
    Ok(())
}
```

The coefficient type is selected once through `Spin`. The spin-1/2 Cartesian primitives `Sx`, `Sy`, and `Sz` are Pauli matrices; physical spin operators are `J = S/2` with `hbar = 1`. The [spin-half README](crates/syntaqs-spin-half/README.md) describes the conventions and model helpers.

## Crates

| Crate | Responsibility |
| --- | --- |
| [syntaqs-core](crates/syntaqs-core/README.md) | Algebra traits, shared expressions, tensor spaces, evaluation, and basis coordinates. |
| [syntaqs-basic](crates/syntaqs-basic/README.md) | UUID and named indices for identifying factors. |
| [syntaqs-geometry](crates/syntaqs-geometry/README.md) | Site geometries, relations, and connections to caller-owned data. |
| [syntaqs-normalize](crates/syntaqs-normalize/README.md) | Norms, state normalization, and normalized expectations. |
| [syntaqs-export](crates/syntaqs-export/README.md) | Numerical containers and the metadata needed to interpret them. |
| [syntaqs-lie](crates/syntaqs-lie/README.md) | Lie analysis numerical settings and internal dense kernels; operator-family APIs are in progress. |
| [syntaqs-spin-half](crates/syntaqs-spin-half/README.md) | Spin-1/2 primitives, bases, and spin-chain models. |
| [syntaqs-spin-one](crates/syntaqs-spin-one/README.md) | Spin-1 dipoles and quadrupoles, bases, and spin-chain models. |
| [syntaqs-fermion](crates/syntaqs-fermion/README.md) | Single-mode fermions, Majorana bases, and Hubbard chains. |

`syntaqs-core`, `syntaqs-geometry`, `syntaqs-spin-half`, and `syntaqs-fermion` use `no_std + alloc`. Other crates use the standard library. Verification limits, including the declared minimum Rust version and dedicated no_std targets, are documented in [future work](docs/future-work.md#3-outstanding-verification-and-performance).

## Examples and documentation

Run the examples from the repository root:

```sh
cargo run -p examples --bin two_spins
cargo run -p examples --bin heisenberg --features export
cargo run -p examples --bin hubbard --features export
cargo run -p examples --bin spin_one --features export
```

The [examples README](examples/README.md) covers basis expansion, expression formatting, tensor products, and custom geometric relations as well.

- [Architecture](docs/architecture.md): responsibilities and dependency direction.
- [Requirements](docs/requirements.md): mathematical conventions and contracts.
- [Future work](docs/future-work.md): planned extensions and verification limits.
- [Workspace tests](tests/README.md): shared test algebras and integration checks.

The design documents are in English. Generate the API documentation locally with `cargo doc --workspace --all-features --no-deps`.

## Development

```sh
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo check --workspace --all-targets --no-default-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Test optional export backends independently with `cargo test -p syntaqs-export --no-default-features`, adding `--features ndarray` or `--features sprs` for each backend.

## License

Licensed under the [MIT License](LICENSE). Copyright (c) 2026 Haruki SAITO.

The license covers this repository's code, examples, tests, and documentation.
