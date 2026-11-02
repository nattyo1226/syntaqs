# Automatic differentiation and tensor-network connections

Updated: 2026-11-02

## 1. Boundary with external libraries and automatic differentiation

Automatic differentiation, ML modules, and tensor-network connections are unimplemented and are not prerequisites for current coordinates or dense/sparse export.

The algebra core would handle spaces, bases, expression structure, embedding, algebraic computation, and reusable coefficient generation. External libraries handle numeric coefficient representations, differentiation, and output containers. Connections must preserve domain/codomain, factor order, basis, precision, and parameter dependencies.

Future tensor-network export must reconcile backend contraction conventions while preserving Hilbert-space inner products/norms, ordinary traces, and actions specified by [current composite evaluation](../design/composite-expression.md).

Symbolic tractability and differentiability are separate requirements. The library should construct parameter-independent algebra and reusable coefficients for external connections without requiring an automatic differentiation engine in core.

Future differentiable modules must preserve external parameter dependencies without cutting the computation graph through conversion to ordinary numbers. A parameter-dependent term must not be discarded solely because its current value is zero.
