# Site-count bottlenecks at fixed Lie dimension

Measured on 2026-11-18 at revision `e1e2ebd99d34a1dec843e7977a8dca19169a934b`.

- **Growing support makes closure expensive.** At the same Lie dimension
  `n=5` and the same total of 16 sites, closure takes 1.83 s for the active
  Heisenberg chain and 11.72 ms for the two-site chain with spectators. The
  number of bracket-to-basis contractions is constant. The growing number of
  terms and the time spent searching for tensor merges dominate the increase.
- **Spectators also cost time at fixed support.** Holding the two-site
  Heisenberg operators fixed, closure grows from 3.28 ms on two sites to
  43.55 ms on 64 sites. The number of enumerated terms stays constant. Term
  collection, reconstruction, and contraction visit the ambient factors.
- **Graded support adds a dependence on site order.** For the same two
  active fermion modes on 64 sites, moving them from the front to the tail
  raises closure from 55.02 ms to 64.83 ms and decomposition from 58.49 ms to
  72.13 ms. The impact appears in parity lowering and local trace reduction.

## Workloads and controls

`L` is the number of tensor factors in the full bound space. Each local space
has dimension two, so the physical Hilbert-space dimension is `2^L`. `n` is the
dimension of the real span used for Lie analysis. The workload sets were as
follows. [Benchmark driver](benchmark.rs).

| Case | Operator family | Controlled variation |
| --- | --- | --- |
| `heisenberg-L` | Three total Pauli components, the open isotropic Heisenberg Hamiltonian, and identity. The Lie algebra is `su(2) ⊕ u(1) ⊕ u(1)` (`n=5`). | Extend the active chain through `L=2,4,8,12,16`. Each total Pauli component has `L` terms and the Hamiltonian has `3(L−1)` bond terms. |
| `heisenberg-2-in-L` | The exact same two-site spin operators and Hamiltonian embedded in a larger ordinary space (`n=5`). | Keep the first two sites active and add spectators through `L=64`. |
| `fermion-front-L`, `fermion-tail-L` | Six Hermitian bilinears of the four Majorana operators on two adjacent modes. The Lie algebra is `so(4)` (`n=6`). | Extend the graded ambient space through `L=2,4,8,16,32,64`. Compare placing the active pair at the front and at the tail. |

Public API times use the uninstrumented executable and are medians in
milliseconds across three fresh processes per case. `Closure` starts with a
retained Gram factorization. `Center`, `Commutant`, and `Decompose` start with
the retained adjoint representation. The internal timings below come from a
paired build with the existing hotspot instrumentation. [Raw runs](site-scaling.csv),
[timing conventions](README.md#measurement-and-reproduction).

## Ordinary spins

| Sites | Active closure | Spectator closure | Active center | Spectator center | Active decomposition | Spectator decomposition |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 3.28 | 3.28 | 0.45 | 0.45 | 0.74 | 0.74 |
| 4 | 17.14 | 4.46 | 1.86 | 0.63 | 2.80 | 1.04 |
| 8 | 134.15 | 6.86 | 10.11 | 1.00 | 14.19 | 1.59 |
| 12 | 588.97 | 9.28 | 29.58 | 1.47 | 40.48 | 2.31 |
| 16 | 1830.79 | 11.72 | 65.23 | 1.88 | 87.97 | 2.94 |
| 32 | — | 22.06 | — | 3.70 | — | 5.85 |
| 64 | — | 43.55 | — | 8.00 | — | 12.64 |

Each case performs 25 bracket projections and 125 right-hand-side HS calls
in closure. The right-hand-side term counter sums intrinsic tensor terms
visited while collecting both operands. On the active chain, the Hamiltonian
and total-spin sums grow with `L`. The bracket expressions contain products
of those sums, including for brackets that eventually cancel. At 16 active
sites, `merge_tensor` alone accounts for 472 ms within the 736 ms RHS phase.
The residual norm is another 306 ms. `merge_tensor` scans retained tensor
terms for compatible factors, so both the number of incoming terms and the
number of retained candidates matter. [Projection loop](../../../crates/syntaqs-lie/src/family/projection.rs#L325),
[tensor merging](../../../crates/syntaqs-core/src/evaluate/collect.rs#L76).

| Case | Intrinsic terms in RHS collections | RHS (ms) | RHS tensor merging (ms) | Residual norm (ms) |
| --- | ---: | ---: | ---: | ---: |
| `heisenberg-2` | 1,250 | 1.44 | 0.14 | 0.60 |
| `heisenberg-8` | 22,310 | 55.71 | 18.07 | 23.40 |
| `heisenberg-12` | 50,750 | 237.83 | 120.91 | 99.33 |
| `heisenberg-16` | 90,710 | 735.88 | 471.63 | 305.85 |
| `heisenberg-2-in-16` | 1,250 | 5.51 | 0.39 | 2.24 |
| `heisenberg-2-in-64` | 1,250 | 21.78 | 1.26 | 8.08 |

For the spectator series, all sizes visit 1,250 intrinsic terms in RHS
collection and 600 ordinary terms across closure traces. The collection loop
visits every ambient factor for each intrinsic term and materializes a local
identity for absent placements. Rebuilding the collected expression places
the full factor list, and the trace visits factors in ambient order. At 64
sites, rebuilding the RHS operands takes 3.82 ms and contracting them takes
8.93 ms. The center calculation also scales with the cost of re-evaluating
the source Gram matrix while forming orthonormal coordinates. [Collection and rebuilding](../../../crates/syntaqs-core/src/evaluate/collect.rs#L153),
[trace traversal](../../../crates/syntaqs-core/src/evaluate/trace.rs#L121),
[center coordinates](../../../crates/syntaqs-lie/src/invariant.rs#L219).

## Graded fermions

| Sites | Front closure | Tail closure | Front decomposition | Tail decomposition |
| ---: | ---: | ---: | ---: | ---: |
| 2 | 4.04 | 4.04 | 1.79 | 1.79 |
| 4 | 5.12 | 5.48 | 2.75 | 3.20 |
| 8 | 7.52 | 8.44 | 5.01 | 6.18 |
| 16 | 12.68 | 14.92 | 10.24 | 13.21 |
| 32 | 24.14 | 28.52 | 23.26 | 29.51 |
| 64 | 55.02 | 64.83 | 58.49 | 72.13 |

The component orthonormality checks enumerate exactly 288 ordinary tensor
terms at every site count and either position. With the active pair at the
front, grading-aware enumeration inside those checks grows from 0.36 ms to
24.76 ms by 64 sites. The homogeneous iterator visits local placements and
splits them into parity parts. Collection supplies explicit identity
placements for the spectators, so a single yielded tensor term entails more
work as `L` increases. [Homogeneous term enumeration](../../../crates/syntaqs-core/src/expression/traversal.rs#L254),
[raw runs](site-scaling.csv).

At 64 sites, moving the active pair to the tail increases the number of
parity-factor insertions during component orthonormality checks from 288 to
36,000. The lowering step visits every preceding factor for each odd
placement. For even bilinears on the adjacent tail modes, the two parity
prefixes on earlier spectator sites multiply to identity during ordinary
local evaluation. The profile counts the intermediate insertions. The
tail's component check takes 56.78 ms compared with 46.94 ms at the front.
Lowering accounts for 2.58 ms versus 1.13 ms, and local trace reduction for
9.39 ms versus 1.52 ms. [Parity lowering](../../../crates/syntaqs-core/src/expression/traversal.rs#L377),
[local reduction](../../../crates/syntaqs-core/src/evaluate/trace.rs#L128).

## Adjoint constraints as a control

The full commutant has a `125 × 25` constraint matrix for all Heisenberg
cases and a `216 × 36` matrix for all graded cases. Its medians range from
0.102 to 0.127 ms for Heisenberg and from 0.240 to 0.254 ms for Majorana
bilinears. The full 16-site Heisenberg analysis still peaks at about 7.6 MiB
of process RSS. These fixed-dimension runs concentrate the site-dependent
growth in expression work. [Constraint dimensions](../../../crates/syntaqs-lie/src/invariant.rs#L195),
[raw runs](site-scaling.csv).

## Reproduction

Build the baseline and instrumented release binaries using the revision at the
top of this report and the current benchmark driver from this directory.
Follow the existing hotspot setup through the second build. The runner uses
one OpenBLAS thread for both binaries. It writes baseline stage times and
process RSS alongside selected internal timers and counts. [Build setup](hotspots.md#verification-and-reproduction).

```sh
python3 "$plan/measure.py" "$bench" "$work/results" --site-scaling "$work/baseline"
```

All 22 cases passed the closure, center, commutant, and simple-ideal dimension
assertions in both builds. The extended driver also passed the original 15
cases. The instrumented closure medians differed from baseline by up to 12%
in this run. Absolute performance is specific to the inputs and host.
