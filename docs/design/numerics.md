# Coefficient types and numeric conventions

Updated: 2026-11-02

This document defines coefficients, operation-specific capabilities, numerical zero, and approximate comparison. See [coefficient.rs](../../crates/syntaqs-core/src/coefficient.rs) for implementation.

## 1. Coefficient types and capabilities

Expressions store the whole coefficient C. Standard examples are `Complex<f32>` and `Complex<f64>`; custom coefficients connect by implementing the capabilities an operation needs. Primitives themselves do not fix numerical precision.

| Capability | Source and role |
| --- | --- |
| `Zero`, `One`, arithmetic traits | num-traits and standard traits, requested only where used. |
| `Conjugate` | Conjugation from a shared reference, used by inner products and adjoints. |
| `RealPart` | Extracting the real part of a self-inner-product for squared norms. |
| `ImaginaryUnit` | The imaginary unit required by particular local rules. |
| `Divide` | Mathematical coefficient division for normalized expectations and HS inner products. |
| `From<R>` / `Into<C>` | Real embedding and input conversion to the selected coefficient type. |

Storage, sharing, and delayed construction without coefficient arithmetic require neither numeric operations nor Clone on coefficients or primitives. Coefficients are commutative scalars whose capabilities obey the unit, distributive, conjugation, and real-embedding laws. There is no mandatory trait combining all capabilities or numeric provider.

Operations do not mix coefficient precisions or implicitly promote/narrow them. Left and right scaling have the same mathematical meaning; `scaled` works for arbitrary C. Standard complex coefficients have concrete left-scaling implementations. See [construction](expression-construction.md) for real inputs and inference.

Arithmetic, exact comparison, rounding, NaN, and infinity follow the numeric type without additional rejection, correction, or substitution. Local rules and expansion follow mathematical structure, so they do not guarantee identical rounding or nonfinite-value propagation to the original sequence of arithmetic operations.

## 2. Divide and complex division

`Divide` is independent of standard Div and permits representation-specific division. A nonzero divisor is the caller's contract: normalized expectations check exact zero self-inner-products, while HS inner products require a nonzero local dimension `Tr(I)`. Even with truncating arithmetic such as integers, operations using fractions in rules, bases, or models require mathematical fractions to be representable.

The standard `Complex<T>` implementation requires `T: PartialOrd`. Purely real divisors are divided directly. Otherwise, the divisor is scaled by its largest component magnitude before squaring. If scaling the dividend overflows, the dividend is halved first and the factor restored afterward. Small components are summed before division by the normalized squared divisor norm; large components are divided before summing. This avoids unnecessary squared-denominator overflow/underflow even for tiny imaginary rounding residuals. It adds no Float, Copy, One, or math backend requirement, and no ordering bound to custom C implementations or expression methods. Truncating division retains the original direct formula.

## 3. Approximate equivalence and verification

This section specifies comparison in tests and reference calculations; there is no general public equivalence-checking API.

Numerical approximate equivalence means corresponding values agree within explicit tolerances. Equivalence as an action means the same linear action on the same Hilbert space.

For complex `a` and `b`, use absolute tolerance `atol` and relative tolerance `rtol`:

```math
|a-b| \leq \mathrm{atol}
       + \mathrm{rtol}\max(|a|,|b|)
```

Approximate equivalence is not transitive and must be checked separately from exact equality. A value containing NaN is never approximately equal, including to itself. Small-system numerical representations may serve as references for action equivalence.

Tests state their `atol` and `rtol`. Comparison and rank tolerances depend on precision and problem size, using independently defined vectors and matrices. Test tolerances must not remove terms in ordinary evaluation.

Evaluation, expansion, and coordinates remove zero coefficients only via exact `Zero::is_zero()`, retaining small nonzero coefficients. Local coefficient generation follows [local primitive contracts](local-structure.md#3-consistency-of-mathematical-capabilities).

Mathematical cancellation in general expressions, basis round trips, and associativity need not yield exact floating-point zero. Compare matrices, actions, and coordinates with tolerances rather than requiring identical sparse term counts. A nonzero cancellation residual need not produce ZeroNorm during normalization and may be amplified. Verify product chains, basis round trips, cancellation between large and small coefficients, and preservation of tiny nonzero values; check analytically zero local rules for exact emptiness.
