# Future work

Updated: 2026-11-20

The topics below are deferred investigations and unadopted proposals, not a
release commitment. Use [architecture](architecture.md) and the
[feature designs](README.md#2-feature-details) to determine what is available
now. The [documentation guide](README.md#1-document-responsibilities) describes
how an adopted task moves into planning and current specifications.

## 1. Required design review

Review ownership and sharing among states, operators, local algebra, and spaces,
including the boundary between geometry and space assignment. A central question
is whether general operators provide a useful common account of states and
operators. The [review topics](future-work/state-and-operator.md) define the
questions and completion criteria without presuming type unification.

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

Current local algebras cover spin-1/2, spin-1, and single-mode fermions and their
models. Extension candidates fall into three groups:

- New local systems: general spin-S, qudits, and bosons with truncation.
- Spin-one representations: MatrixUnitBasis, spherical-tensor phase conventions,
  and exact Product/Act without Float.
- Model construction: generic spinful-fermion helpers, arbitrary lattices, and
  variable couplings.

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

Dedicated `no_std` target builds and performance across platforms remain
unverified. Host measurements cover space construction, state action,
contractions, and basis queries on spin and fermion chains. See
[verification](design/verification.md) for current checks and limits.

Deferred investigations are described in [evaluation improvements](future-work/evaluation-improvements.md):

- [Lie residual accuracy](future-work/evaluation-improvements.md#1-lie-residual-accuracy): reduce cancellation while preserving small residuals and exact-zero rules.
- [Inner-product acceleration](future-work/evaluation-improvements.md#2-inner-product-acceleration) covers basis-aware contraction, orthogonality-based filtering, and reuse of partial contractions.
- [Natural-state collection overhead](future-work/evaluation-improvements.md#3-natural-state-collection-overhead): reduce query preparation and allocation without persistent collection assumptions.
- [Expression management and reuse](future-work/evaluation-improvements.md#4-expression-management-and-reuse): assess caches, retained parity structure, sharing, and parallelism.
- [Implicit identity factors](future-work/evaluation-improvements.md#6-implicit-identity-factors-in-operator-evaluation): elide redundant local identity placements during collection, reconstruction, and contraction.
- [Small operator basis queries](future-work/evaluation-improvements.md#7-setup-cost-in-small-operator-basis-queries) incur term preparation and support tracking overhead.
- [Coordinates with dense support](future-work/evaluation-improvements.md#8-storage-for-coordinates-with-dense-support) can spend substantial time and storage on full-length keys.
- [Extreme Hilbert–Schmidt coefficients](future-work/evaluation-improvements.md#9-extreme-coefficients-in-composite-hilbertschmidt-products) can overflow during term collection before contraction.
