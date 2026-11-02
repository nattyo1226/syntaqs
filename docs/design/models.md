# Chain 上のモデル構築

更新日: 2026-11-02

## 1. 対象と責任

各具体的代数 crate の `models` module は、Chain と利用側が保持する Site 順の空間列から一般の状態・演算子式を構築する。幾何は辺や配置を列挙し、モデルが演算子・係数・結合規約を割り当てる。独自のモデル式型や evaluator は持たない。

Index は利用側が発行し、構築時には列長・因子重複・モデル固有の Site 形状を検証する。局所規則の評価と行列化は遅延する。幾何と接続の API は[geometry](geometry.md)、primitive の規格化は[local-structure](local-structure.md)に定める。

## 2. SpinHalf のモデル構築

モデル構築の公開入口は `syntaqs_spin_half::models::SpinHalfModels<SpinHalf<C>>` の関連関数である。以下では `type Models = SpinHalfModels<Spin>` とする。エラー型 `UtilityError` を crate root から公開する。

| 呼び出し | 成功時の戻り値 |
| --- | --- |
| `Models::xyz(&chain, &factors, jx, jy, jz)` | `OperatorOf<Spin, I>` |
| `Models::heisenberg(&chain, &factors, coupling)` | `OperatorOf<Spin, I>` |
| `Models::neel(&chain, &factors, first)` | `StateOf<Spin, I>` |

全関数は Result を返し、列長と全葉の Index 重複を、辺がなくても検証する。列長の不一致は `UtilityError::Mapping(MappingError::SiteCountMismatch { expected, actual })`、重複は `UtilityError::Space(SpaceError::DuplicateIndex)`、配置・束縛エラーは `UtilityError::Expression` とし、元エラーを source として保持する。Index には Eq のみを要求する。

XYZ は `−(1/4)Σ_(i,j) Σ_(a∈Li,b∈Lj) (Jx Sx_a Sx_b + Jy Sy_a Sy_b + Jz Sz_a Sz_b)` を構築する。Li は Site i の葉集合であり、`NearestNeighbors` と `leaves_cartesian()` の全組み合わせを使う。Site 内の葉間結合は追加しない。Heisenberg は三結合を J に揃え、正の J を強磁性的とする。標準モデルは位置対応を暗黙に選ばない。

XYZ・Heisenberg の全作用空間は、入力の局所空間ハンドルを共有し、列順に tensor した空間である。局所 grouping と相互作用に現れない Site も保持し、返す演算子をその空間に bind する。別途存在する全空間の root・上位 grouping・異なる因子順序は保存しない。

Neel は Site 位置と first から向きを交互に決め、その Site の全葉に同じ向きを置く。空 Site も偶奇に含める。状態の因子順は Site 順に局所空間の葉をたどった順とし、奇数長 periodic も認める。例えば `[(a,b), c]`、first=Up は `Up_a ⊗ Up_b ⊗ Down_c` となる。

0 Site は単位空間に束縛した zero Hamiltonian と単位状態を返す。全 Site が空でも同様であり、Hamiltonian の空間には空 Site の grouping を残す。

数値能力は個別関数へ限定する。xyz は `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>`、heisenberg はさらに `R: Clone` を要求する。neel の C に Clone・数値演算は不要である。モデル固有関数は QuantumSystem trait に追加しない。

## 3. Fermion の Hubbard モデル

標準 Hubbard と symmetric Hubbard の二種類を提供する。以下の Hamiltonian は合意した相互作用の違いを具体化したものであり、API と配置入力は既存 spin-half に合わせる。

### 3.1. Hamiltonian の規約

一様な実パラメータ t・U、Chain の最近接辺、spin を保存するホッピングを対象とする。両モデルとも化学ポテンシャル項を含めない。

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

標準形は [ALPS の Hubbard 定義](https://alps.comp-phys.org/documentation/models/hubbard/)、symmetric 形は [TRIQS/TPRF の定義](https://triqs.github.io/tprf/3.1.x/user_guide/Linearized%20Eliashberg%20equation%20on%20the%20attractive%20Hubbard%20model.html) のホッピング・相互作用と同じ符号規約である。化学ポテンシャル引数は設けない。

symmetric ではホッピングを変更せず、相互作用の展開で生じる定数項を落とさない。L Site に対して、展開から次の恒等式が得られる。

```math
H_{\mathrm{symmetric}}(t,U)
 =H_{\mathrm{Hubbard}}(t,U)
   -\frac U2 N+\frac{UL}{4}I.
```

`n−I/2=−Mz/2` なので、symmetric な局所相互作用は `(U/4) Mz↑Mz↓` とも表せる。両表現の値の一致を検証する。

### 3.2. 公開入口と配置

既存の `SpinHalf<C>` と同様に `models.rs` で構築し、`models::FermionModels<Fermion<C>>` の関連関数として公開する。QuantumSystem trait にモデル固有メソッドを追加しない。

```rust,ignore
FermionModels::<Fermion<C>>::hubbard(&chain, &sites, t, u)
FermionModels::<Fermion<C>>::symmetric_hubbard(&chain, &sites, t, u)
// いずれも Result<OperatorOf<Fermion<C>, I>, UtilityError> を返す。
```

- `sites: &[HilbertSpace<I>]` の一要素を一つの物理 Site とし、各 Site は葉順 `(up,down)` の二 mode の graded 空間とする。Index は利用側で発行し、関数内で再発行しない。
- 全作用空間は Site 順、各 Site 内の up・down 順で graded 合成し、入力の Site 空間を共有して構築する。返す演算子はこの全空間に bind する。
- ホッピングは隣接 Site の同じ spin 同士だけを結ぶ。spin-half の XYZ が使う全葉ペアの直積は用いない。各辺に両方向のホッピングを一度ずつ加え、符号は core の graded 合成に委ねる。
- open / periodic の辺は既存 Chain の契約に従う。0・1 Site は辺なし、2 Site の periodic でも辺は一つとし、ホッピングを重複させない。
- Site 数の一致、各 Site が二つの葉を持つ graded 空間であること、全 mode の Index 重複を、結合値が0でも検証する。不正な Site 構成は `UtilityError::InvalidSite { site }` とし、列長・空間・式のエラーはそれぞれ `Mapping`・`Space`・`Expression` variant に元エラーを保持する。
- 0 Site は graded な単位空間に束縛した zero Hamiltonian を返す。1 Site はホッピングなしで相互作用を保持する。
- standard Hubbard の係数 bound は `C: Clone + From<R> + Neg<Output=C>`、symmetric Hubbard はさらに `One + Add<Output=C> + Div<Output=C>` を要求する。入力 t・U は同じ R とする。symmetric 相互作用は `(U/4) Mz↑Mz↓` を使い、1/4を係数型の中で生成する。虚数単位・局所 Product・Act・Float は要求しない。t・U が実数に対応すること、分数を数学的に表せることを契約とする。

局所演算子・Majorana の規格化・基底・spinful の位相規約は [Fermion の局所代数](local-structure.md#8-fermion-の局所代数)に従う。係数の中で1/4を生成して `(U/4) Mz↑Mz↓` を構築する実装は [models.rs](../../crates/syntaqs-fermion/src/models.rs) に置く。

### 3.3. 検証

- 二 mode の CAR、spinful の二重占有からの消滅符号、および全空間の mode 順を確認する。
- 1 Site の表示順 `[empty,up,down,double]` で、標準形の対角成分が `[0,0,0,U]`、symmetric 形が `[U/4,−U/4,−U/4,U/4]` となることを確認する。実際の列挙順との対応は明示する。
- 2 Site（4 mode、16状態）の両モデルを、core の評価・export を使わない Fock 空間の符号規則から構築した参照行列と比較する。実パラメータでの Hermite 性、粒子数・各 spin の粒子数の保存、`t=0`・`U=0` の極限を確認する。
- 両モデルの差が `−(U/2)N+(UL/4)I` であることと、symmetric 相互作用の `(U/4) Mz↑Mz↓` による構築を、定数項を含めて比較する。
- 0・1・2 Site と3 Site 以上の open / periodic、Site 数不一致、mode 数の過不足、ordinary な Site、Index 重複、非 Clone Index を検証する。
- `Complex<f32>` / `Complex<f64>` と Vec・ndarray・sprs の同じ基底順で値を比較し、構築の係数 bound に評価能力が混入していないことも確認する。

[モデルのテスト](../../crates/syntaqs-fermion/tests/models.rs)で局所規則と独立した Fock 空間の参照行列を使う。各辺に一度だけホッピングを追加すること、同じ spin のみが接続すること、symmetric 相互作用の定数項を保存することを確認する。実行方法と検証限界は[verification](verification.md)を参照する。

## 4. SpinOne のモデル構築

`syntaqs_spin_one::models::SpinOneModels<SpinOne<C>>` は XYZ・Heisenberg・BLBQ の3モデルを関連関数として提供する。局所 primitive・基底・数値規約は [Spin-one の局所代数](local-structure.md#9-spin-one-の局所代数)に従い、構築は [models.rs](../../crates/syntaqs-spin-one/src/models.rs) に置く。

### 4.1. Hamiltonian の規約

`hbar=1` とし、まず1 Site が1 spin の場合を示す。和は Chain の最近接結合を各1回数える。primitive は `S=√(3/2) J_physical` なので、係数を2/3・4/9で補正し、結合定数の従来の物理的な意味を維持する。

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

符号は既存 SpinHalf のモデルにそろえる。J>0 の bilinear 結合は強磁性的、J<0 は反強磁性的となる。`heisenberg(j)` は `xyz(j,j,j)`、`blbq(j,0)` は `heisenberg(j)` と数学的に同じである。単イオン異方性・磁場項・追加の定数シフトは設けず、上の式のエネルギー原点を保持する。

BLBQ は bilinear 結合 J と biquadratic 結合 K を独立に受け取る。角度表示 `H=sum[cos(theta) A_ij+sin(theta) A_ij²]`、`A_ij=J_physical,i·J_physical,j=(2/3)S_i·S_j` は `blbq(-cos(theta),-sin(theta))` で表せる。角度専用の入口は設けない。Hermite Hamiltonian に対して実数値の結合を渡す契約とし、型による実数制限や実行時の判定は追加しない。

### 4.2. 公開入口と空間

| 公開入口 | 成功時の戻り値 |
| --- | --- |
| `SpinOneModels::<SpinOne<C>>::xyz(&chain, &sites, jx, jy, jz)` | `OperatorOf<SpinOne<C>,I>` |
| `SpinOneModels::<SpinOne<C>>::heisenberg(&chain, &sites, j)` | `OperatorOf<SpinOne<C>,I>` |
| `SpinOneModels::<SpinOne<C>>::blbq(&chain, &sites, j, k)` | `OperatorOf<SpinOne<C>,I>` |

全関数は `&Chain` と `&[HilbertSpace<I>]` を受け取り、`Result<_,syntaqs_spin_one::UtilityError>` を返す。Site 列・全葉ペア・束縛の基本契約は SpinHalf の XYZ / Heisenberg とそろえる。

- 一つの入力空間を一 Site とし、隣接 Site 間の全葉ペア `(a,b)` に相互作用を加える。BLBQ の二乗は各葉ペアの `A_ab=S_a·S_b` に個別に適用し、Site 内の結合や異なる葉ペア間の交差項を加えない。
- 列長と全 Site にわたる Index の一意性を、辺の有無・結合定数によらず検証する。Index には Eq のみを要求し、Clone・Hash・Ord は要求しない。
- 入力 Site の空間ハンドルを列順に tensor した全空間へ Hamiltonian を bind し、内部 grouping・葉順・相互作用に現れない Site を保持する。既存の別の全空間の上位 grouping や因子順は復元しない。
- 空 Site は位置を保ち、前後を直接つなぎ直さない。0 Site は単位空間上の zero Hamiltonian、1 Site と全空 Site は入力の空間構造に束縛した zero Hamiltonian を返す。
- 境界条件と辺は[幾何](geometry.md)の Chain に従う。0・1 Site は辺なし、2 Site は open / periodic とも一辺とする。

エラーは `Mapping(MappingError)`・`Space(SpaceError)`・`Expression(ExpressionError)` の3 variant とし、元エラーを source として保持する。列長不一致は SiteCountMismatch、重複 Index は DuplicateIndex とし、UtilityError を crate root から公開する。

### 4.3. 遅延構築と係数能力

XYZ は配置した Cartesian 成分の積と和、Heisenberg は XYZ の等方的な場合として構築する。BLBQ は各葉ペアの `A_ab=Sx_a Sx_b+Sy_a Sy_b+Sz_a Sz_b` を一度構築し、式を共有して `(&A_ab * &A_ab)` を作る。J・K は2/3・4/9で補正して最後に各項へ適用し、J の符号や大きさを二乗へ混入させない。

| 関数 | 構築に必要な bound |
| --- | --- |
| `xyz<I: Eq,R>` | `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>` |
| `heisenberg<I: Eq,R: Clone>` | `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>` |
| `blbq<I: Eq,R>` | `C: Clone + From<R> + Neg<Output=C> + One + Add<Output=C> + Div<Output=C>` |

構築時に係数 C の中で有理数の補正を行う。局所規則の評価・基底展開・密行列・数値 backend は使わず、C の Float・Zero・共役を要求しない。評価・trace・export 時には既存 core と局所 primitive の操作別の bound が適用される。モデル固有メソッドを QuantumSystem trait に追加しない。

### 4.4. 検証対象とテスト

[models.rs](../../crates/syntaqs-spin-one/tests/models.rs) では、直接記述した3×3スピン行列から参照行列を構成し、モデル実装や Chain の辺列挙を参照計算に使わない。

- `Complex<f32>` / `Complex<f64>` の両方で0〜3 Site・両境界の全行列成分・Hermite 性・trace を比較する。異方的結合、正負の J・K、zero 結合、pure biquadratic 項、等方的・bilinear 極限を含める。
- 二 spin の全スピン0・1・2に対する `A_ij` の固有値 -2・-1・1と、BLBQ のエネルギー `2J-4K`・`J-K`・`-J-K` を、独立した状態の期待値と作用の残差で検証する。
- 複数葉 Site の全葉ペアとペアごとの二乗、空 Site、0 / 1 Site、2 Site の周期辺の重複防止、非連番の因子順、grouping の共有と spectator の保持を確認する。
- 辺なし・zero 結合でも列長不一致と重複 Index を拒否すること、エラーの source、有理数の補正に必要な最小能力だけを持つ係数と非 Clone の結合入力・Index で構築できることを確認する。

## 5. SpinHalf の検証と縦断例

複数葉の全組み合わせ結合、Site ごとの向き、空 Site の偶奇・非再接続、全空・0 Site、重複・列長拒否、非 Clone Index、深い grouping、最小の係数能力を対象とする。モデルのテストは [systems.rs](../../crates/syntaqs-spin-half/tests/systems.rs) と [chain.rs](../../examples/tests/chain.rs) に置く。

実数結合 g の Heisenberg は `H=-(g/4)Σ S_i·S_j`。長さ L≥1 の open chain の Neel 状態では期待値 `g(L−1)/4`、0 Site は0となる。独立な Pauli 行列・代数計算・密疎出力を両精度で照合する。

実行例は [heisenberg.rs](../../examples/src/bin/heisenberg.rs)・[hubbard.rs](../../examples/src/bin/hubbard.rs)・[spin_one.rs](../../examples/src/bin/spin_one.rs)を参照する。Spin-one の `|+,−>` は J=1 の等方的 XYZ / Heisenberg で期待値1、J=1・K=0.25 の BLBQ で0.5となる。
