# Model construction on chains

Updated: 2026-11-02

## 1. Scope and responsibilities

Each concrete algebra's models module constructs ordinary state/operator expressions from a Chain and caller-owned site-ordered spaces. Geometry enumerates edges/placements; models assign operators, coefficients, and couplings, without separate model expression types or evaluators.

Callers issue indices. Construction validates length, duplicates, and model-specific site shape while delaying local evaluation and matrices. See [geometry](geometry.md) for connections and [local algebras](local-structure.md) for primitive normalization.

## 2. SpinHalf model construction

Public entry points are associated functions on `syntaqs_spin_half::models::SpinHalfModels<SpinHalf<C>>`. Below, `type Models = SpinHalfModels<Spin>`. UtilityError is re-exported at the crate root.

| Call | Successful result |
| --- | --- |
| `Models::xyz(&chain, &factors, jx, jy, jz)` | `OperatorOf<Spin, I>` |
| `Models::heisenberg(&chain, &factors, coupling)` | `OperatorOf<Spin, I>` |
| `Models::neel(&chain, &factors, first)` | `StateOf<Spin, I>` |

All return Result and validate sequence length and all leaf duplicates even without edges. Length errors are `UtilityError::Mapping(MappingError::SiteCountMismatch { expected, actual })`, duplicates `UtilityError::Space(SpaceError::DuplicateIndex)`, and placement/binding errors UtilityError::Expression. Original errors remain sources; indices need only Eq.

XYZ constructs `−(1/4)Σ_(i,j) Σ_(a∈Li,b∈Lj) (Jx Sx_a Sx_b + Jy Sy_a Sy_b + Jz Sz_a Sz_b)`, using NearestNeighbors and all leaves_cartesian pairs between sites. Li is the leaf set of site i. There are no within-site terms. Heisenberg sets all couplings to J; positive J is ferromagnetic. No positional zip is implicitly chosen.

XYZ/Heisenberg bind to the tensor of input site handles in sequence order, sharing local grouping and retaining sites absent from interactions. They do not preserve an independently existing ambient root, upper grouping, or different factor order.

Neel alternates by site position and first, assigning the same orientation to every leaf of a site. Empty sites still count toward parity. Factor order follows site order and local leaf order; odd periodic lengths are allowed. For `[(a,b),c]` and first=Up, the state is `Up_a ⊗ Up_b ⊗ Down_c`.

Zero sites return a zero Hamiltonian bound to the unit space and a unit state. All-empty sites behave similarly, retaining empty-site grouping in the Hamiltonian's space.

Bounds belong to each function: xyz requires `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>`, and heisenberg additionally `R: Clone`. Neel needs no numeric capabilities or Clone on C. Model-specific methods are not added to QuantumSystem.

## 3. Fermionic Hubbard models

Standard and symmetric Hubbard models differ in their interactions, with APIs and placement inputs following the spin-half pattern.

### 3.1. Hamiltonian conventions

Use uniform real t/U, nearest-neighbor chain edges, and spin-conserving hopping. Neither model includes a chemical-potential term.

```math
K(t)=-t\sum_{\langle i,j\rangle,\sigma}
  \left(f_{i\sigma}^\dagger f_{j\sigma}
       +f_{j\sigma}^\dagger f_{i\sigma}\right),\qquad
N=\sum_{i,\sigma}n_{i\sigma},
```

```math
H_{\mathrm{Hubbard}}(t,U)
 =K(t)+U\sum_i n_{i\uparrow}n_{i\downarrow},
```

```math
H_{\mathrm{symmetric}}(t,U)
 =K(t)+U\sum_i\left(n_{i\uparrow}-\frac I2\right)
                  \left(n_{i\downarrow}-\frac I2\right).
```

The standard form follows [ALPS Hubbard conventions](https://alps.comp-phys.org/documentation/models/hubbard/); the symmetric form follows the hopping/interaction signs in [TRIQS/TPRF](https://triqs.github.io/tprf/3.1.x/user_guide/Linearized%20Eliashberg%20equation%20on%20the%20attractive%20Hubbard%20model.html). There is no chemical-potential argument.

The symmetric form preserves hopping and retains the interaction's constant shift. For L sites:

```math
H_{\mathrm{symmetric}}(t,U)
 =H_{\mathrm{Hubbard}}(t,U)
   -\frac U2 N+\frac{UL}{4}I.
```

Since `n−I/2=−Mz/2`, the symmetric local interaction is also `(U/4) Mz↑Mz↓`. Verify equivalence of both forms.

### 3.2. Public construction and placement

Construct in models.rs through `models::FermionModels<Fermion<C>>`, without model-specific QuantumSystem methods.

```rust,ignore
FermionModels::<Fermion<C>>::hubbard(&chain, &sites, t, u)
FermionModels::<Fermion<C>>::symmetric_hubbard(&chain, &sites, t, u)
// Both return Result<OperatorOf<Fermion<C>, I>, UtilityError>.
```

- Each `sites: &[HilbertSpace<I>]` entry is one physical site with exactly two graded leaves in `(up,down)` order. Callers issue indices; constructors do not reissue them.
- Build the full graded space in site order and up/down order, sharing input site handles, and bind the result to it.
- Hopping connects only equal spins on adjacent sites, unlike spin-half's all-leaf Cartesian pairs. Add each direction once per edge; core's grading supplies signs.
- Follow Chain boundaries: no edges for zero/one site and one edge for two periodic sites, without duplicate hopping.
- Validate site count, two-leaf graded shape, and all mode duplicates even for zero couplings. Invalid sites produce `UtilityError::InvalidSite { site }`; Mapping/Space/Expression retain underlying errors.
- Zero sites give zero bound to the graded unit; one site retains interaction without hopping.
- Standard Hubbard requires `C: Clone + From<R> + Neg<Output=C>`; symmetric additionally One, Add, and Div. Both t/U use one R. Generate 1/4 in C for `(U/4) Mz↑Mz↓`, without imaginary unit, local Product/Act, or Float. Parameters must represent real values and fractions mathematical division.

See [fermion rules](local-structure.md#8-fermion-local-algebra) for operators, Majorana normalization, bases, and spinful phase. [models.rs](../../crates/syntaqs-fermion/src/models.rs) constructs the quarter-scaled Mz product.

### 3.3. Verification

- Check two-mode CAR, annihilation signs from double occupation, and global mode order.
- In one site's display order `[empty,up,down,double]`, check standard diagonal `[0,0,0,U]` and symmetric `[U/4,−U/4,−U/4,U/4]`, explicitly mapping enumeration order.
- Compare both two-site/four-mode/16-state matrices with independent Fock-space signs, without core evaluation/export. Check Hermiticity for real parameters, total/per-spin number conservation, and t=0/U=0 limits.
- Verify their difference `−(U/2)N+(UL/4)I` and quarter-scaled Mz construction including constants.
- Cover zero/one/two/larger open/periodic chains, wrong site counts/mode counts, ordinary sites, duplicate indices, and non-Clone indices.
- Compare Complex<f32>/Complex<f64> and Vec/ndarray/sprs in the same basis order; ensure construction bounds do not acquire evaluation capabilities.

[Model tests](../../crates/syntaqs-fermion/tests/models.rs) use independent Fock-space references, checking hopping multiplicity, spin preservation, and symmetric constants. See [verification](verification.md) for commands and limits.

## 4. SpinOne model construction

`syntaqs_spin_one::models::SpinOneModels<SpinOne<C>>` provides XYZ, Heisenberg, and BLBQ associated functions. [Local spin-one conventions](local-structure.md#9-spin-one-local-algebra) define primitives/bases; implementation is in [models.rs](../../crates/syntaqs-spin-one/src/models.rs).

### 4.1. Hamiltonian conventions

With hbar=1, first consider one spin per site. Count each nearest-neighbor edge once. Since `S=√(3/2) J_physical`, factors 2/3 and 4/9 preserve physical coupling meanings:

```math
H_{\mathrm{XYZ}}=-\frac23\sum_{\langle ij\rangle}
\left(J_xS_i^xS_j^x+J_yS_i^yS_j^y+J_zS_i^zS_j^z\right),
```

```math
H_{\mathrm{Heisenberg}}=-\frac{2J}{3}\sum_{\langle ij\rangle}\mathbf S_i\cdot\mathbf S_j,
```

```math
H_{\mathrm{BLBQ}}=-\sum_{\langle ij\rangle}
\left[\frac{2J}{3}\,\mathbf S_i\cdot\mathbf S_j
+\frac{4K}{9}\,(\mathbf S_i\cdot\mathbf S_j)^2\right].
```

Signs follow SpinHalf: positive bilinear J is ferromagnetic and negative J antiferromagnetic. heisenberg(j) equals xyz(j,j,j); blbq(j,0) equals heisenberg(j). There is no single-ion anisotropy, magnetic field, or additional constant shift; retain the stated energy origin.

BLBQ accepts independent bilinear J and biquadratic K. The angular convention `H=sum[cos(theta) A_ij+sin(theta) A_ij²]` with `A_ij=J_physical,i·J_physical,j=(2/3)S_i·S_j` corresponds to `blbq(-cos(theta),-sin(theta))`. There is no angle-specific entry point. Callers supply real-valued couplings for Hermitian Hamiltonians, without a type-level or runtime reality check.

### 4.2. Public construction and spaces

| Entry point | Successful result |
| --- | --- |
| `SpinOneModels::<SpinOne<C>>::xyz(&chain, &sites, jx, jy, jz)` | `OperatorOf<SpinOne<C>,I>` |
| `SpinOneModels::<SpinOne<C>>::heisenberg(&chain, &sites, j)` | `OperatorOf<SpinOne<C>,I>` |
| `SpinOneModels::<SpinOne<C>>::blbq(&chain, &sites, j, k)` | `OperatorOf<SpinOne<C>,I>` |

All take &Chain and site spaces and return `Result<_,syntaqs_spin_one::UtilityError>`. Site, pairing, and binding contracts follow SpinHalf XYZ/Heisenberg:

- One input space is one site; couple every leaf pair across adjacent sites. BLBQ squares each pair's `A_ab=S_a·S_b` separately, without within-site terms or cross terms between different leaf pairs.
- Check count and global index uniqueness independently of edges/couplings, requiring only Eq on I, not Clone/Hash/Ord.
- Bind to site handles tensored in input order, retaining local grouping, leaves, and spectators; do not reconstruct another ambient space's upper grouping/order.
- Empty sites retain positions without reconnecting neighbors. Zero sites give zero on the unit; one/all-empty sites give zero bound to input space structure.
- Follow [Chain boundaries](geometry.md): no edges for zero/one site, one for two sites with either boundary.

Errors are Mapping(MappingError), Space(SpaceError), and Expression(ExpressionError), preserving sources. Count mismatch uses SiteCountMismatch and duplicate indices DuplicateIndex. UtilityError is re-exported at the crate root.

### 4.3. Delayed construction and coefficient capabilities

XYZ composes placed Cartesian products/sums; Heisenberg is its isotropic case. BLBQ builds `A_ab=Sx_a Sx_b+Sy_a Sy_b+Sz_a Sz_b` once per pair and shares it in `(&A_ab * &A_ab)`. Apply J/K with 2/3 and 4/9 afterward, without squaring J's magnitude or sign.

| Function | Construction bounds |
| --- | --- |
| `xyz<I: Eq,R>` | `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>` |
| `heisenberg<I: Eq,R: Clone>` | `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>` |
| `blbq<I: Eq,R>` | `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>` |

Construction performs rational rescaling in C without local evaluation, expansion, matrices, backends, Float, Zero, or conjugation. Evaluation/trace/export retain their own core/primitive bounds. QuantumSystem gets no model-specific methods.

### 4.4. Verification and tests

[models.rs](../../crates/syntaqs-spin-one/tests/models.rs) builds references from directly written 3×3 spin matrices without using model construction or Chain enumeration.

- Compare every entry, Hermiticity, and trace for both precisions, zero to three sites, and both boundaries; cover anisotropy, positive/negative J/K, zero couplings, pure biquadratic terms, and isotropic/bilinear limits.
- For total spin 0/1/2, verify physical bilinear eigenvalues −2/−1/1 and BLBQ energies `2J-4K`, `J-K`, and `-J-K` using independent states and action residuals.
- Cover all pairs and pairwise squares at multi-leaf sites, empty/zero/one sites, unique two-site periodic edges, nonconsecutive index order, shared grouping, and spectators.
- Reject count mismatches and duplicates even without edges/nonzero couplings; check error sources and construction with minimal rational capabilities, non-Clone couplings, and non-Clone indices.

## 5. SpinHalf verification and end-to-end examples

Cover all cross-site leaf pairs, per-site orientation, empty-site parity and gaps, empty/zero sites, duplicates/count mismatches, non-Clone indices, deep grouping, and minimal coefficients. Tests live in [systems.rs](../../crates/syntaqs-spin-half/tests/systems.rs) and [chain.rs](../../examples/tests/chain.rs).

For real g, Heisenberg is `H=-(g/4)Σ S_i·S_j`; an open Neel chain of length L≥1 has expectation `g(L−1)/4`, and zero sites give zero. Compare independent Pauli matrices, algebraic evaluation, and dense/sparse output at both precisions.

See [heisenberg.rs](../../examples/src/bin/heisenberg.rs), [hubbard.rs](../../examples/src/bin/hubbard.rs), and [spin_one.rs](../../examples/src/bin/spin_one.rs). The spin-one state `|+,−>` has expectation 1 for isotropic XYZ/Heisenberg at J=1 and 0.5 for BLBQ at J=1, K=0.25.
