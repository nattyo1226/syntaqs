# syntaqs examples

Run these workspace binaries to learn expression construction, model building,
basis conversion, and Lie analysis. Start with `two_spins` for a complete small
calculation or `heisenberg` for a chain model.

For release use, [install with a version tag](../docs/installation.md) and run
examples from that tag. The commands and API paths here describe this checkout.
Every binary depends directly on the library crates it uses.

## Choose an example

Run from the repository root, for example:

```sh
cargo run -p examples --bin two_spins
cargo run -p examples --bin heisenberg --features export
```

| Binary | What it demonstrates |
| --- | --- |
| `two_spins` | Singlet/triplet states, exchange Hamiltonian, norms, energy expectations −0.75/0.25, and eigenstate residuals. |
| `basis_expansion` | Local and composite expansion, coordinates, and coefficient-type selection through `SpinHalf<C>`. |
| `tensor_products` | Ordinary/graded spaces with the common action and trace APIs. Spin-half primitives are even, so both kinds agree. |
| `expression_formatting` | Tag/UUID labels, Display, multiline `{:#}` output, product and GHZ states, delayed actions, and a four-site Hamiltonian. |
| `heisenberg` | Caller-issued indices, site spaces, a Heisenberg model, a Neel state, normalization, and dense/sparse export. |
| `hubbard` | Two graded modes per site, standard/symmetric models, parity coordinates, and Vec/sprs output. |
| `spin_one` | Dipoles/quadrupoles, basis round trips, local anisotropy, XYZ/Heisenberg/BLBQ chains, and numerical output. |
| `geometry_relations` | Multiple relations on one chain, a custom square lattice, role metadata, and multi-leaf site mapping. |
| `pauli_strings` | All supports of at most two sites on three spins: 37 strings including identity, with ordinary support iterators. |
| `lie_algebra` | Family validation, projection, closure, retained Lie representations, and center/simple-ideal decomposition. |

Replace the binary name in the first command to run any example. The following
features control optional operations rather than the underlying expression APIs.

## Features and native requirements

| Feature | Default? | Enables |
| --- | --- | --- |
| `normalize` | Yes | Norms, state normalization, and normalized expectations in model examples. |
| `export` | No | Dense/sparse output using both ndarray and sprs. |
| `lie` | No | Lie analysis and the internal system BLAS/LAPACK provider. |

The Lie example needs system OpenBLAS supplying BLAS and LAPACK, plus pkg-config.
Its feature is independent of normalization and export:

```sh
cargo run -p examples --no-default-features --features lie --bin lie_algebra
```

It analyzes su(2) + su(2) + u(1), recovering center dimension 1 and simple-ideal
dimensions `[3,3]`. Residual and seed settings are explicit, and the workflow
uses neither a state basis nor physical action matrices. Downstream applications
choose their own [native provider](../crates/syntaqs-lie/README.md#native-linkage).

To run the other examples without normalization, use `--no-default-features`.
Add export independently when desired:

```sh
cargo run -p examples --no-default-features --bin heisenberg
cargo run -p examples --no-default-features --features export --bin heisenberg
```

## Reading and adapting the code

Select coefficient precision once, for example
`type Spin = SpinHalf<Complex<f64>>`; use `Complex<f32>` for single precision.
Type aliases `StateOf<Spin, I>`, `OperatorOf<Spin, I>`, and their local variants
are available in `syntaqs_core::system`. Matrix export separately selects its
state primitive with `MatrixArgs::<SpinHalfState, _>` and its container through
the result type.

Create spaces with `HilbertSpace`, then construct states on those spaces and
place operators on factors. `scaled(impl Into<C>)` accepts matching real values
such as `.scaled(0.5)`. For fallible collection, binding, inference, and error
precedence, see the [construction contract](../docs/design/expression-construction.md).

The [example sources](src/bin) contain complete setup and assertions. The
[model contracts](../docs/design/models.md) define physical coupling scales and
constant shifts; [formatting](../docs/design/expression-formatting.md) defines
Display syntax and its distinction from Debug.

## Testing examples

Binary tests for the geometry and Pauli-string examples run as follows. The two
Pauli-string matrix comparison tests need export:

```sh
cargo test -p examples --no-default-features --bin geometry_relations --bin pauli_strings
cargo test -p examples --features export --bin geometry_relations --bin pauli_strings
```

See [workspace verification](../docs/design/verification.md) for the complete
feature/toolchain matrix and native linkage checks.
