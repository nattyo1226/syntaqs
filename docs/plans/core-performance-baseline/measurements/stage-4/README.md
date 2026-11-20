# Stage 4 measurements

State inner products align tensor blocks on demand and use conjugate
symmetry for matching normalized sums. The W64 norm took 33.91 ms before
the changes, 9.53 ms with on-demand alignment, and 3.67 ms with shared
normalization and conjugate symmetry.
[Implementation plan](../../README.md#4-state-contractions)

## Norm evaluation time

Each cell reports the median thread CPU time in microseconds per complete
`norm_squared()` call, including normalization. Input states are prebuilt.
The raw timing files also report elapsed time and all seven samples.

| State | Before (µs) | On-demand blocks (µs) | Conjugate symmetry (µs) |
| --- | ---: | ---: | ---: |
| W8 | 109.182 | 60.193 | 27.372 |
| W16 | 711.336 | 302.055 | 124.231 |
| W32 | 4,373.740 | 1,562.734 | 617.050 |
| W64 | 33,913.917 | 9,529.979 | 3,665.115 |
| Factored plus, 18 sites | 20.788 | 15.957 | 8.723 |
| GHZ, 18 sites | 35.417 | 25.755 | 13.219 |

The on-demand checkpoint borrows factor ranges from evaluated tensors and
finds each shared boundary only when its contraction is needed. The symmetry
checkpoint also shares normalization and visits only the diagonal and upper
triangle for matching sums. The timing for that checkpoint includes the
savings from shared normalization. The work counts below identify how many
overlaps each contraction change removes.

The W64 standalone `evaluate()` and `coordinates(&SzBasis)` controls also
ran about 8–9% slower during the on-demand checkpoint, which changed only
the contraction path. In the final run the two controls were within 2% of
the initial values.

## Contraction work and allocation requests

| State and metric | Before | On-demand blocks | Conjugate symmetry |
| --- | ---: | ---: | ---: |
| W64 prepared blocks | 353,665 | 91,521 | 47,777 |
| W64 tensor term pairs | 4,096 | 4,096 | 2,080 |
| W64 local primitive overlaps | 91,456 | 91,456 | 47,776 |
| W64 allocation requests | 1,301,314 | 277,662 | 126,523 |
| W64 requested bytes | 77,750,120 | 22,208,696 | 5,969,856 |
| Factored plus (18 sites), prepared blocks | 36 | 18 | 18 |
| Factored plus (18 sites), local primitive overlaps | 72 | 72 | 54 |
| Factored plus (18 sites), allocation requests | 723 | 576 | 300 |

The W terms still run primitive overlap rules to discover zeros. On-demand
alignment leaves the number of primitive overlaps unchanged and discards
the unvisited suffix when an overlap vanishes. The plus state has one
nonzero block result per factor. On-demand alignment saves the repeated
preparation of its already aligned singleton blocks. Symmetry changes the
four overlaps in each matching two-term local sum to three.

Prepared blocks count groups constructed by the baseline, including
singletons reconstructed on entry, and blocks emitted by the cursor in
the later checkpoints. This count excludes operand normalization and
the structural pass that records factor coverage. Tensor pairs count
contraction entries with tensor terms on both sides, whether a term is
still wrapped in its tensor node or represented by its aligned factors.
Local overlaps count invocations of the primitive inner-product rule.

## Reproduction and verification

The initial run on November 20, 2026 used revision
`f4dfe450191df6c8b343e90147ac49d5dc269583`. The host, Rust 1.99.1, release
profile, and `Complex<f64>` coefficients match Stage 0. Each checkpoint
uses the probe filters `,w,`, `,plus,`, and `,ghz,` for 47 matched timing
and allocation cases and 12 diagnostic rows. Norm assertions compare with
1 within `1e-10`. Coordinate counts and coefficient assertions pass at each
checkpoint. Work counts come from separate copies of the three checkpoints
with counters at group creation, tensor-term contraction entry, and
primitive inner-product invocation. Each counter run resets immediately
before a call on the prebuilt state. [Probe commands](../../README.md#reproduction)

- Stage entry. [Timings](before/timings.csv) · [Allocations](before/allocations.csv) · [Diagnostics](before/diagnostics.csv) · [Work counts](before/work.csv)
- On-demand blocks. [Timings](lazy-blocks/timings.csv) · [Allocations](lazy-blocks/allocations.csv) · [Diagnostics](lazy-blocks/diagnostics.csv) · [Work counts](lazy-blocks/work.csv)
- Conjugate symmetry. [Timings](conjugate-symmetry/timings.csv) · [Allocations](conjugate-symmetry/allocations.csv) · [Diagnostics](conjugate-symmetry/diagnostics.csv) · [Work counts](conjugate-symmetry/work.csv)

Tests cover overlapping tensor boundaries, scalar factors, exact-zero
and underflow stopping, nested matching sums with distinct complex outer
scales, and full pair evaluation for nonreflexive values. The matching
and distinct nested sums are checked against independent amplitudes.
Workspace tests and doctests with all features, all-feature Clippy with
warnings denied, the all-targets check without default features, and
formatting passed on Rust 1.99.1.
