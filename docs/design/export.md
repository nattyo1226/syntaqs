# Export to numerical containers

Last updated: 2026-11-02

## 1. Status and responsibilities

`syntaqs-export` provides vectors for states and coordinates, and action matrices for operators. Public types are available at the crate root.

Basis enumeration and coordinates follow [basis design](basis.md); action and space selection follow [composite expressions](composite-expression.md); coefficients and zeros follow [numeric conventions](numerics.md).

`syntaqs-core` handles algebraic basis enumeration, action, and coordinates; `syntaqs-export` handles numerical storage and backend integration. Export neither accesses core's private expression trees nor reimplements evaluation rules. Core gains no backend dependencies and retains `no_std + alloc`. Export is one crate that may use std.

## 2. Output formats and result types

### 2.1. Numerical containers

| Subject | Backend-independent dense output | External numerical containers |
| --- | --- | --- |
| State vector | `Vec<C>` and metadata | `ndarray::Array1<C>`, `sprs::CsVec<C>` |
| Operator matrix | Flat `Vec<C>` and metadata | `ndarray::Array2<C>`, `sprs::CsMat<C>` |

Both dense vectors and matrices store numerical data in `Vec<C>`. Matrix shape (rows and columns) and storage order belong in metadata; there is no dedicated dense-matrix container. Users can connect Vec or slices and metadata to other external crates.

The API exposes Vec output with metadata, direct ndarray output from inputs, and conversion of existing Vec output to ndarray. Direct ndarray output internally shares Vec output and conversion. Sparse output constructs sprs containers from Coordinates without an intermediate dense result.

### 2.2. Exported and metadata

The common `Exported<D, M>` holds numerical data `D` and interpretation metadata `M`. Concrete destinations are specified directly with this type.

- `data()` and `metadata()` borrow each part.
- `into_data()` extracts the owned numerical container; `into_parts()` extracts owned data and metadata.
- `data().as_slice()` accesses a Vec slice.
- To preserve data-length and shape consistency, the initial API provides no mutable reference permitting arbitrary container resizing or reshaping. Extract ownership to modify data.

`VectorMetadata` stores vector length; `MatrixMetadata` stores rows, columns, and storage order. They provide `shape()` and, for dense matrices, `layout()`. Dense construction and conversion validate that Vec length equals the shape's element count.

Metadata exposes the domain, codomain, basis, and factor order used. Bases are borrowed as in Coordinates; composite spaces share `HilbertSpace` handles. Local expressions record factor count and local dimension. Results do not borrow the source expression and add no Clone bound on bases or Clone, Hash, or Ord on indices. Coefficient type `C` represents precision without a duplicated precision label.

Matrices use the same state basis for input and output and the same selected full ambient space as domain and codomain. State type `S` is associated at the type level with expression export and matrix metadata; storing it requires no S value, Clone, or numeric capabilities.

Coordinates itself carries no target primitive type. Its direct export is a coordinate vector associated with a basis and space, without claiming to recover the original state or operator type from Coordinates alone. An operator-basis coordinate vector is not an action matrix on states.

## 3. Public API and ownership

### 3.1. Export trait

The following trait expresses the capability to export to type T:

```rust
pub trait Export<T, Args = ()> {
    type Error;

    fn export(self, args: Args) -> Result<T, Self::Error>;
}
```

`T` is normally a result type including metadata. Infer the destination from the return type or specify it with `Export::<Target, _>::export(source, args)`. Arguments are also a trait type parameter, allowing different basis types for the same destination. Each implementation imposes its own capability bounds.

| Source | Numerical data in the result | Args |
| --- | --- | --- |
| `&LocalState`, `&State` | Vec, ndarray Array1, sprs sparse vector | `&basis` |
| Owned `Coordinates`, `&Coordinates` | Vec, ndarray Array1, sprs sparse vector | `()` |
| `&LocalOperator`, `&Operator` | Vec, ndarray Array2, sprs sparse matrix | `MatrixArgs` |
| Owned Vec output with vector metadata | ndarray Array1 | `()` |
| Owned Vec output with matrix metadata | ndarray Array2 | `()` |

Expression implementations use borrowed types and preserve inputs. Owned Coordinates moves coefficients; borrowed Coordinates clones only necessary coefficients. They use the existing `into_entries()` and basis and space information obtained before conversion.

Converting Vec output to ndarray takes ownership, moves data without changing coefficient type, and retains basis, space, and shape interpretation. Vector or matrix metadata determines Array1 or Array2.

The following illustrates public API usage. See [heisenberg.rs](../../examples/src/bin/heisenberg.rs) for a complete executable example. From the repository root, `cargo run -p examples --bin heisenberg --features export` also compares dense and sparse output.

```rust
// Export to Vec with metadata.
let dense: Exported<Vec<_>, _> = (&psi).export(&basis)?;
let shape = dense.metadata().shape();
let values = dense.data().as_slice();

// Consume an existing Vec export and convert it to ndarray.
let array: Exported<ndarray::Array1<_>, _> = dense.export(())?;

// Direct output shares the same Vec export and conversion internally.
let direct: Exported<ndarray::Array1<_>, _> = (&psi).export(&basis)?;
```

### 3.2. Operator arguments and space selection

`MatrixArgs::<S, _>::new(&basis)` specifies state type S, which `Operator<I, O, C>` itself does not carry. Add `.in_space(&ambient)` when an explicit space is needed.

The configuration type `MatrixArgs<'b, S, B, A = ()>` holds `&'b B`, space argument A, and `PhantomData<fn() -> S>`. `.in_space` returns arguments with `A = &'h HilbertSpace<I>`. Without a space, A is `()`, avoiding unnecessary Index specification or inference. Local-operator implementations accept `A = ()`.

Choose operator output as `Exported<Vec<_>, _>`, `Exported<ndarray::Array2<_>, _>`, or a sparse-matrix result. Composite operators prefer an explicit space, otherwise using an existing binding. Unbound operators without an explicit space return an error; local placements do not infer a full ambient space. Local operators use a local state basis and require no global space.

Space selection and reference validation precede output allocation and column generation, delegating to public `bind`, `act`, and `act_in`. Zero does not bypass invalid-reference checks. Basis enumeration checks counts and indices, but does not automatically prove completeness or linear independence for arbitrary user implementations.

## 4. Conversion paths and numerical contracts

### 4.1. Dense representations

State output places sparse `coordinates` in a Vec. Matrix output enumerates input basis states one by one and obtains each column from action-result coordinates in the state basis. Dense and sparse output share this column generation without retaining all basis states or all columns' Coordinates at once. Full operator-basis expansion and dense intermediate columns are not required.

For a shared orthonormal input/output state basis, entries equal `A_mn=⟨e_m,Ae_n⟩`. For a nonorthonormal basis, entries are expansion coordinates of the action result, rather than those inner-product values.

Dense matrices are row-major: `(row, col)` is at `row * ncols + col`. Both specification and metadata state storage order. Factor basis indices use the existing rightmost-fastest mixed-radix order, separately from matrix memory layout.

ndarray conversion uses `Array1::from_vec` for vectors and `Array2::from_shape_vec` with metadata shape and the Vec for matrices. It avoids another numerical-data copy and retains metadata with the same interpretation. Reference: [ndarray 0.17.2 layout and construction APIs](https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html).

### 4.2. Sparse representations and sprs constructors

Sparse vectors build index and value arrays from Coordinates, using `CsVec::try_new(dim, indices, data)`. Initial sparse-matrix export uses CSC with `usize` indices. Each column's coordinates, ordered by flat index and without duplicates, form `indptr`, `indices`, and `data` for `CsMat::try_new_csc`. Checked APIs are used, without unsafe or private validation-bypassing paths.

This does not imply sprs already accepts `syntaqs-core` Coordinates directly. `syntaqs-export` implementations connect `flat_entries()`, `into_entries()`, and related APIs to sprs constructors. Neither a dense Vec intermediate nor a new custom sparse container is a required path.

sprs also offers coordinate-format `TriMat::from_triplets`, `add_triplet`, `to_csc`, and `to_csr`. The selected column-generation path constructs CSC from sorted, aggregated coordinates without requiring TriMat. Users needing CSR can convert the extracted sprs matrix. Export provides neither format selection nor alternative integer index types.

References: [sprs sparse-vector constructors](https://docs.rs/sprs/0.11.4/sprs/struct.CsVecBase.html), [CSC construction and format conversion](https://docs.rs/sprs/0.11.4/sprs/struct.CsMatBase.html), and [coordinate-format construction](https://docs.rs/sprs/0.11.4/sprs/struct.TriMatBase.html).

Sparse matrices require representable axis dimensions, column-pointer arrays, and nonzero counts, without requiring the dense element count `nrows * ncols` to fit an integer. CSC column pointers and input-basis traversal remain necessary; memory and runtime are not guaranteed to depend only on nonzero count.

### 4.3. Coefficients, precision, and nonorthonormal bases

Coefficients remain C; standard `Complex<f32>` and `Complex<f64>` are verified. Only implementations needing a capability require it; Float, inner products, and operator Product are not uniform requirements.

- Owned Coordinates → sparse vector adds no Clone or Zero coefficient bound.
- Coordinates → dense vector requires Zero to fill absent entries. Borrowed Coordinates also requires Clone for necessary coefficients.
- Expression → coordinates inherits existing API bounds.
- Owned Vec output → ndarray adds no coefficient arithmetic bounds.

Existing coordinates omit only exact zeros. Export adds neither approximate pruning nor special handling of nonfinite values. Each API preserves C; precision changes use explicit component conversion after extracting numerical data.

For nonorthonormal bases, matrix columns still come from action-result expansion coordinates. A coordinate vector's Euclidean inner product is not the Hilbert-space inner product. Numerical normalized-expectation verification uses `c† G A c / (c† G c)` for coordinates c, Gram matrix G, and action matrix A, or reconstructs in an independent orthonormal basis. A new product API for generating Gram matrices is not a prerequisite.

## 5. Errors and allocation

`ExportError` retains existing basis, space, and dimension errors as sources. It additionally distinguishes element-count, byte-count, and compressed-array-length overflow; failure of allocations performed by export; and invalid backend shapes or compressed structures. Checked constructors supply backend errors.

Dense output checks `nrows * ncols` and storable size before allocation; sparse output checks required array lengths. Major Vec allocations use `try_reserve` variants. Recovery from all out-of-memory conditions, including those within core evaluators and backends, is not guaranteed.

Space-construction and algebraic-evaluation contracts remain unchanged. Export validates sizes required by numerical containers at its boundary. Existing sparse-coordinate storage and basis expansion remain usable even when total dimension overflows.

## 6. Features, re-exports, and dependencies

### 6.1. Enabling and accessing APIs

`syntaqs-export` has no default features; Export, Vec output, and metadata are always available. `ndarray` and `sprs` features enable their APIs and optional dependencies. The ndarray feature enables both direct expression / Coordinates output and conversion from owned Vec output.

Users select `syntaqs-export` features directly. Backend types are re-exported as `syntaqs_export::ndarray` and `syntaqs_export::sprs` under their respective features. The `examples` package's optional `export` feature enables both backends.

sprs depends on ndarray, so enabling sprs alone still includes the ndarray dependency. Feature separation selects the public conversion APIs. sprs defaults alga and multi_thread are disabled; ndarray uses std only. Initial integration includes no BLAS, rayon, or persistence features.

### 6.2. Dependency specifications and MSRV

| Dependency | Selected specification |
| --- | --- |
| sprs | `version = "=0.11.4", default-features = false` |
| ndarray | `version = "0.17.2"` (std only) |

MSRV remains 1.85. The published sprs 0.11.4 manifest specifies Rust 1.85.0 and ndarray `>=0.15.0, <0.18`; ndarray 0.17.2 specifies Rust 1.64. sprs 0.11.5 specifies Rust 1.88.0, conflicting with its README's Rust 1.64 statement. The manifest is authoritative, so sprs is pinned exactly to 0.11.4.

References: [sprs 0.11.4 manifest](https://docs.rs/crate/sprs/0.11.4/source/Cargo.toml), [sprs 0.11.5 manifest](https://docs.rs/crate/sprs/0.11.5/source/Cargo.toml.orig), [ndarray 0.17.2 manifest](https://github.com/rust-ndarray/ndarray/blob/0.17.2/Cargo.toml), and [sprs features](https://docs.rs/crate/sprs/0.11.4/features).

Selected specifications are in the [workspace manifest](../../Cargo.toml) and [export manifest](../../crates/syntaqs-export/Cargo.toml). Environment and backend checks are described in [verification](verification.md).

## 7. Verification targets and tests

Numerical, type, and memory contracts have the following verification targets:

- Compare complex-coefficient states, products, actions, adjoints, and trace across algebraic, coordinate, dense, and sparse paths, using independently defined local spin matrices and two-factor 4×4 matrices. Also compare paths from operator-basis-expanded expressions.
- For states and operators, compare direct ndarray output with conversion from Vec output: entries, shape, storage order, basis, space, factor order, and precision. Asymmetric matrices with imaginary entries detect flattening and transpose mistakes.
- Verify independent reference calculations using only Vec slices and metadata, and preservation of the data pointer during owned Vec → ndarray conversion.
- Check three-dimensional algebras, nonorthonormal-basis coordinates and reconstruction, and Gram-matrix inner products and numerical expectations. Shared conversion has no spin-specific branches.
- Cover empty, single-factor, nested, zero, identity, and spectator cases; explicit-space binding overrides; unbound operators; invalid references and bases; and overflow. Zero factors follow existing unit-space conventions.
- Use open and periodic Heisenberg / Neel systems primarily of lengths 0–4, comparing analytic expectations, independent matrices, and dense and sparse calculations. Maintain an executable end-to-end example.
- Follow [numeric conventions](numerics.md), initially using atol and rtol of `2e-5` each for f32 and `1e-12` each for f64. Record problem size and error justification when adjusting them.
- Verify rejection of implicit precision changes and explicit precision conversion of extracted data. Check that storage and moves add no unnecessary coefficient, Index, or primitive bounds.
- Verify sparse vectors avoid dense allocation in large spaces with few nonzeros. Targeted tests and implementation review check column-by-column sparse matrices without dense conversion or retaining all columns.
- Check no-backend, ndarray-only, sprs-only, and both-backend configurations separately. [Verification](verification.md) specifies test and CI commands.

## 8. Types and module structure

Public types are at the `syntaqs_export` root. Alongside basis, space, and shape, `VectorMetadata<'b, B, H = (), P = ()>` records the state primitive type in P for state export, and uses P = `()` for direct Coordinates export. `MatrixMetadata<'b, S, B, H = ()>` records state type S and the selected space. H is `()` for local expressions and `HilbertSpace<I>` for composite expressions.

Specify destinations as `Exported<Vec<C>, M>`, `Exported<ndarray::Array1<C>, M>`, `Exported<ndarray::Array2<C>, M>`, `Exported<sprs::CsVec<C>, M>`, or `Exported<sprs::CsMat<C>, M>`. There are no aliases for particular combinations. Local variables can infer metadata with `_`. Matrix state type is specified through `MatrixArgs::<S, _>::new(&basis)`. Rust API documentation provides complete type parameters and methods.

Implementation is split into private output, metadata, args, coordinates, vector, matrix, error, and backend modules. Internal storage shares dense and sparse column generation; direct and staged ndarray conversion share Vec construction and array conversion. Export uses core's public API rather than private nodes.

Separately from general map representation coefficients, current numerical matrix output uses the same state basis and full ambient space for both input and output.

## 9. Graded inputs

The shared `Export` handles ordinary and graded inputs. Core action and state coordinates handle signs column by column; Vec / ndarray / sprs storage has no graded-specific logic. The HilbertSpace retained in metadata preserves kind and factor order. Composite state and operator output requires O's `HomogeneousParity + ParityOperator`; local output does not. Vector output also uses shared action evaluation, with common shape and storage order. See [shared evaluation](composite-expression.md#112-shared-apis-and-minimal-capabilities) for capabilities and [verification](verification.md) for test configurations.
