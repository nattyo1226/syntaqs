# Core expression performance baseline

This baseline measures the public construction, evaluation, contraction, and
basis expansion APIs on spin-1/2, spin-1, and graded fermionic chains. It
records where cost follows the ambient space, the number of intermediate
contributions, or the requested output size.

The largest avoidable workload measured was operator expansion of an XX gate
layer followed by its inverse. At 16 sites, the query considered 65,536
syntactic products and took 199 ms to return one identity coordinate. A
single spin flip also exposed costs unrelated to output size. Evaluating it on
a 1,024-site product state took 1.30 ms and approximately 527,000 index
comparisons for one output configuration. The same Hamiltonian can also cost
more on different states. At 64 sites, applying Hubbard to the vacuum took
11.9 ms with zero output coordinates, and applying it to alternating
single occupations took 20.2 ms with 126 coordinates.

## Construction and placement

`tensor_all` starts with existing factor handles. `product_state` starts with
an existing space and creates the placed alternating Néel ket. `build_model`
calls the open-chain Heisenberg helper with coupling 1. `bind` rebinds the
result to an equivalent space built from the same factors. `act` builds a
delayed action with the already bound Hamiltonian and a state sharing its
space handle. Times are median thread CPU microseconds per call.

| Sites | `tensor_all` (µs) | `product_state` (µs) | `build_model` (µs) | `bind` (µs) | `act` (µs) | Index comparisons in `bind` or `act` |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.33 | 0.94 | 7.05 | 2.06 | 2.17 | 189 |
| 32 | 1.21 | 5.52 | 41.8 | 13.1 | 13.3 | 3,069 |
| 128 | 6.65 | 11.6 | 186 | 87.1 | 87.3 | 49,149 |
| 512 | 58.7 | 49.4 | 1,239 | 814 | 822 | 786,429 |
| 1,024 | 190 | 93.3 | 3,608 | 2,749 | 2,871 | 3,145,725 |

`tensor_all` checks each leaf against all previously seen leaves, producing
exactly `n(n−1)/2` comparisons for these inputs. `product_state` compares
ordered indices once, giving `n` comparisons. `Operator::validate` traverses
each local reference and starts a new `contains_index` scan of the ambient
space. Heisenberg places three two-site terms per bond, giving `3(n²−1)`
comparisons per `bind` or `act` in this sweep. At 1,024 sites each call
requested 50.4 MB cumulatively. Each membership scan starts a new leaf
iterator and expands its work stack for the flat ambient tensor. Sharing a
handle short-circuits space compatibility, while the reference check still
runs through `select_space`. A loop calling `act` on the same Hamiltonian
pays this validation for each delayed action it builds. [Space construction
and iteration](../../../crates/syntaqs-core/src/space.rs), [placement and
validation](../../../crates/syntaqs-core/src/expression/composite.rs).

At 64 sites, the BLBQ and Hubbard model helpers took 0.193 ms and 0.212 ms
respectively, starting with prebuilt site handles. The Hubbard inputs already
grouped the two modes at each site.

## Evaluation and scalar contractions

The models use open boundaries. Spin-1 BLBQ uses bilinear coupling `J = 1`
and biquadratic coupling `K = 0.25`, with alternating `m = +1, −1`. Hubbard
uses hopping `t = 1` and interaction `U = 4`, with either alternating up and
down single occupations or the vacuum. The table uses 64 physical sites in
each model. `evaluate` starts with a prebuilt action. A state coordinate count
refers to the action result in `SzBasis` or `OccupationBasis`.

| Model and input | Factors | State coordinates | `evaluate` (ms) | `expectation` (ms) | State `coordinates` (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Heisenberg, Néel | 64 | 64 | 0.933 | 1.81 | 2.35 |
| Heisenberg, fully aligned | 64 | 1 | 0.990 | 1.26 | 1.47 |
| BLBQ, Néel | 64 | 127 | 3.73 | 7.19 | 9.86 |
| Hubbard, alternating single occupations | 128 | 126 | 20.2 | 23.4 | 23.9 |
| Hubbard, vacuum | 128 | 0 | 11.9 | 12.5 | 12.8 |

- **Local actions on a large product state.** A last-site `X` on 16, 64, 256,
  and 1,024 sites took 7.46 µs, 26.6 µs, 135 µs, and 1.30 ms. Its evaluation
  made 168, 2,208, 33,408, and 526,848 index comparisons. This matches
  `n(n−1)/2 + 3n` in the sweep. With no action, `evaluate` uses the primitive
  tensor fast path and made zero allocations at all four sizes. A delayed
  action enters the general evaluator, which interns each factor using a
  linear search, walks the tensor for the local update, and rebuilds its
  ordered structure. [State evaluation](../../../crates/syntaqs-core/src/evaluate/composite.rs),
  [state term interning](../../../crates/syntaqs-core/src/evaluate/state_terms.rs).
- **Graded actions at distant factors.** Applying one directed hop from mode
  zero on a 128-mode product state took 77.4 µs at destination 1 and 132 µs
  at destination 127. Index comparisons rose from 9,023 to 33,152, with one
  output configuration in both cases. The evaluator scans the prefix of the
  ambient order for each non-target local state when applying an odd
  primitive. Even the vacuum path in the Hubbard model processes delayed
  terms before determining their action is zero. For operator coordinates,
  the same hop stayed at one ladder coordinate and took 2.47 to 3.11 µs.
  Intrinsic coordinates encode a graded product with its sign. [Local action
  and grading](../../../crates/syntaqs-core/src/evaluate/composite.rs),
  [intrinsic operator traversal](../../../crates/syntaqs-core/src/expression/traversal.rs).
- **Repeated action and contraction of sums.** Eight successive Heisenberg
  actions on Néel cost 2.56 ms at six sites with 20 output configurations,
  and 47.3 ms at ten sites with 222. At ten sites, one action took 54.4 µs
  and had ten configurations. Collection after each action constrains the
  intermediate states to the reachable configurations. Work still depends
  on their growth and the number of Hamiltonian terms applied at each step.
  For the explicit sum of `n` product kets representing the W state, the
  squared norm rose from 0.762 ms at 16 sites to 38.8 ms at 64 sites, or 51×.
  W64 coordinates took 2.75 ms for 64 entries. The norm considers `n²`
  term pairs and constructs tensor-group metadata before an exact-zero
  factor can stop a pair's contraction. [Action traversal](../../../crates/syntaqs-core/src/evaluate/composite.rs),
  [inner-product traversal](../../../crates/syntaqs-core/src/evaluate/inner_product.rs).
- **Operator contractions.** Heisenberg `hs_inner_product(H, H)` took
  0.268 ms at eight sites, 8.73 ms at 32, and 62.9 ms at 64. It contracts
  `H†H` term by term. Local zero traces can end the current contraction
  after operator-term traversal reaches it. The 64-site graded Hubbard
  `trace` took 7.90 ms, including grade lowering and ambient-factor traversal.
  Both queries return scalars. [Trace and Hilbert–Schmidt traversal](../../../crates/syntaqs-core/src/evaluate/trace.rs).

## Basis coordinates and expanded syntax

`coordinates` returns sparse entries with full-length digit keys. `expand`
reconstructs a sum of basis products as core expression syntax. The operator
queries below use the Cartesian basis except for the explicitly named
Hubbard bases. State entries use the `SzBasis`.

| Input | Sites | Stored coordinates | `coordinates` (ms) | `expand` (ms) |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg `H` | 64 | 189 | 0.248 | 0.952 |
| Heisenberg `H²` | 8 | 175 | 0.428 | 0.926 |
| Heisenberg `H²` | 32 | 4,099 | 9.76 | 18.1 |
| Heisenberg `H²` | 64 | 17,395 | 40.5 | 119 |
| BLBQ | 64 | 505 | 1.80 | 4.21 |
| Hubbard, ladder basis | 64 | 316 | 0.869 | — |
| Hubbard, Majorana basis | 64 | 445 | 1.28 | 4.58 |
| GHZ | 18 | 2 | 0.0491 | 0.0513 |
| Factored `|+⟩^⊗n` | 14 | 16,384 | 6.56 | 22.6 |
| Factored `|+⟩^⊗n` | 18 | 262,144 | 182 | — |

At 64 sites, Cartesian Heisenberg `H²` has 35,721 syntactic term products
before collection and 140,635 local basis-expansion calls, compared with 379
calls for `H`. Cumulative requested bytes for `H²` were 90.0 MB for
coordinates and 261 MB for expansion. The operator path resolves factor
positions and visits the ambient axes for each reached term, then aggregates
full digit keys. Expansion also builds placed nodes, including identities on
spectator factors. Basis choice affects work even at a fixed term count.
BLBQ at 64 sites had 505 entries in both Cartesian and ladder bases, with
coordinate times of 1.80 and 2.91 ms. Hubbard reaches more entries when local
ladder primitives are expanded in the Majorana basis. [Operator basis
enumeration](../../../crates/syntaqs-core/src/basis/expansion/operator.rs),
[composite basis APIs](../../../crates/syntaqs-core/src/basis/expansion/composite.rs).

The XX layer control uses `U = ∏[(I + i X_j X_{j+1})/√2]` over disjoint
bonds. It probes the multiplication of multi-factor sums with a known exact
result for `U†U`.

| Sites | Layer coordinates | `U†U` syntactic products | `U†U` coordinates | `U` coordinates (ms) | `U†U` coordinates (ms) | `evaluate(U†U|↑…↑⟩)` (ms) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 4 | 16 | 1 | 0.00293 | 0.0163 | 0.0177 |
| 8 | 16 | 256 | 1 | 0.0143 | 0.372 | 0.0949 |
| 12 | 64 | 4,096 | 1 | 0.0659 | 11.4 | 0.450 |
| 16 | 256 | 65,536 | 1 | 0.341 | 199 | 1.90 |

Each surviving echo coordinate is exactly the identity. At 16 sites the
coordinate query requested 582 MB cumulatively. The grouping pass targets
single-factor structure. The XX sum spans two sites, so the operator iterator
still distributes the sums before coordinate accumulation. State evaluation
applies products in order and collects equal resulting states between
actions. [Structural grouping](../../../crates/syntaqs-core/src/expression/group.rs),
[operator term distribution](../../../crates/syntaqs-core/src/expression/traversal.rs).

The plus state has an actual `2^n`-entry output. At 18 sites, the factored
`evaluate` and `norm_squared` took 7.75 µs and 24.7 µs, respectively. The
coordinate query requested 272 MB cumulatively and spent 182 ms enumerating
products. Every result key has length `n`. The current state expansion also
builds a temporary map for each prefix of a tensor product, copies prefix
digits, and constructs a final coordinate map. The required output size puts
a floor on work, while the temporary maps and copies are implementation
costs. [State basis traversal](../../../crates/syntaqs-core/src/basis/expansion/composite.rs).

## Refactoring targets supported by this baseline

| Cost exposed | Change to investigate | Baseline to compare |
| --- | --- | --- |
| Ambient lookup and validation recur across construction, action, and graded evaluation. | Reuse ordered leaf information within a query and across operations on the same space where validity is established. Preserve index equality, duplicate detection, explicit rebinding, and ordered graded signs. | Heisenberg `bind`/`act` at 128–1,024 sites, single `X`, and near/far hopping. |
| Distribution spends work on operator terms that later cancel. | Preserve independent multi-factor support through product evaluation where factor order and graded crossing rules permit it. | XX echo plus its noncancelling layer control, and `H²` coordinates and expansion. |
| Scalar contractions prepare tensor groups even for orthogonal summands. | Measure lazy preparation of later groups and reuse of prepared operands. Preserve exact-zero handling and general inner-product semantics. | W norms, Heisenberg Hilbert–Schmidt self products, and the factored plus norm. |
| State enumeration copies partial coordinate maps and reconstructs expanded syntax. | Reduce temporary maps and allocations while preserving complete keys and exact coefficients. | GHZ and plus states at a fixed site count, comparing `coordinates` and `expand`. |

The size ranges and basis choices were selected to expose different costs
while keeping the materialized outputs tractable. The `—` cells denote
unmeasured combinations, including expanded plus-state syntax at 18 sites.
The study covers this host, release profile, and coefficient type. The
source walkthrough and work counts identify mechanisms. Relative time in
individual routines would require CPU profiling. No dense export, memory
peak, or cross-platform scaling figures were collected. In the full timing
run, 29 elapsed-time rows and 10 CPU-time rows had a largest sample more
than twice their smallest. CPU medians are used in the tables, with both
clocks retained for reassessment.

## Reproduction

The probe uses a standalone Cargo workspace and the production crates from the
current checkout. Run the commands from the repository root with Cargo on PATH.
Timing uses the thread CPU clock on macOS or Linux.

```sh
baseline=docs/plans/core-performance-baseline
probe="$baseline/probe/Cargo.toml"
cargo run --release --locked --manifest-path "$probe" --target-dir target -- diagnostics > "$baseline/diagnostics.csv"
cargo run --release --locked --manifest-path "$probe" --target-dir target -- time > "$baseline/timings.csv"
cargo run --release --locked --manifest-path "$probe" --target-dir target --features allocations -- allocations > "$baseline/allocations.csv"
```

An optional argument after the mode filters cases by substring of
`system,workload,sites,factors,order,api`. For example, add
`'xx_gate_echo,16,16,2'` after `time` to rerun both 16-site echo queries.
The [probe source](probe/src/main.rs) and [lockfile](probe/Cargo.lock) accompany
the data.

## Measurement method

- Run on November 18, 2026 at production revision
  `690b7b4ca6a59f88c48d0310d9faa80ab258aff1` with a clean tracked tree. The host
  was an Apple M4 MacBook Air with 24 GB memory running macOS 15.8.1
  (`aarch64-apple-darwin`). Rust 1.99.1 / LLVM 23.1.1 used the default Cargo
  release profile. The probe uses `Complex<f64>` and sequential `usize` factor
  indices. Each spin site has one factor. A Hubbard site has two graded factors
  in site-major, up-then-down order.
- Each query warms up once and calibrates a power-of-two batch toward 10 ms of
  thread CPU time with a cap of 65,536 calls. Seven batch means in nanoseconds
  per call and their medians are recorded for thread CPU and elapsed time.
  A call includes output destruction. Input setup occurs before the timed
  call. Model builds include all syntax and binding from existing site handles.
  The `evaluate` row uses a prebuilt delayed action where applicable. State
  basis calls evaluate their input within the API. `expectation` includes action
  construction, evaluation, and contraction.
- Allocation requests and cumulative requested bytes come from a separate
  instrumented binary and one warm operation per row. They include zeroed
  allocations and the full requested size of reallocations. These sums measure
  allocation traffic. Peak live and resident memory require different counters.
- Diagnostic rows use the actual public APIs and assertions on coordinate
  counts, amplitudes, or scalar results. Counting wrappers observe index
  comparisons and basis expansion calls in separate invocations. The
  `syntactic_term_products` metric is derived from the input expression before
  combining terms. `nnz` is the number of stored entries after exact-zero
  elimination. The comparison tolerance for independently predicted values
  is 1e-10.

The [timing samples](timings.csv), [allocation counts](allocations.csv), and
[diagnostic counts](diagnostics.csv) are keyed by the case ID. `order` is zero
for state-only inputs, the number of repeated Hamiltonian actions or the
operator power for those sweeps, and one for ordinary single-action/one-model
cases. The XX echo uses order two for the layer and its inverse.

## Verification

The 310 timing and allocation rows use the same case IDs. Diagnostics passed
against a bit-state reference for repeated Heisenberg actions, analytic BLBQ
amplitudes, fermionic occupation and sign checks, and the known gate and state
outputs. Release tests and doctests passed for `syntaqs-core`,
`syntaqs-spin-half`, `syntaqs-spin-one`, `syntaqs-fermion`, and `syntaqs-tests`
with default features. Workspace and probe formatting checks passed, and the
standalone probe passed Clippy with all features and warnings denied.
