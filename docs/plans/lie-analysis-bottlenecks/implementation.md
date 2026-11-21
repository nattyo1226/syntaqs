# Lie analysis refactoring plan

Reduce repeated operator evaluation in closure and decomposition, then
reduce the memory used to build symmetric adjoint constraints. The main
changes fit within the existing Lie and core responsibilities. The
commutator and closure operand-reuse candidates need separate experiments
because they change how the core evaluator prepares operands.
[Measured bottlenecks and candidate rationale](README.md).

## Scope and progress

The plan covers the six items from the investigation. Direct core
Hilbert–Schmidt (HS) contraction addresses both closure RHS construction
and the orthonormality checks on reconstructed components. Closure RHS
reuse is a follow-up to that change. The dense-constraint work uses direct
construction of symmetric constraints with the same SVD problem.
The paired runner, direct HS refactoring, Gram reuse in center extraction,
exact-zero residual diagnostics, direct symmetric constraint construction,
indexed commutator filtering, and closure RHS reuse are complete. The results
record verification, raw data, and reproduction for [stage 1](results/stage-1/README.md),
[stage 2](results/stage-2/README.md), [stage 3](results/stage-3/README.md),
[stage 4](results/stage-4/README.md), [stage 5](results/stage-5/README.md), and
[stage 6](results/stage-6/README.md#stage-6-prepared-closure-right-hand-sides).
The stage 6 report includes the combined comparison against the v6 merge.

Preserve ordinary Lie brackets on ordinary and graded spaces, normalized
HS inner products, and acceptance under the configured tolerances in
`f32` and `f64`. Keep space and finite-input validation, ordered closure
diagnostics, and validation of the emitted component expressions.
Collect the full projection residual after subtracting the reconstructed
projection so cancellation can occur across the difference.
[Projection and numerical scales](../../design/lie.md#3-projection-and-numerical-scales),
[decomposition contract](../../design/lie.md#7-complete-center-and-simple-ideal-decomposition).

## Stages

| Stage | Change | Primary comparison |
| --- | --- | --- |
| 0 | Rebuild and measure the pre-refactoring implementation | Reproduce the post-merge baseline under paired conditions |
| 1 | Use core HS directly for composite Lie expressions | Closure and Pauli component rechecks |
| 2 | Reuse the Gram matrix during center extraction | One Gram evaluation, center and decomposition times |
| 3 | Reuse the exact-zero rank-analysis result for residual diagnostics | Per-column diagnostics and abelian full commutant time |
| 4 | Construct the symmetric constraints directly | Constraint equivalence and decomposition peak RSS |
| 5 | Prototype term-pair filtering for commutators | Work counts and active-chain closure time |
| 6 | Evaluate RHS operand reuse if repeated evaluation remains material | Work counts and closure time at larger Lie dimension |

Run stages 1–4 as independently verifiable changes. Stage 1 comes first
because its trial already identified a common cost in two public workflows.
Stages 2–4 address separately measured work inside the existing numerical
pipeline. Compare stage 5 with the direct HS path in place. Use the latest
profile, including stage 5 if adopted, to decide whether stage 6 warrants
a new reuse boundary between Lie analysis and core.

### 0. Prepare paired comparisons

Reuse the benchmark driver, pinned dependencies, workload list, and
measurement code. Allow the benchmark runner to compare executables from
the current implementation and each candidate using explicit build labels.
Keep the archived merge comparison reproducible. Record each comparison's
source revision, dependencies, BLAS backend and thread count, precision,
and numerical settings. Build in isolated scratch directories and
alternate execution order. Use uninstrumented release builds for API time
and process RSS, and instrumented builds for phase times and work counts.
[Probe and reproduction setup](README.md#measurement-and-reproduction).

### 1. Use core HS directly for composite Lie expressions

Apply the measured change at the composite `FamilyExpression::hs_in`
adapter. Core already prepares the factor blocks and collects matching
natural terms within its evaluation. Keep the local operator path and the
explicit collection used to produce the returned component expressions.
Retain Lie family binding checks and core validation of placed references.
[Trial patch](probe/direct-hs.patch),
[composite adapter](../../../crates/syntaqs-lie/src/expression.rs#L103).

Verify nonorthonormal projections, small nonzero residuals, singular
families, and invalid bindings in addition to the existing closed
benchmarks. The trial matched closure diagnostics on 34 inputs in `f64`.
Use the existing tests in both precisions to check acceptance under the
configured tolerances. Measure `pauli-2`, `pauli-2-mixed`, `heisenberg-16`,
and `fermion-4`, including closure and the Pauli component-check phase.
[Trial measurements](README.md#direct-core-hs-probe),
[existing verification cases](../../design/lie.md#8-verification).

### 2. Reuse the Gram matrix during center extraction

Share the computation of the HS-orthonormal transform through an internal
operation that uses an existing Gram matrix. In `orthonormal_coordinates`,
compute and validate `G` once, obtain `U`, and use the same `G` to form
`UᵀG`. Reuse the transform calculation in the public family
orthonormalization path, which also returns the transformed family.
Preserve the checks governing Hermiticity, independence, and real Gram
entries. [Current transform](../../../crates/syntaqs-lie/src/family/transform.rs#L140),
[center coordinate preparation](../../../crates/syntaqs-lie/src/invariant.rs#L219).

Count Gram evaluations and check center/complement dimensions, projectors,
centrality, and decomposition diagnostics. Include real nonorthonormal
families, zero-dimensional centers, and wholly abelian families. Measure
`center()` and `decompose()` on `heisenberg-16` and `pauli-z-4`.
[Measured repeat cost](README.md#refactor-center-extraction-to-reuse-the-gram-matrix).

### 3. Reuse exact-zero information in residual diagnostics

Carry the verified zero-matrix result from the rank kernel to the
diagnostics builder. Construct one tolerance check with zero norm and
zero scale per nullspace column through the existing tolerance API.
Use the matrix's exact-zero classification to select the path. Numerical
rank classification continues to govern the nullspace dimension.
Preserve validation and the reported shape, cutoff, singular values, and
residual fields. [Zero-constraint refactoring item](README.md#refactor-residual-validation-for-zero-constraints).

Exercise exact-zero and empty matrices and a nonzero constraint with a
numerical nullspace in both precisions. Compare the per-column checks
and dimensions. Measure the full `pauli-z-4` commutant and confirm that its
constraint-by-nullspace multiplication is skipped.
[Rank kernel tests](../../../crates/syntaqs-lie/src/numerical/kernels/tests.rs).

### 4. Construct symmetric adjoint constraints directly

In `symmetric_commutant`, write the columns for the Frobenius-orthonormal
symmetric basis directly using the action `C Aᵢ − Aᵢ C`. Reuse the current
row order and diagonal/off-diagonal weights. For a subalgebra of dimension
`d`, retain the `d³ × d(d+1)/2` shape and current SVD, rank, and residual
settings. This removes the `d³ × d²` intermediate from the symmetric
path. [Current construction](../../../crates/syntaqs-lie/src/invariant.rs#L364).

In tests, compare with the existing full-constraint projection. Cover
contributions where the left and right products update the same entry
and both supported precisions. Check ranks, residuals, and ideal
projectors against the reference. On `fermion-4`, verify the removed
intermediate and measure peak RSS through decomposition. Record SVD
time separately from constraint construction.
[Stage 4 results](results/stage-4/README.md).

### 5. Prototype term-pair filtering for commutators

Give the evaluated commutator path access to both validated operands
so the core can use their supports and parity before expanding either
product. Prototype with one sparse sum per operand and use the existing
evaluator for other operand layouts. A site-to-term index selects
overlapping pairs and deduplicates pairs that overlap at more than one site.
Ordinary disjoint pairs cancel. Graded disjoint pairs with an even homogeneous
operand cancel, and odd–odd pairs contribute `2ab` to the ordinary
commutator. Use the existing ordering signs and homogeneous components.
[Commutator candidate and work counts](README.md#skip-commuting-term-pairs-in-commutators).

Keep symbolic bracket construction available through the existing API.
For production adoption, keep term indexing within the core evaluator
and route Lie analysis through an evaluated-bracket operation with an
explicit space. The prototype should establish the useful operand
shapes and workload coverage before choosing a reusable cross-crate
entry point. Preserving factored blocks is part of the evaluation
boundary. [Symbolic Lie bracket contract](../../../crates/syntaqs-lie/src/lib.rs#L197),
[core block evaluation](../../design/composite-expression.md#117-operator-block-evaluation).

Compare the bracket to the current evaluator on ordinary and graded
overlaps, disjoint even and odd pairs, and mixed parity. Measure index
setup, pairs visited, terms built, and total closure time. Use
`heisenberg-16` as the main workload and the smaller and spectator
Heisenberg cases plus the graded cases as controls. The production path
uses the index for eligible closure brackets. [Stage 5 results](results/stage-5/README.md).

### 6. Evaluate reuse within closure RHS construction

Reprofile RHS construction after direct HS delegation and any adopted
commutator optimization. If operand evaluation still dominates, use an
evaluation scope tied to one validated family context. Prepare each
basis operand at most once for the RHS and each bracket once per
ordered pair, then contract that bracket against the prepared basis.
Caching collected expressions in Lie would still require preparing blocks
on each core HS call. A scoped core operation for repeated HS contractions
keeps the evaluated blocks and their primitive numbering private.
Lie analysis continues to own the projection and span diagnostics.
[RHS construction](../../../crates/syntaqs-lie/src/family/projection.rs#L349).

Measure the change from repeated bracket evaluation towards at most
`n²` RHS bracket evaluations across closure. The RHS still requires
`n³` inner products. Record the preparation and memory costs along with
the resulting closure time on `fermion-4`, `pauli-2-mixed`, and
`heisenberg-16`. Use the original diagnostics to verify each ordered pair
and evaluate the full reconstructed residual at its original scale.
[Stage 6 results and adoption](results/stage-6/README.md).

## Completion criteria

1. For each adopted stage, verify the affected entry points and record
   the comparison against the preceding implementation. For stages
   5 and 6, record the adoption decision and its measured basis.
2. Run the existing core, Lie, and shared Lie integration tests, including
   doctests, and the applicable workspace checks. Compare closure
   diagnostics and structure constants on the recorded workloads.
   Compare center and ideal subspaces by projectors or spans, allowing
   equivalent bases within a degenerate subspace.
3. Rerun the primary workload, site scaling, and RSS suites on the
   combined change. Keep revision-tagged raw results and publish the
   changes in times and work counts in the investigation.
4. Reconcile the accepted evaluation paths into the Lie design at
   implementation closeout, including the documented pre-contraction
   collection behavior.

[Verification guide](../../design/verification.md),
[numerical regression coverage](../../design/lie.md#8-verification).
