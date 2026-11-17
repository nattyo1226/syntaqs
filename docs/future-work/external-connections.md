# Automatic differentiation and tensor-network connections

Updated: 2026-11-17

External connections would let syntaqs generate reusable algebraic coefficients
while other libraries handle numeric representations, differentiation, or
numerical containers. Automatic differentiation, ML modules, and tensor-network
connections are unimplemented; current coordinates and dense/sparse export work
independently of them.

Core would retain responsibility for spaces, bases, expression structure,
embedding, algebraic computation, and coefficient generation. Every connection
must preserve domain/codomain, factor order, basis, precision, and parameter
dependencies.

Tensor-network export would need to reconcile backend contraction conventions
with the inner products, norms, ordinary traces, and actions specified by
[current composite evaluation](../design/composite-expression.md).

Symbolic tractability and differentiability are separate requirements.
Parameter-independent algebra should be constructible without an automatic
differentiation engine in core. A differentiable connection must preserve the
external computation graph through conversions and retain parameter-dependent
terms even when their current value is zero.
