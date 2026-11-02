# syntaqs-fermion

Single-mode fermion rules, Majorana bases, and Hubbard-chain helpers for [syntaqs](../../README.md).

## Responsibilities

- Provide `FermionState::{Empty, Occupied}` and the operators `I`, `Annihilate`, `Create`, `Number`, `Mx`, `My`, and `Mz`.
- Implement local algebra rules, dimensions, and homogeneous parity information.
- Provide `OccupationBasis` for states and `LadderBasis` / `MajoranaBasis` for operators.
- Construct standard and symmetric Hubbard Hamiltonians through `models::FermionModels`.

## Conventions

For `a = Annihilate`, `Mx = a + a†`, `My = (a - a†)/i`, and `Mz = I - 2n` is the parity operator. Mx and My are odd; Mz is even. Majorana operator-basis elements are orthonormal under `Tr(A†B)/2`. The state basis is ordered `Empty, Occupied`.

Multiple modes use graded tensor spaces. The common core evaluator supplies exchange signs from factor order and primitive parity. Local primitives do not store mode indices or Jordan–Wigner strings.

## Integration

`Fermion<C>` selects primitives and coefficients through the core's `ExpressionFactory`. Space construction, shared expressions, graded evaluation, and coordinates belong to [syntaqs-core](../syntaqs-core/README.md).

Hubbard helpers use [syntaqs-geometry](../syntaqs-geometry/README.md) and expect each site to contain two graded leaves in `(up, down)` order. The standard interaction is `U n_up n_down`; the symmetric interaction is `U (n_up - I/2)(n_down - I/2)`, including its constant shift.

This crate uses `no_std + alloc`; local numeric requirements are operation-specific.

See the [crate API overview](src/lib.rs), [fermion design](../../docs/design/local-structure.md#8-fermion-local-algebra), and [Hubbard example](../../examples/src/bin/hubbard.rs). Installation is covered by the [project README](../../README.md).
