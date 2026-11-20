# Core expression performance and refactoring plan

This baseline measures the public construction, evaluation, contraction, and
basis expansion APIs on spin-1/2, spin-1, and graded fermionic chains. It
records where cost follows the ambient space, the number of intermediate
contributions, or the requested output size.

[Implementation stages](#implementation-plan) · [Refactoring targets](#refactoring-targets-supported-by-this-baseline) · [Reproduction commands](#reproduction)

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

## Implementation plan

Reduce avoidable work in space construction, state and operator evaluation,
scalar contractions, and basis expansion. Use the agreed refactoring targets
as the design reference and the recorded measurements as the historical
baseline. The implementation includes the affected workspace callers and
basis implementations where API bounds or methods change.
[Refactoring targets](#refactoring-targets-supported-by-this-baseline)

The space index and ordered operator products come first because later
evaluators and basis queries consume them. State evaluation precedes state
contraction so both can share normalized structures. Operator evaluation
precedes direct operator contractions and basis conversion so each can use
the same factor blocks and collected terms.

### 0. Prepare the comparison

Record the production revision and run the existing probe on the environment
used for implementation. Check the probe's result assertions and match cases
to the original CSVs. Add an isolated comparison before a change when the
probe lacks the corresponding counter or case. Measure a first-use `bind`
with a fresh equivalent space prepared outside the timed call, and measure
repeated `bind` separately on a reused space. Capture a few controls for
binary `*` construction and explicit-space rebinding.
[Probe and reproduction commands](#reproduction)

### 1. Space construction and binding

Introduce the ordered factor-ID bounds at the affected constructors and
replace duplicate detection with a temporary `BTreeSet`. Then reuse binding
validation for actions on the same bound space. Use a lazily initialized
position index within each `bind` or operator query. Share it across validation,
operator basis placement, and graded operator ordering in that query. Keep
index construction compatible with core's `no_std` and current dependencies.

Measure these changes separately with `tensor_all`, `build_model`, first and
repeated `bind`, and `act` at 128–1,024 sites. Record equality and ordering
comparisons, index initialization counts, and allocations. Verify duplicate
and mixed-kind errors, missing indices, explicit rebinds, and factor orders
that differ from ID order.
[Duplicate detection](#check-duplicate-factor-ids-with-a-temporary-ordered-set) ·
[Binding reuse](#reuse-operator-binding-during-delayed-action) ·
[Shared factor positions](#share-factor-positions-on-the-hilbert-space)

### 2. Ordered operator products

Replace binary local and composite product nodes with ordered operand sequences. Migrate
construction, grouping, action, term traversal, formatting, destruction, and
basis product construction together. Use the common operand sequence for sums
and products, including `*`. Share nested operand handles. Check right-first action, adjoint
reversal, graded signs, and binding and error precedence for product sequences.

Measure binary `*`, sequence construction, and `expand()` of 64-site
Heisenberg `H²`. Count product nodes and operand-array allocations before
introducing placed-node sharing.
[Ordered operator product design](#store-operator-products-as-ordered-sequences)

### 3. State evaluation

Replace temporary factor IDs in state-term keys with ordered factor IDs.
Then change graded action to follow the state expression in tensor order.
Finally, keep normalized state terms through the full evaluation and build
returned syntax from the surviving structures. Verify collection between
actions, nested sums and graded scopes, exact cancellation, first-occurrence
order, and deep expressions at each applicable change.

Measure each step with the last-site `X` sweep, near/middle/far hopping, and
one through eight Heisenberg actions on Néel states. Track index comparisons,
created expression nodes, final configurations, and allocations. Include
the action-free product state as a control.
[State term keys](#use-ordered-factor-indices-in-state-term-keys) ·
[Graded action traversal](#traverse-the-state-expression-in-order-for-graded-actions) ·
[Returned state syntax](#construct-the-returned-state-expression-after-evaluation)

### 4. State contractions

Generate matching blocks on demand and verify exact-zero stopping with
differently grouped operands. Then use one normalization context and
conjugate symmetry at the common contraction entry point, including nested
matching sums and distinct outer coefficients.

Measure prepared blocks and evaluated pairs separately on W norms and the
factored plus norm. Check the W64 pair target of 2,080 against the original
4,096. Compare numerical results with independent references, including
complex off-diagonal terms and changes in floating-point summation order.
[On-demand blocks](#generate-matching-contraction-blocks-on-demand) ·
[Conjugate symmetry](#use-conjugate-symmetry-within-matching-contraction-blocks)

### 5. Operator evaluation and basis conversion order

Keep independent factor blocks in operator evaluation and combine overlapping
blocks in product order. Exercise the result through `collect_terms_in()`
and the basis queries. Validate the original placements before evaluation,
then apply basis rules to the surviving terms. Verify partially overlapping
supports, graded exchanges and adjoints, and the agreed zero and basis-error
behavior.

Measure the forward XX layer, XX echo, and Heisenberg `H²`. Count reached
intermediate products, surviving coordinates, and allocation requests. Use
the 16-site echo with one output coordinate and the forward layer with 256
coordinates to distinguish eliminated intermediate work from required output.
[Independent blocks](#preserve-independent-blocks-during-operator-evaluation) ·
[Basis conversion after evaluation](#convert-surviving-operator-terms-to-basis-coordinates)

### 6. Operator contractions

Use the block representation to contract Hilbert–Schmidt operands directly.
Measure direct contraction before adding symmetry within matching sums.
Apply local parity in `trace()` and Hilbert–Schmidt contraction, resolving
coefficient bounds for graded exchange signs. Compare against independent
ordinary matrices and cover mixed parity, repeated factors, scale order, and
the existing normalization of local traces.

Measure Heisenberg Hilbert–Schmidt self products and graded Hubbard `trace`.
Record evaluated pairs, pairs rejected by local parity or zero, parity
strings created, and allocations. The 64-site Heisenberg baseline has 35,721
candidate pairs, of which 17,955 are in one triangle including the diagonal.
Track the 252 hopping terms in the 64-site Hubbard trace.
[Direct Hilbert–Schmidt contraction](#contract-hilbertschmidt-operands-directly) ·
[Local parity](#use-local-parity-in-operator-contractions)

### 7. Basis coordinates and expanded syntax

Make the following changes in order and checkpoint each separately.

1. Introduce `Basis::expand_indices()` and `Basis::element()` and migrate
   built-in and test bases. Measure direct indexed conversion before adding
   query-local reuse of conversion results. Check basis order, invalid
   indices, and agreement between index and element APIs.
2. Enumerate operator coordinates using specified positions and spectator
   gaps. Check single-component identities with unit and nonunit
   coefficients and multi-component identities. Hold operator term
   collection fixed for this comparison.
3. Enumerate state tensor blocks with a reusable key and collect complete
   coordinates. Check nested tensors, sums within blocks, and exact zero.
4. Share the placed local nodes used to reconstruct states and operators
   by `(factor position, basis index)`. Check sharing and round trips from
   expansion back to coordinates.

Measure basis calls, visited factors, intermediate map entries, and placed
node counts as applicable. Use 64-site Heisenberg `H` and `H²`, the BLBQ and
Hubbard basis controls, 14-site GHZ and plus for both APIs, and 18-site plus
for `coordinates()`. Compare sharing against the post-product-change run.
[Indexed basis API](#expand-primitives-to-indexed-basis-coefficients) ·
[Operator enumeration](#enumerate-operator-coordinates-from-specified-positions-and-gaps) ·
[State enumeration](#enumerate-state-basis-coordinates-by-tensor-block) ·
[Placed-node sharing](#share-placed-basis-elements-across-expanded-terms)

### 8. Run the full comparison

Rerun the 310 baseline timing and allocation cases and the diagnostic
assertions. Compare end-to-end `evaluate()`, `expectation()`, contractions,
`coordinates()`, and `expand()` across spin-1/2, spin-1, and fermionic
workloads. Run workspace checks and downstream integration tests for the
affected APIs, using the repository verification guide. Summarize changes
by workload, including regressions and the behavior of the controls.
[Workspace verification](../../design/verification.md) ·
[Original measurements](#measurement-method)

### Performance records

Save each checkpoint's raw timing, allocation, and diagnostic output under
`measurements/` in this plan directory. Record the code revision, command,
configuration, and case selection alongside the results. Use the original
release workload definitions, case IDs, and coefficient types for matched
comparisons. Keep existing baseline CSVs as the historical record.

Summarize each checkpoint in a small table with the previous value, new
value, and change. Report median thread CPU and elapsed time, allocation
requests, cumulative requested bytes, and the relevant work counters. Record
output counts or scalar reference checks alongside the performance results.
Investigate regressions beyond observed timing variation before attributing
them to a change. Use the preceding checkpoint to attribute individual changes
and the stage 0 run on the same environment to report the cumulative result.

### Progress and completion

Stage 0 is complete. The reference run covers the original cases and adds
controls for first-use binding, explicit rebinding, and product construction.
Stage 1 is complete with factor positions scoped to one query. Sharing the
index across separate calls is deferred. Stage 2 is complete with ordered
`Operands` storage and shared construction paths for sums and products.
Stage 3 is complete with ordered factor keys, state-order graded traversal,
and deferred state syntax construction. Stage 4 is complete with on-demand
contraction blocks and conjugate symmetry for matching state sums. Stage 5 is
complete with support-aware operator evaluation and basis conversion after
natural term collection. Stage 6 is complete with direct Hilbert–Schmidt
contraction, conjugate symmetry, and local parity checks. Stage 7 is complete
with indexed basis conversion, query-local reuse, sparse operator enumeration,
state tensor block enumeration, and shared placed expression nodes. Stage 8
is complete with a paired comparison of all 310 original cases and 12
controls. Follow-up investigations include small operator-coordinate query
overhead, state action work before zero, repeated factor-position lookup, and
coordinate storage for dense support.
Completion requires the agreed changes, passing applicable correctness and
workspace checks, and a full comparison with explanations for material
regressions or missing measurements.
[Stage 0 measurements](measurements/stage-0/README.md) · [Stage 1 measurements](measurements/stage-1/README.md) · [Stage 2 measurements](measurements/stage-2/README.md) · [Stage 3 measurements](measurements/stage-3/README.md)
[Stage 4 measurements](measurements/stage-4/README.md)
[Stage 5 measurements](measurements/stage-5/README.md)
[Stage 6 measurements](measurements/stage-6/README.md)
[Stage 7 measurements](measurements/stage-7/README.md)
[Stage 8 results](measurements/stage-8/README.md) · [Follow-up investigations](measurements/stage-8/README.md#follow-up-investigations)
[Follow-up implementation sequence](#follow-up-implementation-sequence)

After the refactoring comparison, benchmark `Ord` and `PartialOrd` for state
and operator primitives using the agreed workloads with few and many
distinct primitives. Use those results to decide whether to change primitive
bounds. Orthogonality-based pruning of contraction pairs remains a separate
investigation. [Primitive ordering experiment](#measure-the-effect-of-ordering-primitive-values)

## Additional refactoring

The follow-up work covers the two items below. Use Stage 8 as the reference
and record before-and-after times, allocation traffic, and work counts for
each change. [Stage 8 results](measurements/stage-8/README.md)

### Follow-up implementation sequence

The zero-state change can be verified independently. Build the space index
before changing the tensor traversal so the evaluator can resolve the target
position in the action scope once per local placement. Measure the two new
construction APIs in separate checkpoints.

1. **Capture matched reference measurements.** Keep Stage 8 case IDs and
   add controls for successive `tensor()` construction, nested spaces with
   unsorted IDs, and retention of intermediate spaces. Record the baseline
   for work counters before changing evaluation, including product
   continuations after exact zero and tensor child tasks, temporary arrays,
   and key lookups. Use the existing timing and allocation modes for API
   comparisons and separately instrument work counts. The existing
   single-site action, Heisenberg, and Hubbard cases serve as controls.
   [Probe commands](#reproduction) · [Measurement method](#measurement-method)
2. **Stop product actions on exact zero.** Advance through the operands in
   action order. When an operand returns a normalized zero, finish that
   product before scheduling its next operand. Cover local and composite
   product actions, including products inside placed graded operators.
   Verify adjoint order and an outer sum with a nonzero sibling. Measure
   the Hubbard vacuum and the nonzero Hubbard input, then count work skipped
   when a remaining operand contains a sum.
   [State action requirements](#reduce-intermediate-work-in-state-action)
3. **Build the immutable factor index during space construction.** Merge
   child indices and perform duplicate detection at the construction
   boundary. Use the finished index for validation and operator factor
   lookup. Preserve the available index capabilities, borrowed child
   handles, and factor order. Compare construction alone with prebuilt-space
   `bind()` and `act_in()` on fresh and reused roots, including one-reference
   validation. Measure this checkpoint with the existing construction APIs.
   [Space index requirements](#reuse-factor-positions-across-calls-on-the-same-space)
4. **Add `flatten()`.** Promote the children of each immediate tensor
   child by one level and reuse the existing index. Verify leaf order,
   effective kind, and index sharing on nested spaces, including units and
   single-child wrappers. Measure `flatten()` on a prebuilt space and record
   construction and retained index size for repeated `tensor().flatten()`.
5. **Add `tensor_flatten()`.** Construct the flattened parent directly.
   Compare the result, validation errors, and effective kind with
   `tensor().flatten()`. Measure both paths with matched inputs and compare
   retained index data when callers release or retain intermediate spaces.
6. **Schedule only affected tensor children during state action.** Use
   the target position and the ranges of the stored tensor children to
   select work for each branch. Propagate coefficient changes using the
   original shape ID when child shapes agree. Build the final key once
   when the child shapes change. Verify block sums, exact zero, and parity
   prefixes with the existing ordinary and graded references. Measure the
   1,024-site local action, one and eight Heisenberg applications at ten
   sites, and near and far hopping on 128 modes. Compare the task, array,
   and key counts with the checkpoint immediately before this change.
   [State action requirements](#reduce-intermediate-work-in-state-action)
7. **Run the end-to-end comparison.** Rerun the Stage 8 sweep and the new
   controls. Compare construction alone, construction plus one validation,
   and reuse across validations. Report the number of calls that offset
   any added construction cost where the measured path breaks even. Report
   retained index size separately from cumulative allocation traffic.
   Run the applicable workspace and downstream checks and report any
   persistent regressions with their measured cost.
   [Workspace verification](../../design/verification.md)

The follow-up starts with the reference measurements. Save each checkpoint's
raw timing, allocation, and work-count results under `measurements/follow-up/`
with its revision and run configuration. Compare consecutive checkpoints for
individual effects and Stage 8 for the cumulative result.

### Reduce intermediate work in state action

Stop a product action as soon as an operand returns an exact-zero state.
Return zero once for the whole product branch. Skip the remaining operands
and their continuation tasks, including any distribution over sums inside
those operands. Continue evaluating sibling branches of an enclosing sum
and preserve the existing validation of operator placements.

Check a product whose first applied operand annihilates the input and
whose next operand is a sum. Include the product in an outer sum with a
nonzero sibling. Check the result and count continuations and sum branches
scheduled after the zero is known.
[Current action dispatch](../../../crates/syntaqs-core/src/evaluate/composite.rs)

Traverse the stored tensor child list and schedule work only for children
that need a local action or a parity action. Use each child's range in
tensor order to identify those children, and apply the same traversal to
each branch of a sum. Reuse the existing shape IDs of unaffected children
and avoid the temporary arrays that collect all children before and after
the action. If an action changes only the coefficient, update that
coefficient and reuse the tensor's shape ID. Build and look up a new tensor
key when a child shape changes.

Check a local action that changes only the coefficient and one that changes
the child shape. Include tensors with sum blocks and a graded action that
visits a parity prefix. Count scheduled child tasks, temporary child arrays,
and tensor key lookups during repeated Heisenberg action.
[Current tensor traversal](../../../crates/syntaqs-core/src/evaluate/composite.rs) · [Tensor keys](../../../crates/syntaqs-core/src/evaluate/state_terms.rs)

Count how much work precedes a zero result in graded Hubbard action on the
vacuum. Track the intermediate terms and state shapes rebuilt between
Heisenberg applications. Preserve exact-zero handling, operator action
order, and graded parity signs.

The Stage 8 reference is 3.18 ms and 15.0 MB in cumulative requests for the
64-site Hubbard vacuum, and 28.6 ms and 90.6 MB for eight Heisenberg actions
at ten sites. Check the zero result and the 222 final coordinates respectively.
Include the nonzero Hubbard input and the near and far hopping controls.
[Evaluation measurements](measurements/stage-8/README.md#follow-up-investigations)

### Reuse factor positions across calls on the same space

`FactorPositions` keeps the tensor-ordered leaves and the index from a
factor ID to its position for the duration of one call. Repeated `bind()`,
operator basis queries, and contractions can rebuild that data on the same
space. Build an immutable `by_index` at space construction and share it
through the `HilbertSpace` root. Keep the existing tree and use it to
enumerate leaves. Store entries with the existing factor handle and its
tensor position, sorted by ID. Merge the children's sorted indices with
position offsets and check for duplicate IDs during the merge. Use this
index for membership validation across operators that share the space.

Add two explicit ways to remove one level of grouping.

- `flatten()` promotes the children of each immediate tensor child by one
  level. Preserve leaf order and effective `TensorKind`. Reuse `by_index`
  because every factor keeps its position.
- `tensor_flatten()` produces the same space and errors as `tensor()`
  followed by `flatten()`. Construct the flattened parent directly and
  build its index once. Preserve the existing precedence of mixed tensor
  kinds over duplicate IDs.

Measure the eager index separately from the new APIs, using Stage 8 and a
matched run for the new construction cases. Record CPU and elapsed time,
allocation requests and bytes, factor-ID comparisons, and retained index size.

- **Construction.** Compare one-shot `tensor_all()`, successive `tensor()`
  calls, successive `tensor().flatten()` calls, and successive
  `tensor_flatten()` calls. Include nested inputs and unsorted IDs. Measure
  `flatten()` on a prebuilt space. Report how much index data stays live
  when only the final space is retained and when callers retain intermediate
  spaces.
- **Validation.** Compare `bind()` and explicit `act_in()` with space
  construction outside the timed call. Measure the first call on a fresh
  root and repeated calls on one root. Cover single-reference validation and
  independently built equivalent spaces. Keep operator `coordinates()` as
  a control for position lookup during evaluation. The 1,024-site Stage 8
  Heisenberg `bind()` takes about 130 µs per call on both fresh and reused roots.
- **Combined cost.** Include construction and validation in the comparison
  and report how many repeated validations offset any extra construction
  cost for each measured construction path.

Check factor order, missing references, duplicate IDs across children, and
ordinary and graded spaces with units and singleton wrappers. Preserve
`no_std` and thread sharing with existing dependencies.
[Binding measurements](measurements/stage-8/README.md#results-by-workload) · [Current factor lookup](../../../crates/syntaqs-core/src/space.rs)

## Refactoring targets supported by this baseline

| Cost exposed | Change to investigate | Baseline to compare |
| --- | --- | --- |
| Delayed action revalidates references on the operator's own bound space. | [Reuse operator binding during delayed action](#reuse-operator-binding-during-delayed-action). | Heisenberg `bind` and `act` at 128–1,024 sites. |
| Binding scans the target space for every local reference. | [Share factor positions on the Hilbert space](#share-factor-positions-on-the-hilbert-space). | First and repeated Heisenberg `bind` at 128–1,024 sites. |
| State evaluation assigns temporary factor IDs by linear search. | [Use ordered factor indices in state-term keys](#use-ordered-factor-indices-in-state-term-keys). | Single `X` on Néel and repeated Heisenberg actions. |
| Graded state action repeatedly scans the ambient factor prefix. | [Traverse the state expression in order for graded actions](#traverse-the-state-expression-in-order-for-graded-actions). | Directed hopping from mode zero to near, middle, and far destinations. |
| Intermediate state terms allocate expression nodes during evaluation. | [Construct the returned state expression after evaluation](#construct-the-returned-state-expression-after-evaluation). | One to eight Heisenberg actions on Néel. |
| Space construction scans previously seen leaves to detect duplicate indices. | [Check duplicate factor IDs with a temporary ordered set](#check-duplicate-factor-ids-with-a-temporary-ordered-set). | `tensor_all` at 128–1,024 sites. |
| Distributing operator products creates terms that can cancel within a smaller factor range. | [Preserve independent blocks during operator evaluation](#preserve-independent-blocks-during-operator-evaluation). | XX echo, forward XX layer, and `H²` coordinates and expansion. |
| State contractions prepare tensor groups before checking the first group for zero. | [Generate matching contraction blocks on demand](#generate-matching-contraction-blocks-on-demand). | W norms and the factored plus norm. |
| Contraction of identical sums evaluates both directions of each off-diagonal pair. | [Use conjugate symmetry within matching contraction blocks](#use-conjugate-symmetry-within-matching-contraction-blocks). | W norms and inner products with matching nested sums. |
| Hilbert–Schmidt contraction distributes the product of both operands before taking local traces. | [Contract Hilbert–Schmidt operands directly](#contract-hilbertschmidt-operands-directly). | Heisenberg Hilbert–Schmidt self products. |
| Graded contractions construct parity strings before testing for zero. | [Use local parity in operator contractions](#use-local-parity-in-operator-contractions). | Hubbard trace and graded Hilbert–Schmidt cases. |
| Operator basis queries validate conversions of terms that evaluate to zero. | [Convert surviving operator terms to basis coordinates](#convert-surviving-operator-terms-to-basis-coordinates). | Zero-scaled and cancelling operators, and errors on retained terms. |
| Operator basis queries repeatedly resolve factor positions in the selected space. | [Share factor positions on the Hilbert space](#share-factor-positions-on-the-hilbert-space). | Heisenberg `H²` and graded hopping coordinates. |
| Local basis conversion repeats primitive expansion and basis index lookup. | [Expand primitives to indexed basis coefficients](#expand-primitives-to-indexed-basis-coefficients). | Heisenberg `H²` coordinates and expansion. |
| Operator basis expansion scans every factor to find spectator positions in each term. | [Enumerate operator coordinates from specified positions and gaps](#enumerate-operator-coordinates-from-specified-positions-and-gaps). | Heisenberg `H` and `H²` coordinates and expansion. |
| State basis expansion builds a coordinate map for every prefix of a tensor product. | [Enumerate state basis coordinates by tensor block](#enumerate-state-basis-coordinates-by-tensor-block). | GHZ and plus states at a fixed site count, comparing `coordinates` and `expand`. |
| State and operator expansion allocate a placed local node for every digit of every surviving coordinate. | [Share placed basis elements across expanded terms](#share-placed-basis-elements-across-expanded-terms). | `expand()` on 14-site plus and GHZ states and 64-site Heisenberg `H²`. |
| Each expanded operator term allocates a chain of binary product nodes. | [Store operator products as ordered sequences](#store-operator-products-as-ordered-sequences). | 64-site Heisenberg `H²` expansion. |

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

### Reuse operator binding during delayed action

`bind` validates that every referenced factor belongs to the target space.
Use this guarantee when `act` selects the operator's stored binding or when
`act_in` selects a space sharing that binding's root node. For an unbound
operator or an explicit space with a different root, apply the same membership
validation as `bind` before constructing the delayed action. Then check that the
selected space and the state's space are compatible. Keep the selection
order of explicit space, stored binding, then state space, and record the
selected space as the scope of the action.

Verify that constructors and operator combinators preserve the binding's
factor membership guarantee. Cover method and multiplication forms of
delayed action, compatible state spaces with distinct roots, explicit
overrides, and the existing precedence of `MissingIndex` over
`IncompatibleSpaces`. In the shared-space, 1,024-site Heisenberg control,
expect the 3,145,725 factor-index comparisons in `act` to drop to zero.
Compare elapsed time and allocation requests against the baseline, and run
the separate first-binding control to confirm that it still validates
references. [Diagnostic counts](diagnostics.csv).

### Share factor positions on the Hilbert space

Stage 1 builds the index within one query. Store borrowed leaves in tensor
order and lazily sort a separate array of their positions by `I: Ord`. Find a
leaf directly by its tensor position. Find the position of an ID by binary
searching the second array. Membership validation scans the first reference
directly and uses the position index for later references. This avoids sorting
for an action with just one local reference. Basis queries reuse the index for
all phases that need reverse lookup.

Use the index for membership checks in `bind`, for axis positions in operator
basis expansion, and for computing exchange signs when graded operator terms
are put in factor order. Verify nested spaces, equal IDs on distinct leaf
handles, missing IDs, and factor orders that differ from ID order. The
within-query index uses existing dependencies. Sharing across calls through
the same space root is part of the [additional refactoring](#reuse-factor-positions-across-calls-on-the-same-space).

Compare first-use and repeated calls separately in the Heisenberg `bind`,
`H²` coordinates, and graded hopping coordinate controls. Count index
comparisons and measure allocation requests, requested bytes, and timing.
The 1,024-site `bind` baseline made 3,145,725 index comparisons and requested
50.4 MB cumulatively. [Space construction](../../../crates/syntaqs-core/src/space.rs) ·
[Allocation counts](allocations.csv) · [Diagnostic counts](diagnostics.csv).

### Check duplicate factor IDs with a temporary ordered set

Require `I: Ord` in the tensor-space constructors. After checking tensor
kinds, traverse the leaves in input order when combining two or more
children. Insert each borrowed ID into a temporary `BTreeSet<&I>` and return
`DuplicateIndex` at the first repeated ID. Discard the set after validation.
Queries build their factor-position index when needed.

Verify duplicates across nested children with distinct handles for the same
ID, input factor order, and precedence of `MixedTensorKinds`. Compare
ordering comparisons, allocations, and timing for `tensor_all` at
128–1,024 factors. The 1,024-factor baseline made 523,776 index comparisons.
The ordered set bounds a construction pass over `n` leaves to `O(n log n)`
comparisons. [Current duplicate check](../../../crates/syntaqs-core/src/space.rs) ·
[Diagnostic counts](diagnostics.csv).

### Use ordered factor indices in state-term keys

Require `I: Ord` for composite state evaluation. Compare the factor index
directly in the `BTreeMap` key for a local state term, alongside its primitive
and coefficient IDs. Share the existing factor handle to access its
index. This removes the query-local list that assigns a separate integer ID
to each factor. Carry the `Ord` bound through APIs that invoke state evaluation,
including state contractions and basis expansion.

Verify that distinct handles with the same index still match. Preserve
tensor factor order, graded signs, and first-occurrence order in collected
sums. In the 1,024-site single-`X` control, assigning IDs to the input factors
accounted for 523,776 of the 526,848 index comparisons. Count both equality
and ordering comparisons after the change. Compare timing and allocations
for the single-`X` and repeated-Heisenberg controls. [State-term keys](../../../crates/syntaqs-core/src/evaluate/state_terms.rs) ·
[Diagnostic counts](diagnostics.csv).

### Traverse the state expression in order for graded actions

Use the selected `Act` space's effective tensor kind to identify graded
actions. For each odd primitive, traverse that action's input state expression
in tensor order. Apply the parity operator to factors before the target,
apply the primitive at the matching index, and share the subsequent factors.
Pass whether the target has been reached from each tensor child to the next.
Start every sum branch with the same incoming value and keep the tensor
product factored. Each nested `Act` uses the order of its own input.
[Scope of delayed action](../../design/composite-expression.md#43-scope-of-delayed-act).

Verify signs for sums before and containing the target, nested action scopes,
and near and far targets. Rerun the directed hopping controls and compare index
comparison counts, timing, and allocations. The current traversal accounts
for 16,383 prefix-membership comparisons in the 128-mode far-hop control, as
derived from the implementation. [Diagnostic counts](diagnostics.csv).

### Construct the returned state expression after evaluation

Run operator actions and collect matching terms using the evaluator's
normalized state representation. Keep the local states, ordered child shape
IDs, and coefficient values needed for the result. Once `evaluate()` finishes
its actions and collection, construct `StateExpr` nodes for the structures
reachable from the final result. Reuse existing expression nodes where they
represent the normalized structure, and build newly needed nodes iteratively.

Preserve right-to-left product application, collection between actions,
coefficient evaluation order, exact-zero handling, and first-occurrence
order. Verify sums, nested graded actions, and deep expressions. Compare
`StateExpr` node creation, allocation requests, and timing for one and eight
Heisenberg actions. The eight-action, ten-site baseline took 47.3 ms and
issued 1,024,034 allocation requests. [State collection contract](../../design/composite-expression.md#114-natural-state-collection-between-actions) ·
[Allocation counts](allocations.csv).

### Generate matching contraction blocks on demand

Align the left and right factor sequences until their cumulative factor
counts agree, then contract that block before preparing the next one. When
a block has no further shared tensor boundary, evaluate its local inner
product or distribute one sum and process each branch through the same
contraction entry point. Stop a product when its running contraction reaches
exact zero and continue the other summands. Preserve the processing order
for scalar expressions that cover zero factors.

Verify differently grouped tensors and exact-zero contractions at early and
late factors. Compare how many blocks are prepared, allocation requests,
and timing for the W and factored plus controls. [Scalar contraction contract](../../design/composite-expression.md#115-incremental-scalar-contractions).

### Use conjugate symmetry within matching contraction blocks

Normalize both operands in one evaluation context so their structure IDs
can be compared. At the common contraction entry point, extract each block's
outer scalar coefficients and compare the normalized structures, including
the order and internal coefficients of any sum. When both sides have the
same normalized sum and its values compare equal to themselves, evaluate the
diagonal and one triangle of the sum's pair matrix. Add each off-diagonal
value `z` together with `conjugate(z)`. Apply the extracted scalars according
to the existing conjugation and exact-zero rules. Use the same entry for
the whole inner product and for nested blocks.

Verify matching sums inside otherwise different tensors, distinct outer
coefficients, and the existing equality rules for nonreflexive values.
Check numerical differences from the changed addition order. In the W64
control, expect the top-level pair count to fall from 4,096 to 2,080. Compare
the pair count and timing with the baseline. [State collection contract](../../design/composite-expression.md#114-natural-state-collection-between-actions).

### Contract Hilbert–Schmidt operands directly

Keep the two operator operands available separately during contraction.
Align independent blocks in ambient factor order and contract each block
before preparing later blocks. Distribute sums by linearity within a block
when needed. Compute the normalized local Hilbert–Schmidt inner product
and stop the current term pair on exact zero. Normalize primitive traces
before combining their coefficients, as in the current contraction.

Require `O: PartialEq` and `C: PartialEq` on composite `hs_inner_product()`
and `hs_inner_product_in()` so independently constructed expressions can
match by value. At the shared entry for whole and nested blocks, extract
outer coefficients and compare the normalized structure, primitive values,
and internal coefficients. For matching sums with reflexive values, evaluate
the diagonal and one triangle and use the complex conjugate for the reverse
pair. Restore the extracted coefficients with the existing conjugation rules.

Verify nested matching blocks, different scalar multiples, and numerical
differences from the changed addition order. In the 64-site Heisenberg
self product, the diagonal and one triangle contain 17,955 of the 35,721
candidate pairs. Compare pair counts, allocations, and timing. [Current Hilbert–Schmidt implementation](../../../crates/syntaqs-core/src/evaluate/trace.rs).

### Use local parity in operator contractions

Work with homogeneous terms and determine parity per factor after composing
same-factor operators in order. For `trace()`, a term with an odd local
product has zero trace. For Hilbert–Schmidt contraction, stop a pair when
the local parities differ at any factor. When they agree at every factor,
the parity factors cancel under the trace, so use the ordinary normalized
local Hilbert–Schmidt inner product. Stop the current pair on an exact-zero
local inner product and continue the other summands.

Preserve the exchange sign from ordering odd operators on different factors.
Composite contractions share the block evaluator's `C: Neg` capability.
Verify products of odd operators at the same factor, mixed-parity sums,
and exchange signs against independent small ordinary matrices. In the
64-site Hubbard trace, local parity can discard all 252 hopping terms
before parity string construction. Compare timing and allocations against the
baseline. [Graded operator representation](../../design/composite-expression.md#111-intrinsic-terms-and-ordinary-representations).

### Preserve independent blocks during operator evaluation

Determine the set of referenced factors for each operator subexpression.
Use each local placement's index, the empty set for identity, and the union
of the children's sets for a sum or product. Process product operands in
order, including the reversal from an adjoint. Keep disjoint supports as
factored blocks. Where supports overlap, evaluate the partial product on
their combined factors and collect identical terms there. Preserve the
order of operations on shared factors and account for graded exchange signs
whenever reordering disjoint homogeneous operands. Decompose mixed-parity
operands before using the homogeneous exchange rule.

Make this processing available to `collect_terms_in()`, basis expansion,
and contractions. Use local algebra rules to evaluate products and adjoints,
and use `O: PartialEq` and `C: PartialEq` on paths that collect terms by
value. Convert to a chosen basis at the basis-query boundary.
[Current operator collection](../../../crates/syntaqs-core/src/evaluate/collect.rs).

Verify partially overlapping supports, disjoint supports, and graded product
and adjoint order. Compare intermediate term counts, allocations, and timing
for the XX echo, the forward XX layer, and `H²`. The 16-site XX echo has
65,536 syntactic term products and one output coordinate in the baseline.
[Diagnostic counts](diagnostics.csv).

### Convert surviving operator terms to basis coordinates

Validate the original operator's factor references against the selected
space. Then evaluate the operator and collect matching terms. Run basis
conversion and validate its output for terms that survive, expanding
spectator identities as needed. When evaluation produces zero, return empty
coordinates or a zero operator on the selected space.

Update the existing zero-scaled operator tests to expect zero when an invalid
basis rule applies only to a removed term. Cover exact cancellation, invalid
basis outputs for retained terms, and missing factor references in the
original expression. [Current basis error rules](../../design/basis.md#74-errors-and-examples) ·
[Zero-scaled operator test](../../../tests/tests/core_basis_expansion.rs).

### Expand primitives to indexed basis coefficients

Keep integer basis indices as coordinate keys. Add
`Basis::expand_indices(&self, primitive: &P) -> LinearCombination<usize, C>`
to return indices and coefficients directly, and
`Basis::element(&self, index: usize) -> Option<P>` to retrieve a basis
element by its zero-based index. Implement `Basis::expand()` as a fallible
convenience that replaces each index with its basis element and reports a
missing element as a basis error.

Aggregate indexed contributions directly in core basis queries. Cache the
validated expansion of each encountered primitive within the query, and
apply each contribution's coefficient when using it. When returning an
expanded expression, fetch and share the basis element for each index that
appears in the final result.

Verify index bounds, missing elements, basis order, and agreement between
coordinate and expression results. Measure direct indexed conversion and
within-query reuse separately. The 64-site Heisenberg `H²` coordinate
control made 140,635 `Basis::expand()` calls in the baseline. Compare basis
call counts, allocations, and timing. [Current local basis conversion](../../../crates/syntaqs-core/src/basis/expansion/local.rs) ·
[Diagnostic counts](diagnostics.csv).

### Enumerate operator coordinates from specified positions and gaps

For each surviving operator term, order the specified factors by their
positions in the selected space. Derive the spectator ranges from the gaps
between those positions and the space boundaries. Convert the specified
local operators in factor order. At the first spectator range in the query,
expand and validate the local identity in the chosen basis, and reuse those
indexed coefficients for every spectator.

For a single identity component, prefill the working key with its index and
restore the changed digits between terms. Apply its coefficient across each
gap in factor order, skipping unit multiplications where the coefficient
arithmetic permits it. For a multi-component identity, enumerate its
choices at each spectator position in the ranges. Accumulate full coordinate
keys after all their digits are set.

Verify identities with a unit coefficient, a nonunit coefficient, and
multiple components. Cover terms that specify every factor, the unit space,
and coordinate and expression round trips. Compare factor visits, allocation
requests, and timing for Heisenberg `H` and `H²`, with the operator term
collection held fixed. [Current operator coordinate enumeration](../../../crates/syntaqs-core/src/basis/expansion/operator.rs) ·
[Existing identity cases](../../../crates/syntaqs-core/src/basis/expansion/operator/tests.rs).

### Enumerate state basis coordinates by tensor block

Expand each child of a tensor node once into coordinate and coefficient pairs.
A local child contributes a single digit. A child spanning several factors
contributes an ordered tuple as one choice, preserving correlations such as
the paired digits in `|00⟩ + |11⟩`. Use the same traversal for nested tensor
nodes and combine equal coordinates within each sum before using its result
as a child of an enclosing tensor.

Enumerate the Cartesian product of child coordinates in tensor order. Reuse
one working digit buffer sized to the tensor's factor count, writing each
child's tuple into its corresponding slice. Carry the coefficient product
in the existing multiplication order and drop exact-zero prefixes. Copy a
key into the result map when all digits of the tensor are filled. Share this
enumeration between `coordinates()` and `expand()`, building expanded
expression nodes from the surviving coordinates.

Verify block correlations on `(|00⟩ + |11⟩) ⊗ |+⟩` and cover nested tensors,
factor order, and exact cancellation. Compare allocations and timing for
both APIs on 14-site GHZ and plus states and for `coordinates()` on the
18-site plus state. The latter currently stores 262,142 entries in maps for
its nonempty proper prefixes and 262,144 entries in the full product.
[Current state basis traversal](../../../crates/syntaqs-core/src/basis/expansion/composite.rs).

### Share placed basis elements across expanded terms

In each state or operator expansion, build the returned terms from the
surviving coordinates. Retrieve basis elements by index and share each
`LocalState` or `LocalOperator` across positions. Lazily create one placed
`StateNode::Local` or `OpNode::Local` per `(factor position, basis index)`.
Populate the tensor or product of each scaled term by cloning the
`Arc`-backed handles. Operator expansion uses the same sharing for basis
elements at spectator factors, including each component of a spectator
identity with several basis coefficients.

Verify shared node identity, factor order, and agreement between expanded
terms and sparse coordinates. The 14-site plus control creates 229,376
placed local nodes in the baseline. The target is 28 placed nodes for the
same 16,384 terms. For 64-site Heisenberg `H²` in the Cartesian basis, the
baseline creates 1,113,280 placed nodes for 17,395 terms and the target is
at most 256 placed nodes. Compare placed node counts, allocation requests,
and timing for these workloads and the 14-site GHZ control, where reuse
is low. [Current term construction](../../../crates/syntaqs-core/src/basis/expansion/composite.rs).

### Store operator products as ordered sequences

Store local and composite operator products as ordered sequences. Construct
the sequence directly in `LocalOperator::product()`, `Operator::product()`,
`try_product()`, operator basis enumeration, and expansion. Sums and products
use the same sequence representation, including binary `*`. Use shared handles for
existing product operands so successive `*` calls avoid copying their child
lists. Use identity for an empty product and the operand for a singleton.

Use the sequence in validation, structural grouping, term traversal, state
action, formatting, and iterative destruction. Apply the operands from right
to left, reverse their order under adjoint, and calculate graded exchange
signs when changing factor order. Preserve nested product grouping and the
sequence constructors' binding and input-error precedence.

Verify order-sensitive products on one factor, graded products and
adjoints, and basis expansion round trips. In the 64-site Heisenberg `H²`
control, the baseline builds 1,095,885 product nodes. The target is one
product node and one child array per coordinate, giving 17,395 product
nodes. Measure this cost separately from local node reuse and compare
binary `*` construction as well. [Current product construction](../../../crates/syntaqs-core/src/expression/composite.rs) ·
[Current expansion construction](../../../crates/syntaqs-core/src/basis/expansion/composite.rs).

### Measure the effect of ordering primitive values

In a later benchmark, compare the current `PartialEq` scans with approaches
that require `Ord` or `PartialOrd` on state and operator primitives. Measure
state primitive interning in `evaluate()` and operator primitive collection
in `collect_terms()` and `collect_terms_in()`. Include both built-in spin and
fermion cases and a workload with many distinct primitives. Track timing and
comparison counts, and account for incomparable values in any `PartialOrd`
approach. [State primitive interning](../../../crates/syntaqs-core/src/evaluate/state_terms.rs) ·
[Operator primitive collection](../../../crates/syntaqs-core/src/evaluate/collect.rs).

## Reproduction

The probe uses a standalone Cargo workspace and the production crates from the
current checkout. Run the commands from the repository root with Cargo on PATH.
Timing uses the thread CPU clock on macOS or Linux. Set the results
directory for the checkpoint being measured. The example records the
stage 0 reference run.

```sh
plan=docs/plans/core-performance-baseline
probe="$plan/probe/Cargo.toml"
results="$plan/measurements/stage-0"
mkdir -p "$results"
cargo run --release --locked --manifest-path "$probe" --target-dir target -- diagnostics > "$results/diagnostics.csv"
cargo run --release --locked --manifest-path "$probe" --target-dir target -- time > "$results/timings.csv"
cargo run --release --locked --manifest-path "$probe" --target-dir target --features allocations -- allocations > "$results/allocations.csv"
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
