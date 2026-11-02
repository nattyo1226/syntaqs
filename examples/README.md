# syntaqs examples

See the [project README](../README.md) for an overview and Git dependency setup.

This workspace package contains runnable examples as Cargo binaries. Each example uses crates such as `syntaqs-core`, `syntaqs-basic`, `syntaqs-geometry`, and `syntaqs-spin-half` directly, inheriting dependency settings from the workspace.

Run these commands from the repository root:

```sh
cargo run -p examples --bin expression_formatting
cargo run -p examples --bin two_spins
cargo run -p examples --bin tensor_products
cargo run -p examples --bin basis_expansion
cargo run -p examples --bin heisenberg
cargo run -p examples --bin heisenberg --features export
cargo run -p examples --bin hubbard
cargo run -p examples --bin hubbard --features export
cargo run -p examples --bin spin_one
cargo run -p examples --bin spin_one --features export
cargo run -p examples --bin geometry_relations
cargo run -p examples --bin pauli_strings
```

`expression_formatting` demonstrates Tag and Id128 indices, ordinary Display, and multiline formatting with `{:#}`. It prints product-state kets, tensors containing local superpositions, the three-factor GHZ state `(|up,up,up> + |down,down,down>)/√2`, and a delayed action on that state. It covers coefficients, actions, adjoints, and the eight-character index limit used only in expression Display. It also constructs a four-site open Heisenberg chain with tags `s0` through `s3` and a transverse field, `H = -(1/4) Σ⟨i,j⟩ S_i·S_j - 0.2 Σᵢ Sx_i`, and compares ordinary and multiline formatting of the same expression. Here Sx, Sy, and Sz are Pauli matrices. The example does not print Debug output. It needs no additional features and also runs with `--no-default-features`. See the [expression formatting design](../docs/design/expression-formatting.md) for the formatting contract.

`two_spins` constructs two-spin singlet and triplet states and the exchange Hamiltonian `H = J_a · J_b = (S_a · S_b)/4` using `Spin::product_state`, `Spin::operator_product_at`, and `Spin::try_operator_sum`. It computes and checks squared norms, energy expectations (−0.75 for the singlet and 0.25 for the triplet), eigenstate residuals, and the overlap between the states. It needs no additional features and also runs with `--no-default-features`.

`hubbard` uses `Fermion<Complex<f64>>` and graded spaces with two modes per site, ordered `(up,down)`. It constructs the standard and symmetric Hubbard models on two sites at `t=1,U=4` and checks that their expectations in `|up,down>` are 0 and −2, respectively. It also obtains basis coordinates of the parity operator `Mz=I−2n`. The default `normalize` feature enables normalization and normalized expectations; the optional `export` feature checks agreement between 16×16 Vec and sprs matrices. It also runs with `--no-default-features`. Indices are issued by the caller.

`spin_one` uses the three states and quadrupoles of `SpinOne<Complex<f64>>` to construct a local Hamiltonian with single-ion anisotropy and a transverse field. It checks ladder coordinates of `Sp²=√(3/2) Q2p`, round trips between Cartesian and ladder bases, and actions on two factors. It constructs two-site XYZ, Heisenberg, and BLBQ models through associated functions and checks that the BLBQ expectation in `|+,->` is 0.5 at `J=1,K=0.25`. Positive bilinear coupling is ferromagnetic; the periodic edge on two sites is counted once. The default `normalize` feature enables normalization and normalized expectations. The `export` feature checks agreement between Vec and sprs matrices for the local 3×3 Hamiltonian and the 9×9 BLBQ Hamiltonian. It also runs with `--no-default-features`.

`geometry_relations` uses `SiteMapper` and `Relation<G>` to enumerate nearest-neighbor pairs and consecutive three-site groups on the same Chain. Its square-lattice example shows how to implement custom Geometry and Relation types and a Group that retains its center and direction. It also covers `leaves_cartesian()`, `leaves_zip()`, and `combine_leaves()` for sites containing multiple leaves. The square lattice is implemented only within this example.

`pauli_strings` enumerates Pauli strings supported on at most two sites, including the identity, with one spin-1/2 per site. It constructs 37 terms on three sites, using `Sx/Sy/Sz` directly as Pauli matrices. All combinations of support sites are enumerated using ordinary iterators, separately from geometric Relation types. It also demonstrates construction without coefficient evaluation capabilities or Clone on indices.

These two examples also run with `--no-default-features`. Run their binary unit tests with the following commands. The two Pauli-string matrix comparison tests require the `export` feature.

```sh
cargo test -p examples --no-default-features --bin geometry_relations --bin pauli_strings
cargo test -p examples --features export --bin geometry_relations --bin pauli_strings
```

`normalize` is enabled by default; `export` is optional. You can also run without normalization:

```sh
cargo run -p examples --bin heisenberg --no-default-features
cargo run -p examples --bin heisenberg --no-default-features --features export
```

`basis_expansion` selects the coefficient type once with `type Spin = SpinHalf<Complex<f64>>` and constructs local expressions with `Spin::state` and `Spin::operator` through the ExpressionFactory type alias. `heisenberg` issues indices in caller code, builds local spaces in site order, and uses `SpinHalfModels<Spin>::heisenberg` and `neel` through normalization and dense/sparse export. To switch to single precision, change the Spin definition to `Complex<f32>`.

Use `syntaqs_core::StateOf<Spin,I>` and `OperatorOf<Spin,I>` for return type annotations, or `LocalStateOf<Spin>` and `LocalOperatorOf<Spin>` for local expressions. For matrix export, `MatrixArgs::<SpinHalfState,_>` selects the state primitive acted on, rather than specifying coefficient precision again. The output container is selected through a return type such as `Exported<Vec<_>,_>`.

The previous `Chain<I>`, `ToSpace`, and free functions in `syntaqs_spin_half::utility` have been removed. Construct a Chain with `Chain::new(n, boundary)` and pass an external `&[HilbertSpace<I>]` for the site assignment.

Expression construction and collection use `Spin::state_sum`, `Spin::tensor_in`, `Spin::operator_sum`, and `Spin::operator_product`. State constructors require an explicit existing space: `product_state(&space, primitives)` places primitives in leaf order, and `tensor_in(&space, states)` combines placed states. The operator `_at` constructor accepts a sequence of `(&factor, primitive)` assignments. The `try_` collection variants accept Results, give precedence to the first input ExpressionError, and stop pulling subsequent elements. They validate spaces and bindings after all inputs succeed.

`scaled` accepts `impl Into<C>`, so you can pass real values directly, such as `.scaled(0.5)`, after selecting a complex coefficient type. Replace `.scaled(value.into())` calls that relied on the previous expected type with `.scaled(value)`. Specify the type of Default values or the constructor's C when inference is insufficient. Scalar multiplication with `*` continues to take the coefficient type directly.

`tensor_products` selects TensorKind when constructing a space and uses the common action and trace APIs. All spin-half primitives are even and P=I, so ordinary and graded spaces give the same numerical results. See the [ordinary/graded evaluation design](../docs/design/composite-expression.md#12-ordinary--graded-の共通評価と移行) for the common evaluator's contract.
