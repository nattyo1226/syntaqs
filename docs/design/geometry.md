# 幾何・Site 関係・借用接続

更新日: 2026-11-02

## 1. 対象と責任分担

`syntaqs-geometry` は幾何、Site の配置規則、外部列への借用接続を分離する。公開型は crate root から利用する。core は geometry に依存せず、geometry は HilbertSpace 接続のため core に依存する。`no_std + alloc` を維持する。

| 型・trait | 責務 |
| --- | --- |
| `Geometry` | `num_sites()` を保証する最小の共通契約 |
| `Chain` | Site 数と境界条件を保持する不変な幾何 |
| `Relation<G>` | G の隣接・方向・境界条件などから Site 番号の Group を列挙 |
| `SiteGroup<CAP>` | 最大容量の配列と有効長。`AsRef<[usize]>` は使用部分だけを公開 |
| `SiteMapper<G,T>` | geometry と Site 順の `&[T]` を借用し、対応付けを検証 |
| `MappedGroup<T,Group>` | 元の Group と外部列を保持し、組内順の `&T` を列挙 |

`Chain::new(num_sites, boundary)` は検証エラーなしで Self を返し、0 Site も認める。`num_sites()` と `boundary()` で保持内容を取得する。Index・HilbertSpace・局所代数・係数型は保持・発行しない。Site は `0..num_sites()` の位置番号であり、専用の SiteId や所属トークンは設けない。

同じ geometry へ複数の Relation を適用でき、同じ規則型を複数の geometry に実装できる。共通の Group 契約は `AsRef<[usize]>` だけとし、固定長配列・可変長・保存済み slice・中心や方向を持つ独自型を使える。Group の順序や重複の意味は各 Relation が定める。SiteGroup 自体はソート・重複除去を行わない。

`SiteGroup<CAP>` の CAP は格納上限であり、実際の Site 数は有効長で表す。`new` は容量超過を `SiteGroupError::CapacityExceeded { capacity, len }` として拒否し、未使用部分を公開しない。組ごとのヒープ確保は不要だが、容量に比例した領域・移動コストはある。

提供する標準実装は共通の trait・借用接続と Chain の最近接・連続配置である。正方格子は [geometry_relations.rs](../../examples/src/bin/geometry_relations.rs) 内の使用例に限り、一般の支持集合の組み合わせ列挙は標準 Relation に含めない。モデル構築の契約は[models](models.md)に定める。

## 2. 関係の列挙と借用接続

`Relation<G>::groups(&geometry)` は独自の関連型 `Error` を返し、成功時は Group を遅延列挙する。設定の検証は Iterator の返却前に行う。エラー型に Debug・Display・Error の実装を一律には要求しない。GAT により規則や geometry 内の保存済みデータも借用できるが、生成 Iterator の可変バッファを貸し出す形式ではない。

| Chain の Relation | 列挙規約 |
| --- | --- |
| `NearestNeighbors` | `[i,j]`（i<j）の辞書順・一意な無向辺。0・1 Site は辺なし、2 Site は両境界で一辺 |
| `Consecutive<K>` | 役割順を保持する K Site の連続配置。open は開始位置 `0..=N-K`、periodic は `0..N` で折り返す |

`Consecutive<0>` は `ConsecutiveError::ZeroLength`、K>N は配置なしとする。K=1 は各 Site を一度ずつ返す。配置内の並べ替えや、同じ支持集合になる回転の統合は行わない。2 Site の periodic で `Consecutive<2>` は `[0,1]`・`[1,0]` を返し、最近接の一辺とは区別する。NearestNeighbors の Chain 実装のエラー型は Infallible である。

N=0・K>=1 は配置なしとし、K>N の場合も同じ Site を繰り返して埋めない。算術 overflow と0による剰余を避ける。例えば N=5・K=3 は次の順序になる。

```text
open:     [0,1,2], [1,2,3], [2,3,4]
periodic: [0,1,2], [1,2,3], [2,3,4], [3,4,0], [4,0,1]
```

組内の位置は局所演算子の役割に対応する。回転した支持集合が同じでも、`X_i Z_j X_k` の中央の役割や graded な積の符号を保つために統合しない。相互作用の対称性による重複の扱いはモデル側が決める。

```rust,ignore
use syntaqs_geometry::{Boundary, Chain, Consecutive, NearestNeighbors, SiteMapper};
let chain = Chain::new(4, Boundary::Open);
let mapper = SiteMapper::new(&chain, &site_spaces)?;
for group in mapper.groups(&Consecutive::<3>)? {
    let group = group?;
    for leaves in group.leaves_cartesian() {
        // 各 Site から葉を一つずつ選び、既存 core で配置・積を構築する。
    }
}
for group in mapper.groups(&NearestNeighbors)? {
    let group = group?;
    // 同じ geometry と外部列に別の Relation を適用できる。
}
```

`SiteMapper::new` は列長の完全一致を Relation の適用前に検証し、空の列挙でも省略しない。不一致は `MappingError::SiteCountMismatch { expected, actual }`。`groups()` の外側の Result は `R::Error`、各 Group の接続結果は `Result<MappedGroup<...>, MappingError>` とする。番号範囲は各 Group を消費する時点で検証し、不正なら `SiteOutOfRange { site, num_sites }` を返す。全件の事前収集や消費済みの結果の巻き戻しは行わない。

geometry の Site 数・番号の意味は mapper の借用中に変えない。列長が一致しても、利用側が意図した座標と列順の意味までは自動検証しない。

MappedGroup の `iter()` は組内順の `&T`、`get(position)` は組内位置の要素、`source()` は元の Group を返す。中心・方向なども保持されるため、独自 Group 用の拡張 trait から読み取れる。要素・Index の Clone・Eq は接続に不要であり、Index 重複は空間・モデル構築側で検証する。

## 3. 空間と局所自由度

局所空間列 `&[HilbertSpace<I>]` の一要素を一 Site に対応させる。葉・複数葉・入れ子・単項 Tensor・空の空間を使える。利用側は既存の空間ハンドルを共有して列を構成でき、Index の Clone・再発行は不要である。

`SiteMapper::from_space` は保存されたトップレベル分割を Site と読む。葉入力は一 Site、空 Tensor は0 Site、`Tensor([unit])` は一つの空 Site、単項 Tensor は一 Site とする。全葉数から Site 数を決めず、局所 grouping を保持する。

HilbertSpace 用 MappedGroup は次の三つの入口を持つ。

| メソッド | `[A⊗B,C⊗D]` に対する結果 |
| --- | --- |
| `leaves_cartesian()` | `[A,C], [A,D], [B,C], [B,D]` |
| `leaves_zip()` | `[A,C], [B,D]` |
| `combine_leaves(FnOnce)` | Site ごとの `Vec<Leaves>` を一つの Iterator にまとめる独自処理 |

任意の Site 数で同じ入口を使い、体数ごとの tuple trait を要求しない。標準の二つは `combine_leaves` を利用する。直積は先頭 Site を最も外側とし、読み取った葉への参照をキャッシュして再走査する。全結果の事前生成や Leaves・Index の Clone は不要である。zip は最短の葉列までとし、いずれも Site 内のテンソル積順を保つ。

`combine_leaves` は Site ごとの `Vec<syntaqs_core::space::Leaves<'s, I>>` を FnOnce に一度だけ渡し、返された Iterator をそのまま返す。出力 Item は固定せず、専用の選択 trait は導入しない。直積を全件消費した場合の参照キャッシュは、入力の葉数の和に比例する。

空 Site を含めば結果なしとし、位置を詰めてつなぎ直さない。Site の組自体が空なら両方とも空の葉列を一つ返す。一つの Site はその葉を一つずつ返す。独自 Fn が順序や対応を変える場合の契約は利用側が定める。

Site 順序と別途存在する全空間の因子順序を区別する。接続は空間・状態・行列を暗黙に並べ替えず、tensor 合成もしない。全作用空間と X/Y/Z などの選択・規格化は利用側、配置・積・bind は既存 core が担当する。各 Site から葉を一つ選ぶ操作は、複数葉に対する全 Pauli string の列挙とは区別する。

既存の `ExpressionFactory::operator_product_at`・`try_operator_product` は任意長の配置・積を受け取れる。体数別の構築 API や geometry 固有の評価器は追加せず、列挙・接続に評価用の数値能力を要求しない。

全 Pauli string の使用例は [pauli_strings.rs](../../examples/src/bin/pauli_strings.rs) に置く。1 Site が1 spin-1/2 の場合、通常の Iterator で大きさ `r = 0..=min(K,N)` の支持集合を一度ずつ選び、各集合へ `3^r` 通りの X/Y/Z を割り当てる。項数は `Σ(r=0..min(K,N)) binomial(N,r) × 3^r` となり、3 Site・高々2 Site では恒等項を含む37項になる。空の支持集合は全作用空間上の恒等項一つであり、`Consecutive<0>` とは別の契約である。Sx/Sy/Sz は Pauli 行列をそのまま表すため、追加の倍率を掛けずに全作用空間へ bind する。構築に必要な制約は `I: Eq, Group: AsRef<[usize]>` と Group の IntoIterator に留める。

K は作用する Site 数であり、実際に作用する葉の数とは区別する。1 Site に m 個の spin-1/2 があれば、その Site 上の非恒等 Pauli string は `4^m - 1` 種類となる。複数葉への全割り当ては利用側の Iterator や独自 combiner で扱い、geometry の標準操作は増やさない。この全割り当ては現在の使用例には含めていない。

## 4. 検証対象

geometry のテストは 0・1・2・3 以上の open／periodic、列長過不足、非 Clone・非 Eq 要素、借用の同一性、入れ子・空・可変長局所空間の順序、全組み合わせと位置対応、トップレベル分割の解釈と `new`／`from_space` の一致、3体・4体の連続配置、独自エラー・役割情報と遅延範囲検証を検証する。

モデルの数値的な規約と検証対象は[モデル構築](models.md)に分ける。

### 4.1. テストと使用例

| 対象 | テスト・使用例 |
| --- | --- |
| 最近接・多体配置の列挙規約、容量制限、保存済み Group の借用、独自エラーと遅延範囲検証 | [geometry.rs](../../crates/syntaqs-geometry/tests/geometry.rs) |
| 最大 Site 数での折り返し算術 | [consecutive.rs の単体テスト](../../crates/syntaqs-geometry/src/relation/consecutive.rs) |
| 入れ子・複数葉・可変長 Group、空入力、終了後の next、借用の同一性、FnOnce、トップレベル解釈 | [space.rs](../../crates/syntaqs-geometry/tests/space.rs) |
| 3体・4体の Pauli 配置、Pauli string 37項、周期境界の fermion 積を手作業構築・独立な行列と比較 | [geometry_relations.rs](../../examples/tests/geometry_relations.rs) |
| 複数 Relation、異なる geometry、方向付き列、中心・方向を持つ可変長近傍、最小限の型制約 | [geometry_relations.rs のサンプル](../../examples/src/bin/geometry_relations.rs) |
| 支持集合の Iterator と Pauli の規格化・独立行列 | [pauli_strings.rs のサンプル](../../examples/src/bin/pauli_strings.rs) |

サンプルは workspace の examples パッケージに含める。実行方法と feature の選択は [examples の README](../../examples/README.md)を参照する。
