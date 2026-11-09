# レビュー結果

今回確認した範囲では、対応を推奨する Defect / Design Concern は特定できなかった。これは全入力・全構成での正しさを証明するものではない。未解決の具体的な不具合仮説も残っていない。

## 対象と契約

- 対象は HEAD `4a7d6d8`。開始時に未コミット差分がなく、比較対象の指定もないため、現行実装を対象とする重点レビューを実施した。
- 契約は [requirements](../../docs/requirements.md)、公開 API の Rustdoc、[Lie 設計](../../docs/design/lie.md)、ルート README を参照した。
- 実装の修正は行っていない。作業成果物はこのディレクトリに限定した。

## Verification

### 静的確認

以下は確認した主要経路であり、全モジュールを同じ深さで精査したという意味ではない。

| 対象 | 主な確認内容 |
| --- | --- |
| core `space.rs` | 空間の順序、重複、ordinary/graded の互換性、unit、次元オーバーフロー、反復的な破棄 |
| core `evaluate/inner_product.rs`, `trace.rs`, `local.rs`, `collect.rs` | 左側共役、右側からの作用、独立因子の縮約、normalized HS の局所正規化、明示空間の優先、項収集 |
| core `coefficient.rs`, `expr/parts/linear_combination.rs` | 複素除算のスケーリング、係数の法則、ゼロ・重複項の保存 |
| core `basis/enumeration.rs`、export `matrix.rs`, `backend/sprs.rs` | unit の列挙、列挙契約違反、基底順序、行列列の構築、疎形式 |
| normalize `norm.rs`, `expectation.rs` | 自己内積による除算、厳密ゼロの拒否、明示 ambient |
| geometry `chain.rs`, `relation/consecutive.rs`、fermion `models.rs` | 端点・周期境界、整数加算の境界、Hubbard の係数とサイト検証 |
| Lie `expression.rs`, `family.rs`, `transform.rs`, `representation.rs` | 文脈整合性、Hermiticity・独立性、実射影、基底変換、閉包、随伴表現 |
| Lie `tolerance.rs`, `numerical.rs`, `invariant.rs`, `decomposition.rs` | 有限値、閾値、rank/nullspace、中心、commutant、完全分解の検証、有限試行・失敗経路 |

関連テストは core の係数契約・空間・深い式のテスト、Lie の分解テストを抜粋して確認した。その他のテストについては実行結果による確認であり、すべてのテスト本文を精査していない。

### 実行チェック

- `cargo fmt --all -- --check`：成功。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：成功。
- `cargo test --workspace --all-features --quiet`：成功（doctest を含む、終了コード 0）。
- `git diff --check`：成功。最終 status は新規 `plan/` のみで、既存追跡ファイルへの変更なし。

### 検証限界

Rust 1.85 での再実行、専用 no_std ターゲット、全 feature 組合せ、export backend の個別構成、下流 BLAS provider fixture、独立した build/doc チェックは今回実行していない。追加の再現プログラムやテスト変更は行っていない。性能の計測や、独立した数学的参照実装による新たな照合も行っていない。
