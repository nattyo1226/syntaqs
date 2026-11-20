# Stage 5 measurements

Composite collection and basis queries share an evaluator that combines
overlapping operator blocks and keeps disjoint product blocks independent.
Basis conversion runs on the surviving natural terms. The 16-site XX echo
reduces to one basis input term after 32 block-local term products.
[Implementation plan](../../README.md#5-operator-evaluation-and-basis-conversion-order)

## Time per query

Thread CPU medians include evaluation and basis conversion for prebuilt
operators. The raw data also retain elapsed times.

| Workload | Before (µs) | After (µs) |
| --- | ---: | ---: |
| XX forward coordinates, 4 sites | 2.176 | 3.350 |
| XX forward coordinates, 8 sites | 11.564 | 12.183 |
| XX forward coordinates, 16 sites | 304.482 | 268.112 |
| XX echo coordinates, 16 sites | 178,239.291 | 17.752 |
| Heisenberg H² coordinates, 64 sites | 36,125.792 | 22,189.458 |
| Heisenberg H² expanded syntax, 64 sites | 84,398.375 | 66,575.209 |

The four-site forward layer pays for the query-local primitive and block
tables before emitting four coordinates. Its coordinate query took
1.17 µs more and issued 49 extra allocation requests. The delay fell to
0.62 µs at eight sites. At 16 sites the forward-layer query improved by
about 12% while still emitting 256 coordinates.

## Intermediate products and allocations

| Workload | Basis input terms before | Block-local term products after | Basis input terms after | Output coordinates |
| --- | ---: | ---: | ---: | ---: |
| XX forward layer, 16 sites | 256 | 0 | 256 | 256 |
| XX echo, 16 sites | 65,536 | 32 | 1 | 1 |
| Heisenberg H², 16 sites | 2,025 | 2,025 | 907 | 907 |
| Heisenberg H², 64 sites | 35,721 | 35,721 | 17,395 | 17,395 |

The baseline distributes the product before sending complete terms to
basis conversion. The block evaluator multiplies each two-site gate with
its adjoint independently. The forward layer still needs all 256 complete
terms for its coordinate output. For H², the two Hamiltonian sums have the
same factor support. All term pairs enter that block's multiplication, and
collection eliminates cancellations and merges duplicate natural terms
before conversion.

| Workload | Allocation requests before | Allocation requests after |
| --- | ---: | ---: |
| XX forward layer, 4 sites | 81 | 130 |
| XX forward layer, 16 sites | 10,573 | 9,856 |
| XX echo, 16 sites | 7,048,604 | 712 |
| Heisenberg H² coordinates, 64 sites | 797,331 | 461,694 |
| Heisenberg H² expanded syntax, 64 sites | 2,331,319 | 1,850,206 |

The 64-site H² coordinate query went from 140,635 calls to the local basis
rule to 68,827. The remaining calls include repeated conversion of the same
primitive on different terms. The expanded-syntax workload also constructs
placements for all 17,395 output coordinates. The evaluator extracts shared
leading and trailing blocks before forming the polynomial for a sum. A sum
whose addends have different factorizations can still require a large
intermediate polynomial.

## Reproduction and verification

The reference revision is `bbd01f532cf52a7679cbc2ddce6dfe5a46376474`. Both
checkpoints use the Stage 0 host, Rust 1.99.1, default release profile,
`Complex<f64>` coefficients, and the probe filters `,xx_gate_` and
`,heisenberg,`. The Heisenberg data shown use product order 2. Each checkpoint
contains 30 matched timing and allocation cases and 39 diagnostic rows.
Timings were rerun in succession with prebuilt release binaries for both
checkpoints. The files retain median thread CPU time, elapsed time, and
seven samples per case. [Probe commands](../../README.md#reproduction)

The separate work runs used temporary copies instrumented at basis-input
term entry and, in the second checkpoint, at overlapping-block
multiplication. Each counter was reset immediately before the prebuilt
operator's coordinate query. The 13 coordinate-producing inputs had
identical keys and coefficients across all 22,945 returned coordinates.

- Stage entry. [Timings](before/timings.csv) · [Allocations](before/allocations.csv) · [Diagnostics](before/diagnostics.csv) · [Work counts](before/work.csv)
- Shared block evaluation. [Timings](evaluated/timings.csv) · [Allocations](evaluated/allocations.csv) · [Diagnostics](evaluated/diagnostics.csv) · [Work counts](evaluated/work.csv)

Independent dense references check partially overlapping supports, adjoint
order, and graded exchanges with uniform and term-dependent signs in single
and double precision. Basis tests check zero scaling, natural cancellation
after multiplication, validation of surviving terms, and invalid original
placements.
The existing Lie-family projection test exercises products of 32 independent
local sums and verifies that collection retains their factorization through
Hermiticity and residual checks. Workspace tests and doctests with all
features, all-feature Clippy with warnings denied, the all-targets check
without default features, and formatting passed on Rust 1.99.1.
