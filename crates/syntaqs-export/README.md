# syntaqs-export

Basis-aware numerical export and interpretation metadata for [syntaqs](../../README.md).

## Responsibilities

- Export state expressions and coordinates as numerical vectors.
- Export operators as matrices by applying them to ordered state-basis elements.
- Retain basis, factor-space, shape, and layout information in `Exported<Data, Metadata>`.
- Convert owned Vec exports to ndarray containers without copying their coefficient data.
- Build sprs output from sparse coordinates without constructing an intermediate dense vector or matrix.

## Output types and features

| Feature | State/coordinate vectors | Operator matrices |
| --- | --- | --- |
| Always available | `Vec<C>` | Flat, row-major `Vec<C>` |
| `ndarray` | `ndarray::Array1<C>` | `ndarray::Array2<C>` |
| `sprs` | `sprs::CsVec<C>` | `sprs::CsMat<C>` in CSC format |

The default feature set is empty. Enable `ndarray` and `sprs` independently or together. The corresponding container crates are re-exported when enabled.

`Export` is the common conversion trait. Use a state basis directly for vector export and `MatrixArgs::<StatePrimitive, _>` for operator matrices. MatrixArgs can supply an explicit ambient space; otherwise a composite operator needs an existing binding. `VectorMetadata` and `MatrixMetadata` describe how to interpret the output.

## Integration

This crate uses the standard library and depends on [syntaqs-core](../syntaqs-core/README.md) for basis enumeration, actions, and coordinates. Export preserves the coefficient type and the selected basis and factor order. Algebra rules and expression evaluation remain in the core and concrete algebra crates.

Coordinates of an operator in an operator basis form a coefficient vector; they are distinct from its matrix acting on a state basis.

See the [crate API overview](src/lib.rs), [export design](../../docs/design/export.md), and [Heisenberg export example](../../examples/src/bin/heisenberg.rs). Git dependency and feature settings are in the [project README](../../README.md).
