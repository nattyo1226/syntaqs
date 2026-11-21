# syntaqs

A Rust library for building quantum states and operators algebraically. Combine
local spin or fermion expressions into larger systems, evaluate them without
full matrices, and choose a basis when you need numerical output.

## Quick start

Use Rust 1.85 or newer. Create a project beside the `syntaqs` checkout by
running these commands from its parent directory.

```sh
cargo new --edition 2024 two-spins
cd two-spins
```

Add these entries to the `[dependencies]` section of `Cargo.toml` to use the
local checkout. For a released version, follow the
[installation guide](docs/installation.md).

```toml
syntaqs-core = { path = "../syntaqs/crates/syntaqs-core" }
syntaqs-spin-half = { path = "../syntaqs/crates/syntaqs-spin-half" }

num-complex = "0.4.6"
```

Save the two-spin calculation below as `src/main.rs`.

```rust
use num_complex::Complex;
use syntaqs_core::space::HilbertSpace;
use syntaqs_spin_half::{
    SpinHalf, operator::SpinHalfOperator as Op, state::SpinHalfState as Ket,
};

type Spin = SpinHalf<Complex<f64>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = HilbertSpace::factor("a");
    let b = HilbertSpace::factor("b");
    let space = a.tensor(&b)?;
    let state = Spin::state_tensor_from([Ket::Up, Ket::Down], &space)?;
    let zz = Spin::operator_product_from([(Op::Sz, &a), (Op::Sz, &b)])?.bind(&space)?;

    println!("Expectation: {}", state.expectation(&zz)?.re);
    Ok(())
}
```

Run `cargo run` to print `Expectation: -1`. Here `Sz` is Pauli Z, so the
product has expectation −1 in `|up,down>`.

## Crates

| Crate | Purpose |
| --- | --- |
| [syntaqs-core](crates/syntaqs-core/README.md) | Algebra, expressions, spaces, and basis coordinates. |
| [syntaqs-basic](crates/syntaqs-basic/README.md) | UUID and named factor identifiers. |
| [syntaqs-geometry](crates/syntaqs-geometry/README.md) | Chains, site relations, and data mapping. |
| [syntaqs-normalize](crates/syntaqs-normalize/README.md) | Norms, normalization, and normalized expectations. |
| [syntaqs-export](crates/syntaqs-export/README.md) | Vec, ndarray, and sprs output. |
| [syntaqs-lie](crates/syntaqs-lie/README.md) | Operator-family projection and Lie analysis. |
| [syntaqs-spin-half](crates/syntaqs-spin-half/README.md) | Spin-1/2 algebra and chain models. |
| [syntaqs-spin-one](crates/syntaqs-spin-one/README.md) | Spin-1 algebra and chain models. |
| [syntaqs-fermion](crates/syntaqs-fermion/README.md) | Fermion algebra and Hubbard chains. |

## Docs

- [Installation](docs/installation.md): version tags, optional crates, and features.
- [Examples](examples/README.md): runnable calculations and models.
- [Documentation guide](docs/README.md): requirements, architecture, and feature contracts.
- [Development checks](docs/design/verification.md): tests, toolchains, and native dependencies.

## License

[MIT](LICENSE). Copyright (c) 2026 Haruki SAITO.
