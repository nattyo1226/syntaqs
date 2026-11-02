# syntaqs-spin-half

Spin-1/2 primitive rules, bases, and chain-model helpers for [syntaqs](../../README.md).

## Responsibilities

- Provide `SpinHalfState::{Up, Down}` and `SpinHalfOperator::{I, Sx, Sy, Sz, Sp, Sm}`.
- Implement local products, actions, adjoints, inner products, traces, dimensions, and trivial grading.
- Provide `SzBasis` for states and `CartesianBasis` / `LadderBasis` for operators.
- Construct XYZ and Heisenberg Hamiltonians and Neel product states through `models::SpinHalfModels`.

## Conventions

With `hbar = 1`, `Sx`, `Sy`, and `Sz` are Pauli matrices, and physical spins are `J = S/2`. The ladder operators are `Sp = (Sx + i Sy)/2` and `Sm = (Sx - i Sy)/2`. The state basis is ordered `Up, Down`. Cartesian operator-basis elements are orthonormal under `Tr(A†B)/2`.

Model couplings refer to physical spins: the Heisenberg helper constructs `H = -(J/4) sum S_i · S_j`, with positive coupling ferromagnetic. Site spaces and their indices are supplied by the caller.

## Integration

Select coefficients once with `SpinHalf<C>`, an alias for the core's `ExpressionFactory`. Expressions, tensor composition, evaluation, and coordinates use [syntaqs-core](../syntaqs-core/README.md). Chain helpers use [syntaqs-geometry](../syntaqs-geometry/README.md). Normalization and numerical export are supplied by their separate crates.

This crate uses `no_std + alloc`. Local numeric capabilities are requested per operation, and primitives carry no fixed coefficient precision.

See the [crate API overview](src/lib.rs), [two-spin example](../../examples/src/bin/two_spins.rs), and [Heisenberg example](../../examples/src/bin/heisenberg.rs). Installation and a minimal example are in the [project README](../../README.md).
