# syntaqs architecture overview

Updated: 2026-11-09

## 1. Overall structure

syntaqs connects local algebra rules to shared expressions, spaces, and evaluation, then combines that core with geometry-based model construction and numerical output. This document describes the structure currently provided. See [requirements](requirements.md) for purpose and functional requirements and the [design guide](README.md#2-feature-details) for feature contracts.

## 2. Crates and responsibilities

The workspace contains nine libraries and three packages for examples,
integration tests, and shared internal BLAS/LAPACK support.

| Crate / package | Provides |
| --- | --- |
| `syntaqs-core` | Local algebra traits, coefficient capabilities, finite linear combinations, local/composite expressions, Hilbert spaces, construction, evaluation, bases, and coordinates. |
| `syntaqs-basic` | Standard UUID and named indices. |
| `syntaqs-geometry` | Chains, boundaries, site relations, and borrowing connections to caller-owned data and space sequences. |
| `syntaqs-normalize` | Extension traits for norms, state normalization, and normalized expectations. |
| `syntaqs-export` | Vec, ndarray, and sprs output with interpretation metadata. |
| `syntaqs-lie` | Operator families, real HS projection, Lie brackets and closure diagnostics, retained adjoint/Killing representations, real family transformations, and dense numerical foundations; center and ideal decomposition remain pending. |
| `syntaqs-spin-half` | Spin-1/2 primitives and bases, XYZ/Heisenberg models, and Neel construction. |
| `syntaqs-spin-one` | Spin-1 primitives and bases, and XYZ/Heisenberg/BLBQ construction. |
| `syntaqs-fermion` | Single-mode fermion primitives and bases, and standard/symmetric Hubbard construction. |
| [examples](../examples/README.md) | Runnable examples using the individual crates directly. |
| [syntaqs-tests](../tests/README.md) | Shared custom-algebra fixtures and public-API integration tests. |
| [blas-provider](../support/blas-provider/README.md) | Internal system OpenBLAS selection shared by development, examples, and verification; kept under `support/` and not published. |

## 3. Dependency direction

The core does not depend on concrete algebras, basic, geometry, normalize, export, or lie. Local rules and bases connect through public traits; concrete algebras do not provide separate expression trees or evaluators.

Geometry, normalize, export, lie, and the three concrete algebras depend on core. Each algebra's model construction uses geometry. Basic depends on core only for tests and doctests; UUIDs and randomness stay in basic. Production libraries do not depend on examples, syntaqs-tests, or the development BLAS provider.

Core, geometry, spin-half, and fermion use `no_std + alloc`; basic, normalize, export, lie, and spin-one use std. Export has no default features, always supports Vec, and independently enables ndarray and sprs. Lie enables ndarray BLAS and retains native linkage through provider-neutral blas-src; applications select a source supplying BLAS and LAPACK.

## 4. Main data and operations

| Object | Role and relationships |
| --- | --- |
| Primitives and coefficients | Types and individual capability implementations define local states, operators, and rules. Numeric bounds belong to each operation. |
| `HilbertSpace<I>` | Retains factor identity, order, grouping, and ordinary/graded composition; it does not store local algebra, dimension, or basis. |
| Local/composite expressions | Share immutable nodes and delay sums, products, actions, and adjoints. States have a definite space; operators have an optional ambient-space binding. |
| `ExpressionFactory` / `QuantumSystem` | Separate construction from primitive/coefficient type information. `SpinHalf<C>` and related names are factory aliases. |
| Evaluation | Uses the selected space and local rules. Ordinary and graded calculations share an API; core handles signs from order and parity. |
| `Basis` / `Coordinates` | Select a basis per operation for ordinary-expression expansion, sparse coordinates, and lazy enumeration. Expression nodes do not retain basis metadata. |
| `Exported` | Associates numerical data with basis, space, shape, and layout. Action-matrix columns come from acting on state-basis elements. |
| `Chain` / `SiteMapper` | Borrow geometry and caller-owned site-ordered data. Callers issue indices and construct, retain, and order the spaces. |

Typically, callers construct indices and spaces, choose primitive and coefficient types, then place and compose local expressions. Actions, inner products, expectations, and traces can be evaluated without matrices. A basis is chosen when coordinates or numerical matrices are needed. Model helpers also return ordinary state/operator expressions.

## 5. Module structure

Core is divided into `ops`, `coefficient`, `space`, `system`, `expr`, `basis`, and private `evaluate` modules. Shared syntax parts live in `expr::parts` and internal traversal in `expr::traversal`. Public expression handles and major traits are available at the crate root; node enums and shared representations are private.

Geometry separates geometry, boundaries, relations, groups, and connections. Normalize separates norms, expectations, and errors. Export separates arguments, results, metadata, coordinate/vector/matrix conversion, and backends. Concrete algebras separate primitives, coefficient helpers, bases, and models. Feature designs specify detailed types, capabilities, errors, and processing.

See [verification](design/verification.md) for checks, [future work](future-work.md) for unimplemented extensions, and the [documentation guide](README.md) for document responsibilities and maintenance conventions.
