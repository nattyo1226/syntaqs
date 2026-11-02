# Partial trace proposal

Updated: 2026-11-02

These are mathematical contracts for an unimplemented feature. Timing and public APIs are undecided; current full trace, bases, and export do not depend on it.

## 1. Inputs, remaining space, and evaluation

Resolve two spaces for partial trace:

- `ambient`: the full Hilbert space, taking an explicit argument before the operator's binding.
- `trace_out`: the space whose basic factors identify the contracted degrees of freedom.

`trace_out` must be a subsystem of `ambient` under the [space subsystem contract](../design/hilbert-space.md#6-subsystem-checks). Callers may use retained internal-factor spaces; an entire site corresponds to all its factors. All operator references must also exist in ambient. An unbound expression with an explicit space needs no preliminary bind call. Reject missing binding/explicit space; do not infer ambient from trace_out. A bound operator still needs a separate trace_out argument.

Return an operator `B: R→R` bound to the remaining space R, preserving remaining indices, local structures, and order even for zero or scalar identity. Do not require rebinding by the caller. A delayed node may retain the selected pre-contraction ambient and contracted factors; later rebinding of the result must not change that original scope.

When the operator acts as identity on contracted factors, multiply by their dimensions. In particular:

```math
\operatorname{Tr}_{\mathcal H_1}
\left(A_0;\mathcal H_0\otimes\mathcal H_1\right)
= \dim(\mathcal H_1)A_0
```

Tracing out nothing returns an equivalent expression. Tracing out all factors returns a scalar operator expression on the zero-site Hilbert space; full trace returns the same mathematical value as a complex scalar.

For all systems including fermions, full and partial traces are ordinary, unnormalized Hilbert-space traces. Define partial trace using the embedded factor order and ordinary tensor representation. For remaining indices `r,r'` and a complete orthonormal contracted basis t, entries are `B_{r,r'} = Σ_t A_{r,t;r',t}`. Arrange composite indices in the original order; this definition does not require generating a matrix.

For fermions, respect ordering, grading, and the actual action including parity strings. Do not merely remove traced-out labels and re-embed the original syntax. Supertrace is not included; graded contraction must not change ordinary trace semantics.

Syntactic references differ from nontrivial factors in the ordinary representation. For two modes, `c_1=P_0⊗a_1` gives:

```math
\operatorname{Tr}_0(c_1)=\operatorname{Tr}(P_0)a_1=0
```

Treating mode 0 as an identity spectator and returning `2a_1` is wrong. The result is the zero operator on `H_1→H_1`, as required by ordinary partial trace.

For spinful fermions, tracing out only the up mode uses its internal factor; tracing out a whole site uses both factors.

Partial trace preserves linearity and adjoints. Full trace on the remaining space equals the original full trace, and successive traces over disjoint sets agree with tracing their union. Verify these and embedding consistency when implemented.

## 2. Verification and open questions

Public APIs, delayed nodes, remaining-space construction, and evaluator connections are unimplemented. Use `B[r,r′]=Σ_t A[r,t;r′,t]` as the meaning, with composite indices in the original order, without requiring full matrix generation.

Verify empty, proper, and complete contracted sets; linearity, adjoints, full trace, disjoint successive contraction, spectator dimensions, and space retention for zero/scalar results. Include internal-mode and whole-site spinful selections, graded expressions versus independent ordinary matrices, and invalid references/subsystems.
