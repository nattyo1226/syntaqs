# Norms, state normalization, and normalized expectations

Updated: 2026-11-02

`syntaqs-normalize` provides extension traits for square-root-based norms and normalization, and for expectations divided by the self-inner-product. See [numerics](numerics.md) for coefficient capabilities and [local](primitive-expression.md)/[composite expressions](composite-expression.md) for actions and inner products.

## 1. Norm and normalization APIs and features

Let C be the whole coefficient and R the real type associated with `C: RealPart`. Inner products, expectations, squared norms, and scalar division belong to `syntaqs-core`. The following extension traits and `NormalizationError` belong to the std-based `syntaqs-normalize` crate.

| Operation | Provider | Local-state result | Composite-state result |
| --- | --- | --- | --- |
| `norm_squared()` | Core state expressions | R | `Result<R, ExpressionError>` |
| `norm()` | `Norm` | R | `Result<R, ExpressionError>` |
| `normalized()` | `Normalize` | `Result<Self, NormalizationError>` | Same |
| `normalized_expectation(&operator)` | `NormalizedExpectation` | `Result<C, NormalizationError>` | Same |
| `normalized_expectation_in(&operator, &ambient)` | `NormalizedExpectationIn` | Unavailable | `Result<C, NormalizationError>` |

The squared norm is the real part of the self-inner-product; the norm is its square root. Only `norm()` and `normalized()` require `R: num_traits::Float`; normalization additionally requires `From<R>` to embed the real scale in C. Normalization scales a shared expression by its real inverse norm, preserving phase and leaving the input unchanged. It uses the ordinary action/inner-product path without expansion or matrices.

Normalization and normalized expectations reject an exactly zero full self-inner-product, using `Zero::is_zero()`, as `NormalizationError::ZeroNorm`. This includes explicit zero and cancellation between dependent primitives or terms. `norm(zero)` returns zero. There is no negative-value correction or approximate zero test; NaN and infinity follow the numeric type. Composite space/evaluation errors become `NormalizationError::Expression(ExpressionError)` with the original error retained as source.

Normalized expectations divide the expectation by the full self-inner-product and require `Divide`, but not square roots, real-part extraction, or real embedding. Complex division follows [numerics](numerics.md). An explicit ambient space overrides an operator's existing binding.

Import extension traits and errors directly from `syntaqs-normalize`. Core has no libm feature; its algebra APIs, squared norms, and scalar division work without the normalization crate. Quotients by nonzero self-inner-products can be computed explicitly; the convenience trait for expectations without square roots still belongs to normalize.

## 2. Mathematical meaning and verification

An ordinary expectation is `⟨ψ,Aψ⟩` with no denominator inserted for unnormalized states. A normalized expectation is `⟨ψ,Aψ⟩/⟨ψ,ψ⟩`. State normalization is `ψ/√Re⟨ψ,ψ⟩`, preserving phase.

Composite expressions use the shared ordinary/graded evaluator and preserve the selected space. No numerical matrix or basis expansion is required. Mathematical cancellation in a general expression need not yield ZeroNorm if floating-point residuals remain nonzero.

Tests live in [contracts.rs](../../crates/syntaqs-normalize/tests/contracts.rs), [expectation.rs](../../crates/syntaqs-normalize/tests/expectation.rs), [three-level integration tests](../../tests/tests/normalize_three_level.rs), and [graded integration tests](../../tests/tests/normalize_tensor_products.rs). They cover phase preservation, exact-zero rejection, minimal custom-coefficient bounds, basis-independent computation, and independent matrix references at both precisions.
