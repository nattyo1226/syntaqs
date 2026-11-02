# Verification history for v3 release preparation

Last updated: 2026-11-02

This document collects verification records from earlier implementations and v3 adjustments that inform ongoing release preparation. Dates, toolchains, test counts, and structures such as the former umbrella crate reflect their respective times, rather than the current structure or checks rerun during documentation reorganization.

See [verification configuration](../../design/verification.md) for current commands, CI, and outstanding checks, and [feature designs](../../README.md#2-feature-details) for provided contracts.

## 1. Preliminary numerical-export verification (2026-10-27)

After fetching dependencies, acceptance checks on 2026-10-27 ran the following successfully:

- `cargo test --workspace --all-features --offline -j 2 --quiet` (151 tests including doctests) and `cargo test --workspace --no-default-features --offline -j 2 --quiet` (146 tests).
- `cargo test -p syntaqs-export --no-default-features --features ndarray --offline -j 2 --quiet` (15 tests), and the equivalent sprs configuration (16 tests).
- The then-existing umbrella crate with normalization disabled and export enabled (13 tests). Corresponding end-to-end tests now belong to `examples`.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --offline -j 2 -- -D warnings`, `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --offline -j 2`, and syntaqs-export documentation with default features disabled.
- `cargo check -p syntaqs-core --no-default-features --offline -j 2`, and heisenberg runs with `--all-features`, `--no-default-features --features export`, and `--no-default-features`.

Algebraic, dense, and sparse expectations for a six-site open chain were all 1.25. The CSC matrix had shape `(64, 64)` and 224 nonzeros. Current CI checks MSRV on pushes and pull requests to `main`. Dedicated no_std targets and performance benchmarks remain [future work](../../future-work.md#3-outstanding-verification-and-performance).

## 2. Ordinary and graded acceptance checks (2026-10-29)

These checks were performed while implementing the shared ordinary / graded machinery. Concrete fermion and spin-one verification is described in [local-algebra verification](../../design/local-structure.md#86-verification-targets-and-tests) and [spin-one verification](../../design/local-structure.md#96-verification-targets-and-tests). The shared machinery added no production dependencies.

| Subject | Evidence at implementation time |
| --- | --- |
| Kind, identifications, mixed-kind rejection, subsystems, construction entry points, fallible precedence | [tests/tests/core_tensor_products.rs](../../../tests/tests/core_tensor_products.rs) and existing spaces / fallible_construction / systems tests |
| Signs, CAR, action, adjoints, inner products, trace, Hilbert–Schmidt inner products | Two- and three-factor comparisons using a test-only two-state algebra and independent Kronecker matrices, for f32 / f64 and ordinary / graded spaces |
| Act scope, local-embedding homomorphisms, three odd components | Scope / local expression / three odd factor cases in the same tests |
| Intrinsic coordinates, ordinary matrices, re-expansion, enumeration, empty bases | The same tests and [tests/tests/core_basis_expansion.rs](../../../tests/tests/core_basis_expansion.rs), including a nonorthonormal three-state basis in both kinds |
| Dense and sparse export and metadata | Vec / ndarray / sprs matrices and delayed-action vectors in both precisions in [tests/tests/export_tensor_products.rs](../../../tests/tests/export_tensor_products.rs) |
| Normalization and expectations | Independent-matrix comparisons in both precisions in [tests/tests/normalize_tensor_products.rs](../../../tests/tests/normalize_tensor_products.rs) |
| Minimal capabilities, borrows, sharing, sparsity | Migration of existing capability / contract tests; graded action without Product or coefficient arithmetic; 80-factor separated states and local trace; 32-factor allocation limits |
| Deep trees | Existing ordinary regressions and construction, comparison, evaluation, and destruction of 20,000 graded wrapper levels on a 128 KiB stack |
| Spin-half and examples | All primitives even, P=I, matching action and trace in both kinds; execution of [tensor_products.rs](../../../examples/src/bin/tensor_products.rs) |

The nontrivially graded reference is a [test-only two-state algebra](../../../tests/src/graded.rs), separate from the production fermion API. It uses even `|0>`, odd `|1>`, `a=|0><1|`, `P=diag(1,-1)`, and two-mode basis order `|00>,|01>,|10>,|11>`. Independent Kronecker matrices verified `c_1|11>=-|10>`, `c_0 c_1|11>=-|00>`, `c_1 c_0|11>=|00>`, local and inter-mode CAR, and `Tr(c_1†c_1)=2`. Reference matrices were constructed without the evaluator or export, checking intrinsic coordinates and ordinary matrices separately. No approximate zero pruning was used.

The following passed in a Rust 1.98.0 direnv devShell:

- `cargo test --workspace --all-features` (including doctests)
- `cargo check -p syntaqs-core -p syntaqs-geometry --no-default-features`
- `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo run -p examples --bin tensor_products`

Final construction-entry and export contract tests were also rerun. The no_std check was a host build rather than dedicated-target verification. Current [CI](../../../.github/workflows/ci.yml) builds and tests MSRV `1.85.0` on pushes and pull requests to `main`. Dedicated no_std targets and performance benchmarks remain [outstanding](../../future-work.md#3-outstanding-verification-and-performance).

## 3. Implementation and relocation verification

On 2026-10-31, geometry and relation implementation and acceptance checks completed in a Rust 1.99.1 direnv devShell. Successful checks included workspace tests with all features and without default features; 13 final geometry tests and doctests; three numerical comparisons for many-body terms, Pauli strings, and graded products; formatting; all-target Clippy; and rustdoc with warnings denied. Boundary tests added after the all-feature workspace run also passed in geometry alone and the no-default-feature workspace.

Relocation to examples included running both examples. Eleven tests without features and thirteen with all features passed, along with formatting and all-target Clippy for both configurations. Two tests involving Pauli-matrix comparisons require `export`. These are implementation and relocation records; tests were not rerun for documentation reorganization.

```sh
direnv exec . cargo test -p syntaqs-geometry --all-features --offline
direnv exec . cargo test -p examples --all-features --offline --test geometry_relations
direnv exec . cargo test -p examples --all-features --offline --bin geometry_relations --bin pauli_strings
direnv exec . cargo test --workspace --all-features --offline -j2
direnv exec . cargo test --workspace --no-default-features --offline -j2
direnv exec . cargo fmt --all --check
direnv exec . cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
direnv exec . sh -c 'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --offline'
```

Current [CI](../../../.github/workflows/ci.yml) builds and tests MSRV `1.85.0` on pushes and pull requests to `main`. Dedicated no_std target attempts during implementation and relocation were blocked by dependency resolution and network restrictions, and are not counted as successful verification. Performance measurement, cache-versus-regeneration comparisons, and a formal periodic square-lattice implementation also remain unimplemented. Shared outstanding checks are in [future work](../../future-work.md#3-outstanding-verification-and-performance).

## 4. Normalization-change verification

Implementation and acceptance verification ran on 2026-11-01 in a Rust 1.99.1 direnv devShell. This section records checks at implementation time, rather than tests rerun for documentation reorganization. All following commands passed from the repository root with `direnv exec .`:

```sh
cargo test --workspace --all-features --no-fail-fast --quiet
cargo test --workspace --no-default-features --offline --no-fail-fast --quiet
cargo test -p syntaqs-spin-one -p syntaqs-fermion --all-features --offline --quiet
cargo test -p syntaqs-spin-one --test tensor_products -p syntaqs-fermion --test models --offline --quiet
cargo check -p syntaqs-core -p syntaqs-geometry --no-default-features --offline
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --offline -- -D warnings
sh -c 'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --offline'
```

syntaqs-export also ran `cargo test -p syntaqs-export --no-default-features --offline --quiet` in three configurations: no additional features, `--features ndarray`, and `--features sprs`. Workspace all-feature tests covered both backends enabled together.

The checks covered:

- All primitives' products, actions, adjoints, ordinary trace, both operator-basis transformations, and coordinates against independent matrices for f32 / f64. Spin-one checks included all 225 products and 45 actions, associativity, and compatible action.
- Local normalized HS Gram matrices, spin-one Cartesian / Ladder tensor-basis Gram matrices, and fermion graded Majorana tensor-basis Gram matrices. Graded signs and CAR were also checked against ordinary action matrices.
- [HS inner-product regressions](../../../tests/tests/core_hs_inner_product.rs): explicit-matrix `Tr(A†B)/D`, empty spaces, added spectators, explicit-space binding overrides, and `⟨I,I⟩=1` for 2048 factors. They also verified no state type, Dimension, or integer-conversion requirement; coefficients with only Divide support; and no division requirement on ordinary trace.
- Preservation of independent physical-model matrices, energies, and constant shifts; tiny coefficients and exact cancellation; and empty results for analytically zero local rules. No implicit approximate pruning was added.

All eight example binaries ran through `cargo run -p examples --bin <name> --offline --quiet` with default features, `--all-features`, and `--no-default-features`. Preserved results included singlet / triplet energies −0.75 / 0.25, symmetric Hubbard energy −2, spin-one BLBQ expectation 0.5, and 37 Pauli-string terms with matching dense and sparse export.

Dependencies and feature requirements were unchanged. Current [CI](../../../.github/workflows/ci.yml) builds and tests MSRV `1.85.0` on pushes and pull requests to `main`. Actual builds for a dedicated no_std target remain [unverified](../../future-work.md#3-outstanding-verification-and-performance).

## 5. Expression-formatting verification (2026-11-01)

Checks ran on 2026-11-01 with Rust 1.99.1 / aarch64-apple-darwin in the direnv devShell. Tests and doctests for the five relevant crates and the following workspace checks passed:

```sh
direnv exec . cargo test --workspace --all-features --offline --quiet
direnv exec . cargo check --workspace --all-targets --no-default-features --offline
direnv exec . cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
direnv exec . cargo fmt --all -- --check
direnv exec . cargo doc --workspace --all-features --no-deps --offline
direnv exec . cargo run -p examples --no-default-features --bin expression_formatting --offline
```

[Core formatting.rs](../../../crates/syntaqs-core/tests/formatting.rs) covers syntax, bounds, errors, and depth regressions; [basic indices.rs](../../../crates/syntaqs-basic/tests/indices.rs) covers Tag and Id128; each builtin crate's `tests/formatting.rs` covers every primitive variant. Local and composite actions, sums, scaling, products, adjoints, and mixed tensors at depth 20,000 were displayed and destroyed on a 128 KiB stack. Internal tests also checked combining deep simple tensors into kets and classifying shared subexpressions. Final depth, Debug, and classification additions received targeted reruns.

In these acceptance checks, no_std support was checked on the host; a dedicated-target build was not performed because the environment contained only host-target libraries. Current [CI](../../../.github/workflows/ci.yml) builds and tests MSRV `1.85.0` on pushes and pull requests to `main`. Dedicated no_std targets, benchmarks, and speed comparisons remain [future work](../../future-work.md#3-outstanding-verification-and-performance).
