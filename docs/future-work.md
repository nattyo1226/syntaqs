# Future work

Updated: 2026-11-17

This document collects future features, required design reviews, and candidates not yet adopted. Current specifications belong in [architecture](architecture.md) and [design](README.md#2-feature-details). Follow the [documentation guide](README.md#1-document-responsibilities) when starting an implementation plan or completing work.

## 1. Required design review

Revisit ownership, sharing, and associations among State, Operator, Structure, and Space, and the boundary between Geometry and spaces. Consider whether states and operators can be understood as special cases of general operators, and document the mathematical objects, API correspondence, shared responsibilities, and distinctions with reasons. See [design review topics](future-work/state-and-operator.md) for questions and completion criteria.

## 2. Unimplemented features and proposals

These features or candidates have not started; public APIs, schedules, and priorities are undecided. Use a plan in `plans/` when implementation needs multiple stages or prior investigation; straightforward changes need no plan. Reflect implemented specifications in `design/`.

| Area | Future scope |
| --- | --- |
| [Partial trace](future-work/partial-trace.md) | Ambient/contracted spaces, delayed nodes, remaining spaces, ordinary-action contraction for graded inputs, evaluation, and verification. |
| [Map representation coefficients](future-work/representation-matrix.md) | General linear, conjugate-linear, and multiple-input maps; different input/output bases; retained, reusable coefficients. |
| [API usability](future-work/api-usability.md) | Construction helpers, factor lookup, inference, diagnostics, onboarding, and custom-algebra guidance. |
| [Storage and restoration](future-work/identity-and-storage.md) | Restoration preserving indices, independent copies with consistent fresh indices, and storage formats. |
| [External connections](future-work/external-connections.md) | Automatic differentiation, ML modules, tensor-network conventions, and parameter dependencies. |

### 2.1. Local algebra and basis extensions

Consider heterogeneous local algebras within one system, type erasure and storage for them, runtime representation parameters, and definition matching between implementation types. Per-factor bases, subspace bases, projections, constrained subspaces, conserved quantities, and particle-number sectors remain future work.

Spin-1/2, spin-1, single-mode fermions, and their current models are already provided. Further candidates include general spin-S, qudits, bosons and truncation; spin-one MatrixUnitBasis, spherical-tensor phase conventions, and exact Product/Act without Float; and generic spinful-fermion helpers, arbitrary lattices, and variable couplings.

### 2.2. Expression and space operations

Consider bra types and constructors, general U→V maps, internal labels, subsystem extraction by factor selection, regrouping/permutation, and effective mixtures of ordinary and graded products.

Other candidates are factoring sums of products into products of sums, selecting individual simplification rules, general expression transformations, advanced simplification strategies, decomposition/reconstruction of arbitrary expression types, and connections to custom expression trees. Distribution, local multiplication, and coefficient handling needed by ordinary evaluate and basis expansion already exist and are separate from these additional operations.

Automatic source retention, structural expression comparison, and general action-equivalence checking are unimplemented. Sharing, caches, parallel evaluation, and cross-thread sharing are [further design topics](future-work/evaluation-improvements.md#4-expression-management-and-reuse).

[Approximate component pruning](future-work/evaluation-improvements.md#5-explicit-component-pruning) would require explicit basis, tolerance, and application-point choices. Current evaluation, coordinates, and export must not silently prune components.

### 2.3. Geometry, numerical backends, and additional quantum features

- Standard geometries beyond one-dimensional chains: arbitrary graphs, coordinates, distances, plaquettes, and hyperedges. Square lattices currently exist only in examples.
- CSR/CSC selection during sparse matrix export, other index integer types, and additional backends.
- Density operators, mixed and reduced states, thermal states, eigensolvers and ground-state searches, time evolution, measurement, and probability sampling.
- Parameterized operators, symbolic coefficients, and large symbolic computations.
- MPS/MPO conversion, MPO generation/minimization, and tensor-network truncation, gauges, and canonical forms.
- GPU, distributed, and large-scale numerical backends.

Do not create crates solely for prospective features before their independent responsibilities, dependencies, and execution boundaries are concrete. Supertrace differs from ordinary trace and is not currently provided.

## 3. Outstanding verification and performance

Dedicated no_std target builds and performance across workloads and targets
remain unverified. Targeted host measurements cover local grouping, repeated
state actions, exact-zero contractions, and sparse operator coordinates; they
do not establish general speed or memory guarantees. See
[verification](design/verification.md) for current checks and limits.

Deferred investigations are described in [evaluation improvements](future-work/evaluation-improvements.md):

- [Lie residual accuracy](future-work/evaluation-improvements.md#1-lie-residual-accuracy): reduce cancellation while preserving small residuals and exact-zero rules.
- [Inner-product acceleration](future-work/evaluation-improvements.md#2-inner-product-acceleration): investigate basis-aware contraction, lazy group preparation, and partial reuse.
- [Natural-state collection overhead](future-work/evaluation-improvements.md#3-natural-state-collection-overhead): reduce query preparation and allocation without persistent collection assumptions.
- [Expression management and reuse](future-work/evaluation-improvements.md#4-expression-management-and-reuse): assess caches, retained parity structure, sharing, and parallelism.
