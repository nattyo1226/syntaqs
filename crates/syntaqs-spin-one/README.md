# syntaqs-spin-one

Spin-1 dipoles, quadrupoles, bases, and chain-model helpers for [syntaqs](../../README.md).

## Responsibilities

- Provide `SpinOneState::{Plus, Zero, Minus}` and fifteen operator primitives spanning a nine-dimensional operator space.
- Implement local products, actions, adjoints, inner products, traces, dimensions, and trivial grading.
- Provide `SzBasis` for states and complete `CartesianBasis` / `LadderBasis` operator bases.
- Construct XYZ, Heisenberg, and bilinear-biquadratic (BLBQ) Hamiltonians through `models::SpinOneModels`.

## Conventions

With `hbar = 1`, physical `Jz` has eigenvalues `1, 0, -1` in state order `Plus, Zero, Minus`. Cartesian primitives use `S = sqrt(3/2) J`, and Cartesian operator-basis elements are orthonormal under `Tr(A†B)/3`. The normalized state `SpinOneState::Zero` is distinct from a zero vector.

Model parameters retain their physical-spin meaning. The helpers use negative bilinear coupling, so positive coupling is ferromagnetic. BLBQ constructs `-sum [j (J_a · J_b) + k (J_a · J_b)^2]` for each coupled leaf pair, without discarding constant terms.

## Integration

`SpinOne<C>` selects primitive and coefficient types through the core's `ExpressionFactory`. Shared expressions, tensor composition, evaluation, and coordinates belong to [syntaqs-core](../syntaqs-core/README.md); model helpers use [syntaqs-geometry](../syntaqs-geometry/README.md).

This crate uses the standard library. Local products and state actions use `Complex<R>` with `R: num_traits::Float`; other capabilities use their own numeric bounds. Model construction remains delayed and does not require local evaluation.

See the [crate API overview](src/lib.rs), [spin-one design](../../docs/design/local-structure.md#9-spin-one-local-algebra), and [spin-one example](../../examples/src/bin/spin_one.rs). Installation is covered by the [project README](../../README.md).
