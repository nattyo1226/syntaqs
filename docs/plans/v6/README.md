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
