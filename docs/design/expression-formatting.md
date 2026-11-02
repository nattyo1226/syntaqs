# Expression-tree formatting

Last updated: 2026-11-02

This document defines Display and Debug for local and composite state and operator expressions, and formatting of standard indices. See [expression_formatting](../../examples/src/bin/expression_formatting.rs) for Display examples of states and a four-Site Hamiltonian, and the rustdoc for `syntaqs_core::expr` and `syntaqs_basic::Tag` for public APIs.

## 1. Scope and type bounds

| Entry point | Purpose |
| --- | --- |
| `{}` | Mathematical notation without expression-inserted line breaks |
| `{:#}` | The same formula with structural line breaks |
| `{:?}` / `{:#?}` | Stored structure through `derive(Debug)` |

| Type | Display bounds | Debug bounds |
| --- | --- | --- |
| `LocalState<S, O, C>` | `S, O, C: Display` | `S, O, C: Debug` |
| `LocalOperator<O, C>` | `O, C: Display` | `O, C: Debug` |
| `State<I, S, O, C>` | `I, S, O, C: Display` | `I, S, O, C: Debug` |
| `Operator<I, O, C>` | `I, O, C: Display` | `I, O, C: Debug` |
| `HilbertSpace<I>` | Not provided | `I: Debug` |

- Formatting bounds apply only to trait implementations, not construction or evaluation. Formatting requires no additional Clone, Eq, Hash, Ord, or numeric capabilities.
- Formatting borrows the retained expression without evaluation, expansion, simplification, coefficient arithmetic, or term aggregation. Zero and unit coefficients, duplicate terms, and double adjoints remain visible.
- Ordinary and graded expressions follow stored order without reordering, sign calculation, or parity-string insertion. Unbound operators can also be displayed.
- Display shows only the formula. Debug exposes spaces, ambient spaces, and original grouping. Display strings are neither identity checks nor serialization formats.
- Debug is also derived for internal shared syntax, nodes, retained-action types, and HilbertSpace. It includes bookkeeping fields and delegates to leaf Debug. Internal types need not be public, and Debug strings have no compatibility guarantee.
- Debug fallback for types without Display, arbitrary label callbacks, and dedicated LaTeX or ASCII modes are outside this scope.

## 2. Index

### 2.1. Standard concrete types

[Index](index.md) defines construction, identity, and type capabilities of standard Id128 and Tag. Standalone Display outputs the full UUID or name; Debug also retains the full identifier. Truncation below applies only inside expression Display.

### 2.2. Eight-character limit within expression trees

Only expression-tree Display limits an arbitrary Index's Display output to its first eight Rust `char` values. There is no ellipsis, collision detection, or automatic extension. Standalone Index Display and expression Debug are not truncated.

- An internal `core::fmt::Write` adapter counts across multiple writes. Arbitrary Display implementations may ignore precision, so `{:.8}` alone is insufficient.
- UTF-8 sequences are not split. Grapheme counts and terminal column widths are not considered; label line breaks and delimiters are not corrected.
- No complete intermediate String or lookup table is built. Characters after the limit are accepted and discarded. Reaching the limit is not `fmt::Error`; actual output errors propagate.
- The Index's own Display may still run to completion, so its internal runtime and allocation cannot be bounded to a constant.
- There is no naming from tensor order or lookup of an Index's position in a space.

## 3. Display syntax

`A, B, C` are illustrative operator labels and `psi` is a state label. Primitives and coefficients delegate to their own Display without parsing its output. State primitives output only labels; expressions add ket notation and indices.

| Subject | Display |
| --- | --- |
| Operator primitive | The primitive's own Display |
| Unplaced / placed state primitive | `\|up>` / `\|up[a]>` |
| Empty sum / composite-operator Identity / state Unit on an empty tensor | `0` / `I` / `unit` |
| Scaling / sum / operator product | `(2+3i) * A` / `A + B + C` / `A * B` |
| Action / adjoint | `A psi` / `(A)†` |
| Local-operator placement | `Sx[a]` / `(Sx + Sy)[a]` |
| Simple product state | `\|up[a],down[b],up[c]>` |
| Tensor of composite factors | `(\|up[a]> + \|down[a]>) ⊗ \|up[b]>` |

### 3.1. Coefficients, signs, and parentheses

Coefficients are always parenthesized; `A + (-1+0i) * B` is not converted to subtraction. `Operator::scalar(c)` displays `(c) * I`. Local identity delegates to its retained primitive; formatting generates no identity value.

Precedence from weakest to strongest is sum, tensor written with `⊗`, action, product / scaling, then adjoint / placement / primitive. A combined ket is atomic. Children are parenthesized as follows:

| Parent | Parenthesized children |
| --- | --- |
| Sum | Sums |
| Product | Sums and scaling; also products on the right |
| Scaling | Sums, products, scaling, actions, and tensors written with `⊗` |
| Tensor written with `⊗` | Sums, scaling, actions, and tensors written with `⊗` |
| Operator side of action | Sums, products, and scaling |
| State side of action | Sums, scaling, actions, and tensors written with `⊗` |
| Adjoint | Always the entire operand |
| Local-operator placement | Entire sums, scaling, products, and adjoints; empty sum `0` is atomic and displays as `0[a]` |

Examples: `A * B * C`, `A * (B * C)`, `A + (B + C)`, `(2) * ((3) * A)`, `A * ((2) * B)`, `((A)†)†`, `(A * B) |up[a]>`, and `A (B |up[a]>)`.

### 3.2. State placement and tensors

A local state's Index reaches each primitive ket even inside sums, scaling, and actions. Operators inside local actions receive the same Index using operator-placement syntax. Formatting creates no new nodes and performs no distribution.

```text
|up[a]> + |down[a]>
(2+3i) * (|up[a]> + |down[a]>)
Sx[a] |up[a]>
(Sx + Sy)[a] |up[a]>
(|up[a]> + |down[a]>) ⊗ |up[b]>
```

- A simple product state is a subtree containing only Tensor nodes and Local nodes whose local expressions are Primitive. It combines `label[Index]` leaves in order within one ket. Single factors use the same syntax; tensor grouping is omitted visually.
- A whole subtree containing sums, scaling, actions, or explicit Unit is not combined and uses `⊗`. Simple product-state children may still be combined. Sums are not expanded and coefficients are not moved.
- Empty sums remain `0` after placement; Debug shows their spaces. Formatting neither strips ket symbols from primitive strings nor introduces an additional required trait.

### 3.3. Multiline display

`{:#}` uses four spaces per nesting level. Sums and tensors written with `⊗` put each operand on a separate line, with `+ ` or `⊗ ` preceding subsequent operands. Operators occupy two positions to the left of each operand's starting column, aligning first and later operands. Top-level sums and `⊗` tensors reserve two initial spaces. Continuation indentation for parentheses and kets is based on the operand's start excluding the operator. A ket begins with `| `, breaks after commas, and aligns factor labels in one column. Even after a coefficient or action operator, alignment uses the actual column immediately after the emitted `| `. It closes with ` >` on the last factor's line; a single factor is one line such as `| up[a] >`. Columns count Rust chars, not terminal display width.

Parenthesized composite operands indent their contents. Simple parts of products and actions avoid unnecessary line breaks. Parenthesized sums and `⊗` tensors break after the opening and before the closing parenthesis; other parentheses remain inline.

```text
(
    A
  + B
  + C
) | up[a],
    down[b],
    up[c] >
```

Coefficients, indices, and order match ordinary Display. alternate, width, and precision are not forwarded wholesale to leaves. width is not a wrap width; precision does not change coefficient precision or truncate expressions. There is no terminal-width detection or automatic wrapping.

## 4. Builtin primitives

Each algebra crate's primitives provide Display and derive(Debug). Standalone labels are:

| Type | Display |
| --- | --- |
| SpinHalfOperator | `I, Sx, Sy, Sz, Sp, Sm` |
| SpinOneOperator | `I, Sx, Sy, Sz, Sp, Sm, Dxy, Q0, Qxy, Qyz, Qzx, Q1p, Q1m, Q2p, Q2m` |
| FermionOperator | I → `I`, Annihilate → `a`, Create → `a†`, Number → `n`, Mx/My/Mz → `Mx`/`My`/`Mz` |
| SpinHalfState | Up → `up`, Down → `down` |
| SpinOneState | Plus → `+1`, Zero → `0`, Minus → `-1` |
| FermionState | Empty → `0`, Occupied → `1` |

SpinOneState::Zero and FermionState::Empty are not zero vectors. Ket `|0>` differs from empty sum `0`. Normalization and physical coupling constants are unchanged.

## 5. Implementation and resource handling

Formatting is contained in [syntaqs-core/src/expr/format.rs](../../crates/syntaqs-core/src/expr/format.rs). A borrowed work stack spans local / composite and state / operator expressions, iteratively handling parentheses, line breaks, and placement. Iterator continuation items handle sum and tensor children without pushing all children at once. No leaves or coefficients are copied, no whole-expression intermediate String is built, and evaluation traversal is not used. Output errors stop formatting immediately.

An iterative postorder traversal classifies tensors that can form a combined ket. A `BTreeMap` local to the display call records both positive and negative results, keyed by shared syntax-node addresses. Keys are neither dereferenced nor used for Index identity; I needs no Eq, Hash, or Ord. With U unique nodes and E edges, classification costs O((U + E) log(U + 1)) time and O(U) records. This avoids quadratic rescanning of deep mixed tensors for each ancestor.

The display work stack and ket leaf traversal use space proportional to depth. Shared subexpressions appear at each occurrence, so output can exceed the shared-node count. No automatic truncation applies beyond indices; indentation in deep multiline output also contributes to output length. Core retains `no_std + alloc` and forbids unsafe, with no additional formatting dependencies.

derive(Debug) is recursive and is not guaranteed to succeed on extremely deep trees. This differs from the depth robustness of Display and existing evaluation and destruction.

## 6. Verification

| Concern | Checks |
| --- | --- |
| Basic syntax | Every node of all four expression types; zero, I, unit; zero, unit, and negative coefficients; duplicate terms; double adjoints; parentheses; noncommutative product and action order |
| States and placement | Single, product, and nested tensors; propagation of indices into local sums, scaling, and actions; no expansion of complex factors |
| Multiline output | Sum, ket, and parenthesis indentation; empty and single inputs; matching coefficients, indices, and order with ordinary Display |
| Index | Lengths 0, 7, 8, and 9 or more; Unicode; multiple writes; Display ignoring precision; complete standalone and Debug output |
| Standard concrete types | Tag equality, hash, Ord, Clone, and duplicate-placement rejection; Id128 Copy, 16-byte size, UUID formatting and generation |
| Primitives | All variant labels and ket construction; ket 0 distinguished from the zero state |
| Spaces and capabilities | Unbound, bound, spectator, ordinary / graded cases; non-Clone leaves; display-only types; existing construction without formatting capabilities; no evaluation-rule calls, preserving inputs and sharing |
| Depth | Display at depth 20,000 on a small stack; no quadratic deep-tensor classification. Debug checks structure, space, and full indices at ordinary depths |
| Output failure | Propagation of actual `fmt::Error` without formatting subsequent leaves |

Regression tests for syntax, bounds, errors, and depth are in [core formatting.rs](../../crates/syntaqs-core/tests/formatting.rs); standard indices are in [basic indices.rs](../../crates/syntaqs-basic/tests/indices.rs); primitive labels are in each builtin crate's `tests/formatting.rs`. Tests include display and destruction of local / composite syntax at depth 20,000 and classification of shared subexpressions. See [verification](verification.md) for methods and limits.
