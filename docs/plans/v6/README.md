# Updates with version 0.6.0

## Core expression performance

Space binding reuses factor positions stored on each Hilbert space. State
actions skip unaffected tensor children, and operator evaluation keeps
independent blocks factored to reduce work in contractions and basis queries.
The reference workloads and measured trade-offs are recorded in the
[performance baseline](../../decisions/core-evaluation-performance.md#follow-up-performance-baseline).

Tensor construction and composite evaluation require ordered factor IDs
(`I: Ord`). `HilbertSpace::flatten()` and `tensor_flatten()` promote one level
of grouping. Custom bases implement `Basis::expand_indices()` and
`Basis::element()`. The convenience method `Basis::expand()` returns a
`Result`.

## Lie analysis performance

Closure reuses evaluated Hilbert–Schmidt operands and skips disjoint commuting
term pairs in eligible brackets. Center extraction reuses the Gram matrix.
Exact-zero constraints skip residual matrix multiplication, and decomposition
builds symmetric adjoint constraints directly to reduce peak memory.
[Reference measurements](../../decisions/lie-analysis.md#evaluation-performance).

Scale the input basis coefficients before checking closure or decomposing a
family to limit roundoff from large intermediate products.
[Basis scaling guidance](../../design/lie.md#3-projection-and-numerical-scales).
