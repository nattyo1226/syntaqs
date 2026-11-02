# syntaqs requirements

Updated: 2026-11-02

## 1. Purpose

syntaqs is a Rust library for constructing, composing, and evaluating quantum Hilbert spaces, states, and operators algebraically, with numerical output when needed. Its goal is to use local algebra and expression structure to compute actions, inner products, expectations, and traces before constructing numerical matrices.

This document defines what the project builds. See [architecture](architecture.md) for crates and overall structure, [feature designs](README.md#2-feature-details) for concrete types, public APIs, and mathematical conventions, [future work](future-work.md) for features to implement, and [plans](plans/v3/README.md) for ongoing work.

## 2. Scope

- Support finite-dimensional local quantum systems and their ordinary or graded tensor products.
- Distinguish geometry, factor identity, factor order, local algebra, and the selected basis. Allow spaces and expressions to be constructed without geometry.
- Connect concrete local rules to shared expression and evaluation machinery. Custom algebras must not need their own expression trees or evaluators.
- Separate algebraic computation from numerical containers so matrix-free, dense, and sparse calculations use consistent conventions.
- Delegate arithmetic, rounding, and special values to the numeric type. Automatic differentiation, eigensolvers, and time evolution are outside the current scope.

Within one system, every factor currently uses the same state and operator primitive types. A composite basis applies one local basis to all factors. Effective mixtures of ordinary and graded products after removing unit and singleton wrappers, implicit factor reordering, and implicit projection onto subspaces are unsupported.

## 3. Functional requirements

### 3.1. Local algebras and extensibility

- Provide states, operators, local rules, and bases for spin-1/2, spin-1, and single-mode fermions.
- Return finite linear combinations from local products, actions, and adjoints; do not require all primitives to be linearly independent.
- Provide state inner products, ordinary traces, identity operators, and state/operator space dimensions.
- For graded local algebras, provide homogeneous primitive parity and a parity operator; represent mixed parity by expression sums.
- Request numeric capabilities per operation without imposing evaluation requirements on storage, sharing, or delayed construction.

See [local algebras](design/local-structure.md) and [local expressions](design/primitive-expression.md).

### 3.2. Degrees of freedom and Hilbert spaces

- Identify basic factors with existing identifiers, separating identity comparison from identifier generation.
- Construct ordered recursive spaces and enumerate children, leaves, and identifiers without changing their order.
- Reject duplicate tensor factors and incompatible product kinds. Grouping alone must not change mathematical compatibility.
- Provide space compatibility and subsystem checks that preserve relative factor order.
- Treat the empty tensor space as a one-dimensional unit, distinct from an indexed one-dimensional factor.
- Do not reject space construction merely because its total dimension is unrepresentable; detect overflow when dimensions are requested.

See [indices](design/index.md) and [Hilbert spaces](design/hilbert-space.md).

### 3.3. Expression construction, placement, and evaluation

- For local and composite state/operator expressions, provide finite sums, scaling, and zero; operator products and adjoints; operator action; and tensor composition of states.
- Distinguish placing a local expression on a factor from binding it to a full ambient space, including spectators.
- Retain a state's space and resolve an operator's ambient space explicitly or from its binding. Explicit spaces take precedence over existing bindings.
- Return errors for invalid placement, factor references, and incompatible spaces when the required information becomes available. Do not skip validation for zero coefficients or identities.
- Construct delayed expressions and evaluate actions, inner products, expectations, ordinary full traces, and dimension-normalized Hilbert–Schmidt inner products without requiring a basis or full matrices.
- Use conjugate-linearity on the left of inner products and right-first operator action. Graded exchange, action, and adjoint rules follow factor order and parity while preserving the ordinary positive-definite inner product and ordinary trace.
- Preserve the selected space after evaluation or expansion without modifying input expressions.
- Provide state norms, normalization, and normalized expectations; reject exactly zero norms for normalization and normalized expectations.
- Support ordinary and multiline mathematical formatting and structural Debug output without evaluating expressions.

See [construction](design/expression-construction.md), [composite expressions](design/composite-expression.md), [normalization](design/normalization.md), and [formatting](design/expression-formatting.md).

### 3.4. Bases and numerical output

- Support complete, linearly independent state and operator bases without requiring orthonormality.
- Provide basis expansion rules, indices, and lazy enumeration, extending them to whole expressions and tensor-product bases through shared machinery.
- Return ordinary expressions from basis expansion, support re-expansion in another basis, and provide aggregated sparse coordinates.
- Remove only exact zeros during coordinate aggregation; preserve small nonzero components.
- Export states as dense/sparse vectors and operators as dense/sparse action matrices. Distinguish operator-basis coordinates from action matrices in a state basis.
- Use consistent bases and factor order for dense and sparse output, retaining the information needed to interpret basis, space, shape, layout, and coefficient precision.
- Avoid dense vector/matrix intermediates for sparse output. Distinguish expansion coordinates from inner products in non-orthonormal bases.

See [bases](design/basis.md) and [export](design/export.md).

### 3.5. Geometry and model construction

- Support one-dimensional chains with site count, order, and boundary conditions, nearest-neighbor edges, and ordered many-site relations.
- Borrow caller-owned site-ordered data to connect it to geometry, validating lengths and referenced positions.
- Support multiple leaves, nesting, and empty spaces at sites; distinguish site order from the full space's factor order.
- Construct spin-1/2 XYZ, Heisenberg, and Neel states; spin-1 XYZ, Heisenberg, and BLBQ models; and standard/symmetric fermionic Hubbard models as ordinary expressions.
- Preserve the physical meaning of couplings by compensating for primitive normalization. Do not silently discard constant shifts.
- Specify boundary conventions for zero, one, and two sites, and validate assignments even without edges or nonzero couplings.

See [geometry](design/geometry.md) and [models](design/models.md).

## 4. Non-functional requirements

- Support single- and double-precision complex coefficients without implicit precision mixing or promotion. Connect custom coefficients through the required capabilities.
- Share immutable expressions and spaces; do not deep-copy entire operands when composing or cloning handles.
- Do not require dense storage proportional to the full Hilbert-space dimension before explicit dense export. No general bound on expression growth or optimal complexity is guaranteed.
- Traverse, evaluate, and destroy deep expressions/spaces without recursion proportional to depth. User-defined formatters and recursive Debug output have separate limitations.
- Keep the algebra core independent of geometry, randomness, concrete algebras, normalization, and numerical backends, maintaining `no_std + alloc`.
- Target Rust 1.85 and edition 2024 without unsafe code.

See [numeric conventions](design/numerics.md) for numerical contracts and [verification](design/verification.md) for commands and environment limitations.

## 5. Acceptance criteria

The representative end-to-end scenario constructs a one-dimensional spin-1/2 chain and factor spaces, builds its Heisenberg Hamiltonian and a Neel product state, and evaluates the energy without generating a matrix. It then exports the same state and operator to dense and sparse forms and compares algebraic results, independent small-system reference matrices, and numerical backends within explicit tolerances.

For physical spins `J_i=S_i/2` and real coupling `g`, the Hamiltonian is `H=-g Σ⟨i,j⟩ J_i·J_j=-(g/4) Σ⟨i,j⟩ S_i·S_j`. On an open chain of length L≥1, the Neel expectation is `g(L−1)/4`; it is zero for zero sites. Positive g is ferromagnetic.

Also verify all local rules, a custom finite-dimensional algebra, non-orthonormal bases, graded CAR and signs, basis round trips, spaces, bindings, errors, sharing, deep trees, and preservation of small coefficients. Record executed checks in the progress of active plans rather than putting command results or migration history in this document.
