# Expression-based Lie analysis and application-owned native linkage

Status: Accepted

Keep Lie analysis in a separate crate that reuses core operator expressions and
normalized HS contractions. Retain results with their source family, validate
complete center/simple-ideal splitting, and let applications select native
BLAS/LAPACK providers. The decisions below explain these boundaries and their
consequences.

## Context

The implementation reference was
[LieAlgebra.jl at revision 575029c12fcad44b3b41206ee564437a7ec5fa45](https://github.com/nattyo1226/LieAlgebra.jl/tree/575029c12fcad44b3b41206ee564437a7ec5fa45/src).
It provided the starting feature set and commutant/eigenspace decomposition
approach. syntaqs already supplies shared operator expressions, ordinary/graded
evaluation, and normalized Hilbert–Schmidt contractions without physical action
matrices. Its existing `Basis` describes complete local primitive bases, while
Lie analysis needs selected families of operator expressions.

## Decisions

Keep Lie analysis in `syntaqs-lie`, reusing core expressions and contractions.
Use `OperatorFamily` for ordered candidates and retain coefficient results with
their source family and context. This preserves core's dependency direction and
no_std boundary without a second evaluator or a production dependency on concrete
algebras. General map-representation abstractions remain a separate
[proposal](../future-work/representation-matrix.md).

Fix the domain to the real span of Hermitian operators, preserve f32/f64, and use
the normalized HS metric and residual norm. Use the ordinary `-i(AB-BA)` bracket
on both ordinary and graded spaces and the classical Killing form
`Tr(ad_i ad_j)`, without normalization or conjugation. These are intentional
project contracts rather than drop-in compatibility with the reference's
matrix-based API or normalized adjoint Gram convention.

Keep context construction separate from mathematical validation. Require the
necessary properties before computing retained Lie representations; reject
singular coordinate solves instead of discarding inputs or using a pseudoinverse.
Separate the full center before sampling the symmetric commutant on its
orthogonal complement. Validate complete simple-ideal splitting and use finite,
seeded retries so accidental spectral collisions cannot silently become a
successful incomplete decomposition.

Leave native BLAS/LAPACK selection to the application through `blas-src`, with
no provider-selection features on `syntaqs-lie` or its `ndarray-linalg` dependency.
Use the unpublished `support/blas-provider` package for shared development,
examples, and tests. Selecting a provider in the production library would couple
downstream applications to that choice and complicate workspace feature
unification. The separation follows the dependency-layout pattern reviewed in
[tensory at revision abac6a1b491738e509139a21a74a8833134c4a4f](https://github.com/todo-group/tensory/tree/abac6a1b491738e509139a21a74a8833134c4a4f),
without adopting its repository-specific dependency patches.

## Evaluation performance

Reuse prepared HS operands within closure and index term support for eligible
placed commutators. The index reduced pair visits on long Heisenberg chains,
and the measurements included an eager collection control to isolate that gain.
Construct symmetric adjoint constraints directly to avoid the full-commutant
matrix as an intermediate while preserving the SVD problem. Dense SVD remains
the main cost for the larger commutant and decomposition cases.

On 2026-11-21, the `f64` comparison against the v6 merge `c26ea49` used
an Apple M4 and single-threaded OpenBLAS 0.3.33. The median closure speedup
across 15 primary workloads was 4.42×. The 16-site Heisenberg closure fell
from 64.00 ms to 2.94 ms, and `fermion-4` decomposition peak RSS fell from
279.77 MiB to 147.89 MiB. The benchmark sources, reproduction instructions,
and raw samples are retained in Git revision `76e6224c599bafc186fc5d51035b5882f629e993`.

## Consequences and references

The [Lie design](../design/lie.md) owns current APIs, numerical defaults, and
validation semantics; [requirements](../requirements.md#36-lie-algebra-analysis)
owns scope and exclusions. Native setup is documented in the
[crate README](../../crates/syntaqs-lie/README.md#native-linkage), and
[verification](../design/verification.md) owns checks and their limits.

The pinned [Julia tests](https://github.com/nattyo1226/LieAlgebra.jl/tree/575029c12fcad44b3b41206ee564437a7ec5fa45/test)
were starting examples, not a claim of upstream suite equivalence or freshness.
Dimension-scaled rank thresholds have background in the
[NumPy rank](https://numpy.org/doc/stable/reference/generated/numpy.linalg.matrix_rank.html)
and [SciPy nullspace](https://docs.scipy.org/doc/scipy/reference/generated/scipy.linalg.null_space.html)
documentation; syntaqs' precision-dependent floors are project policy, not those
libraries' defaults or general accuracy guarantees.
