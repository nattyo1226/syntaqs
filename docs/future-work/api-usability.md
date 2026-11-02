# API usability candidates

Updated: 2026-11-02

These are unadopted, unstarted candidates for helper APIs and clearer explanations, not a list of approved additions. Names, signatures, and timing are undecided. Create a plan in `plans/` when concrete implementation starts.

Current APIs are described in [construction](../design/expression-construction.md), [composite evaluation](../design/composite-expression.md), [geometry](../design/geometry.md), and [export](../design/export.md).

## 1. Additional ExpressionFactory helpers

These candidates were excluded from the initial implementation. Existing constructors and expression methods already perform the same operations; consider helpers when real examples show substantial repetition.

| Candidate | Role |
| --- | --- |
| `state_at(primitive, &factor)` | Place one state primitive. |
| `operator_at(primitive, &factor)` | Place one operator primitive. |
| `state_zero(&space)` | Zero state on an explicit space. |
| `state_unit()` | Unit state on the unit space. |
| `operator_zero()` | Unbound zero operator. |
| `operator_identity()` | Unbound identity operator. |
| `local_state_zero()` | Local zero state. |
| `local_operator_zero()` | Local zero operator. |
| `local_operator_identity()` | Local identity operator. |
| `local_state_sum` / `try_local_state_sum` | Collect existing local states. |
| `local_operator_sum` / `try_local_operator_sum` | Collect existing local operators. |
| `local_operator_product` / `try_local_operator_product` | Multiply existing local operators. |

Local-expression collection methods would be factory entry points, distinct from existing expression sum, try_sum, and product methods. Preserve empty/singleton input, ordering, and delayed construction, requesting only required capabilities. Zero/unit construction must not require numeric arithmetic; composite expressions without a space argument may need an index-type annotation.

## 2. Space inspection and output type inference

### 2.1. Finding an actual leaf by index

There is no `HilbertSpace::find_factor(&index)` equivalent. Leaves can currently be searched through `leaves()`, so assess the value of reducing repeated calls. Such a helper should require Eq and return a reference to the actual leaf without cloning/issuing indices, reordering factors, or constructing a subsystem.

### 2.2. Inferring MatrixArgs from a state

Matrix export still specifies a state primitive, such as `MatrixArgs::<SpinHalfState, _>`, independently of the coefficient type. A `MatrixArgs::for_state(&psi, &basis)` equivalent could remove that annotation.

- Should the state supply only type inference, leaving ambient-space selection to existing bindings and explicit arguments?
- If it also supplies the space, how should selection and lifetimes be explicit?
- Should local states be supported, and what guidance applies when no state exists?
- How should state-type inference be explained alongside result-type container selection?
- Are named output helpers needed in addition to selecting the container through the return type?

Consider this in syntaqs-export without adding a spin-half dependency on export. Preserve the [export contract](../design/export.md).

## 3. Ambient-space and evaluation guidance

Placement, the full ambient space including spectators, binding, explicit-space precedence, and when composition becomes unbound affect interpretation. Use examples to assess whether one Operator type and its documentation are sufficient. Distinguishing binding states in types is undecided.

Explain when explicit evaluate is necessary, when an expression can be passed directly to inner products/expectations, and how evaluation differs from aggregation by basis expansion. This does not approve evaluator changes. Display and Debug already follow [expression formatting](../design/expression-formatting.md). Any complexity or memory diagnostics should have a clear boundary with [deferred expression management and performance](../future-work.md).

## 4. Error diagnostics

`ExpressionError::MissingIndex` and `IncompatibleSpaces` do not store the offending index or expected order. Consider actionable diagnostics without imposing Debug, Display, or Clone on every index.

Preserve ordinary `?` error handling while helping users find invalid placement, missing/excess factors, and order mismatches. Diagnostic improvements must not silently change try_ error types, precedence, or short-circuit behavior.

## 5. Onboarding and custom-algebra guidance

[Examples](../../examples/README.md) cover manual construction, expansion, chain models, and numerical export. Further documentation candidates are:

- Dependencies, features, and trait imports for common uses, distinguishing missing imports from disabled features.
- Connections between crate-root guidance, rustdoc, designs, and runnable examples. Repository and library READMEs already exist.
- A minimal user-facing custom ExpressionFactory alias, capability traits, and Basis derived from the test [three-level system](../../tests/src/three_level.rs).
- Factor/basis/container order, metadata sharing and borrowing, and inner products in non-orthonormal coordinates.

Reducing import line counts is not itself a priority; no prelude or unified entry crate is adopted. Explain the relationship between common construction and capability-specific extension APIs.

## 6. Deferred syntax helpers

- **Macros:** ordinary associated functions remain the basic API. Consider syntax expanding to the same constructors only when needed.
- **Broader Mul inputs:** a blanket `Into<C>` implementation overlaps operator multiplication and state action. Extensions, including concrete real-input implementations for standard complex coefficients, remain deferred. Convertible inputs can use `scaled(impl Into<C>)`.

## 7. Criteria for adopting candidates

Compare intent, error recovery, and amount of code before and after changes to examples. Preserve index identity, factor order, ambient spaces, and coefficient precision without unnecessarily restricting custom algebras, non-Clone types, or no_std. Upon adoption, document public names/contracts in designs and remove resolved proposals from this backlog.
