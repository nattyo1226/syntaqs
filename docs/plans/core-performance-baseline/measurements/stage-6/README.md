# Stage 6 measurements

Operator contractions reuse the evaluated operator blocks. Normalized
Hilbert–Schmidt inner products reduce the two operands directly, then use
conjugate symmetry in matching sums. Local parity discards zero
contributions in graded contractions. [Implementation plan](../../README.md#6-operator-contractions)

## Time per query

Thread CPU medians are in microseconds per call on prebuilt operands.

| Workload | Before (µs) | Direct HS (µs) | Conjugate symmetry (µs) | Local parity (µs) |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg HS self product, 4 sites | 33.513 | 7.276 | 6.958 | 7.017 |
| Heisenberg HS self product, 16 sites | 1,120.690 | 43.803 | 39.125 | 37.778 |
| Heisenberg HS self product, 64 sites | 57,929.167 | 354.241 | 262.929 | 271.171 |
| Hubbard trace, 4 sites | 23.499 | 26.002 | 23.292 | 10.900 |
| Hubbard trace, 16 sites | 330.806 | 370.562 | 324.689 | 56.311 |
| Hubbard trace, 64 sites | 5,980.271 | 6,177.146 | 5,914.625 | 255.095 |

The first two changes target HS and retain the original trace path.
Hubbard timing varies between those checkpoints with the same count
of lowered terms. The local parity checkpoint keeps the same visited
Heisenberg pairs as the symmetry checkpoint. Its 64-site median is
about 3% higher than the preceding run.

## Contraction work

| 64-site workload and metric | Before | Direct HS | Conjugate symmetry | Local parity |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg HS pairs visited | 35,721 | 35,721 | 17,955 | 17,955 |
| Heisenberg HS pairs rejected by zero | 35,532 | 35,532 | 17,766 | 17,766 |
| Hubbard trace terms visited | 316 | 316 | 316 | 316 |
| Hubbard trace terms rejected by zero | 252 | 252 | 252 | 0 |
| Hubbard trace terms rejected by parity | 0 | 0 | 0 | 252 |
| Hubbard nonempty parity strings | 502 | 502 | 502 | 0 |
| Hubbard parity factor insertions | 32,004 | 32,004 | 32,004 | 0 |

Direct HS computes normalized local overlaps from the two operands and
reuses them within each query. The same set of term pairs is visited as
at stage entry, with the first exact-zero factor ending a pair. Conjugate
symmetry visits the diagonal and upper triangle of the 189-term
Heisenberg sum. Its visited off-diagonals all vanish.

The 64 sites of the Hubbard chain contain 128 graded modes. Each of the
252 hopping terms has an odd local factor. Local parity identifies these
terms before primitive trace evaluation. The 64 interaction terms have
even local parity and contribute to the result.

| 64-site workload | Allocation requests before | Direct HS | Conjugate symmetry | Local parity |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg HS self product | 3,306,399 | 4,309 | 4,309 | 4,309 |
| Hubbard trace | 233,661 | 233,661 | 233,662 | 7,158 |

## Reproduction and verification

The reference revision is `538ae88f685756ae87365e363a48b355780a8562`.
The Stage 0 host, Rust 1.99.1, release profile, and `Complex<f64>`
coefficients are used throughout. The `,hs_self` and `,trace` filters
provide 11 matched timing and allocation cases across five Heisenberg
sizes and six Hubbard sizes. The raw timing files include seven samples
and medians for thread CPU and elapsed time. The diagnostic queries check
the analytic Heisenberg HS value `3(n-1)/16` and the Hubbard trace per
dimension `n`. [Probe commands](../../README.md#reproduction)

Work counts use separately instrumented copies of the checkpoints and
reset before each call on the prebuilt operator. The baseline counts a
distributed `lhs† rhs` term in HS and a lowered operator term in trace.
Direct HS counts each visit to its term-pair kernel. A numerical rejection
counts an exact-zero contraction, while a parity rejection counts the
local selection rule. One parity string is counted for each odd placement
with a nonempty prefix. The factor count includes every parity operator
inserted into those strings.

- Stage entry. [Timings](before/timings.csv) · [Allocations](before/allocations.csv) · [Diagnostics](before/diagnostics.csv) · [Work counts](before/work.csv)
- Direct HS. [Timings](direct/timings.csv) · [Allocations](direct/allocations.csv) · [Diagnostics](direct/diagnostics.csv) · [Work counts](direct/work.csv)
- Conjugate symmetry. [Timings](symmetric/timings.csv) · [Allocations](symmetric/allocations.csv) · [Diagnostics](symmetric/diagnostics.csv) · [Work counts](symmetric/work.csv)
- Local parity. [Timings](local-parity/timings.csv) · [Allocations](local-parity/allocations.csv) · [Diagnostics](local-parity/diagnostics.csv) · [Work counts](local-parity/work.csv)

Dense references cover interleaved mixed-parity blocks, unequal outer
scales, and the exchange sign of products with repeated odd placements.
Independent homogeneous odd blocks remain factored when reordered.
Normalization tests cover extreme representable scales, unit spaces,
dimensions beyond integer range, and early stopping on zero.
Workspace tests and doctests with all features, all-feature Clippy with
warnings denied, the all-targets check without default features, and
formatting passed on Rust 1.99.1.
