# 式木の表示

更新日: 2026-11-01

局所・複合の状態式と演算子式の Display・Debug、および標準 Index の表示契約を定める。状態と4 Site の Hamiltonian の Display 利用例は [expression_formatting](../../examples/src/bin/expression_formatting.rs)、公開 API の説明は `syntaqs_core::expr` と `syntaqs_basic::Tag` の rustdoc を参照する。

## 1. 対象と型制約

| 入口 | 用途 |
| --- | --- |
| `{}` | 数式風の表示。式側では改行を挿入しない |
| `{:#}` | 同じ数式を構造に沿って改行する表示 |
| `{:?}` / `{:#?}` | `derive(Debug)` による保持構造の表示 |

| 型 | Display の条件 | Debug の条件 |
| --- | --- | --- |
| `LocalState<S, O, C>` | `S, O, C: Display` | `S, O, C: Debug` |
| `LocalOperator<O, C>` | `O, C: Display` | `O, C: Debug` |
| `State<I, S, O, C>` | `I, S, O, C: Display` | `I, S, O, C: Debug` |
| `Operator<I, O, C>` | `I, O, C: Display` | `I, O, C: Debug` |
| `HilbertSpace<I>` | 提供しない | `I: Debug` |

- 表示能力の制約は trait 実装だけに置き、構築・評価には追加しない。表示のための Clone・Eq・Hash・Ord・数値能力も要求しない。
- 保持式を借用して表示する。評価・展開・簡約・係数計算・項の集約をせず、零・一の係数、重複項、二重随伴も残す。
- ordinary / graded とも保存順に従い、並べ替え・符号計算・parity string の挿入をしない。未束縛演算子も表示できる。
- Display は数式本体のみとし、空間・ambient・元の grouping は Debug で確認する。表示文字列は同一性判定や保存形式に使わない。
- Debug は内部の共有構文・節点・作用保持型・HilbertSpace にも derive する。管理用フィールドを含め、葉の Debug に委譲する。内部型の公開や Debug 文字列の互換性保証は不要。
- Display がない型への Debug フォールバック、任意ラベル callback、LaTeX・ASCII 専用モードは今回の対象外。

## 2. Index

### 標準具体型

`syntaqs-basic::Id128` は現行の UUID を維持する。単体の Display・Debug は完全な UUID を表示し、Copy・16 byte・UUID 全体による同一性を変えない。`syntaqs_basic::HilbertSpace` も `HilbertSpace<Id128>` の別名を維持する。

`syntaqs-basic` は以下の標準 Index を提供する。Tag は `syntaqs_core::HilbertSpace<Tag>` で利用し、core は標準具体型に依存しない。

```rust
pub struct Tag {
    raw: String,
}
```

- raw は非公開。`Tag::new(impl Into<String>)`、`as_str()`、`From<String>`、`From<&str>` を提供する。改名 API は追加しない。
- Clone・Debug・PartialEq・Eq・Hash・PartialOrd・Ord を提供し、Copy は提供しない。同一性・Hash・Ord は文字列に従い、隠れた UUID や自動採番は持たせない。
- Display は raw をそのまま表示する。引用符・接頭辞・エスケープ・Unicode 正規化を加えず、空文字列も許容する。Debug は `Tag { raw: "a" }` の形。
- 同名 Tag は別々に構築しても等しい。同じ tensor 空間へ独立な因子として追加した場合は既存の重複検証で拒否する。名前管理は利用側の責任とする。

### 式木内の8文字制限

任意の Index の Display 出力を、式木の Display 内でのみ先頭8文字に制限する。文字数は Rust の `char` 単位とし、省略記号・衝突検出・自動伸長を加えない。Index 単体の Display と式木の Debug は切り詰めない。

- `core::fmt::Write` の内部 adapter で複数回の書き込みを通算する。任意の Display が precision に従うとは限らないため、`{:.8}` だけで実装しない。
- UTF-8 の途中で切断しない。書記素クラスタ数・端末上の列幅は扱わず、ラベル中の改行・区切り文字も補正しない。
- 完全な中間 String や対応表を作らず、上限以降は受理して破棄する。上限到達を `fmt::Error` で表さず、実際の出力エラーは伝播する。
- Index 自身の Display は最後まで実行され得るため、内部処理時間・割り当てまでは定数に制限できない。
- tensor ordering からの命名、Index から空間内の位置を検索する処理は追加しない。

## 3. Display の書式

`A, B, C` は演算子、`psi` は状態を表す説明用ラベル。primitive・係数は自身の Display に委譲し、出力を構文解析しない。状態 primitive はラベルだけを出し、式側が ket 記号と Index を付ける。

| 対象 | 表示 |
| --- | --- |
| 演算子 primitive | primitive 自身の Display |
| 未配置／配置済みの状態 primitive | `\|up>` / `\|up[a]>` |
| 空の和／複合演算子の Identity／空 tensor 上の状態 Unit | `0` / `I` / `unit` |
| 係数倍／和／演算子の積 | `(2+3i) * A` / `A + B + C` / `A * B` |
| 作用／随伴 | `A psi` / `(A)†` |
| 局所演算子の配置 | `Sx[a]` / `(Sx + Sy)[a]` |
| 単純な積状態 | `\|up[a],down[b],up[c]>` |
| 複合的な因子の tensor | `(\|up[a]> + \|down[a]>) ⊗ \|up[b]>` |

### 係数・符号・括弧

係数は常に括弧で囲み、`A + (-1+0i) * B` を減算へ変換しない。`Operator::scalar(c)` は `(c) * I`。局所 identity は保持 primitive の表示に委譲し、表示用の identity は生成しない。

優先順位は弱い順に、和、`⊗` 表記の tensor、作用、積／係数倍、随伴・配置・primitive。まとめた ket は原子的に扱う。以下の場合に子を括弧で囲む。

| 親 | 括弧を付ける子 |
| --- | --- |
| 和 | 和 |
| 積 | 和・係数倍。右の子は積も対象 |
| 係数倍 | 和・積・係数倍・作用・`⊗` 表記の tensor |
| `⊗` 表記の tensor | 和・係数倍・作用・`⊗` 表記の tensor |
| 作用の演算子側 | 和・積・係数倍 |
| 作用の状態側 | 和・係数倍・作用・`⊗` 表記の tensor |
| 随伴 | 常に全体を囲む |
| 局所演算子の配置 | 和・係数倍・積・随伴の全体。空の和 `0` は原子的に扱い、`0[a]` と表示 |

例: `A * B * C`、`A * (B * C)`、`A + (B + C)`、`(2) * ((3) * A)`、`A * ((2) * B)`、`((A)†)†`、`(A * B) |up[a]>`、`A (B |up[a]>)`。

### 状態の配置と tensor

局所状態の Index は、和・係数倍・作用の内側でも各 primitive の ket 内へ引き継ぐ。局所の作用内の演算子にも同じ Index を演算子配置の書式で付ける。新たな節点の生成や分配はしない。

```text
|up[a]> + |down[a]>
(2+3i) * (|up[a]> + |down[a]>)
Sx[a] |up[a]>
(Sx + Sy)[a] |up[a]>
(|up[a]> + |down[a]>) ⊗ |up[b]>
```

- 単純な積状態は、Tensor と、局所式が Primitive の Local のみからなる部分木。葉順に `ラベル[Index]` を並べ、1つの ket にまとめる。単一因子も同じ書式とし、tensor の grouping は表示上省略する。
- 和・係数倍・作用・明示的な Unit を含む部分木全体はまとめず、`⊗` を使う。その子にある単純な積状態はまとめられる。和の展開・係数の移動はしない。
- 空の和は配置後も `0`。空間は Debug で確認する。状態 primitive の文字列から ket 記号を取り除く処理や追加の必須 trait は導入しない。

### 複数行表示

`{:#}` は1段4スペース。和と `⊗` 表記の tensor は operand ごとに改行し、2つ目以降の行頭に `+ `・`⊗ ` を置く。演算記号は operand の開始位置より2桁左へ出し、先頭と後続の operand を揃える。最上位の和・`⊗` 表記の tensor は先頭に2スペースを確保する。括弧や ket の継続行のインデントは、演算記号を除いた operand の開始位置を基準とする。ket は `| ` で始め、カンマの後で改行して各因子のラベルを同じ列に揃える。係数や作用の演算子が前にある場合も、実際に出力した `| ` の直後の列を基準とする。最後の因子と同じ行の ` >` で閉じ、単一因子も `| up[a] >` と1行で表示する。列は Rust の char 単位で数え、端末の表示幅は取得しない。

括弧で囲んだ複合的な operand は内側をインデントする。積・作用の単純な部分では過剰に改行しない。和・`⊗` 表記の tensor を括弧で囲む場合は開き括弧の後と閉じ括弧の前で改行し、他の括弧はインラインに保つ。

```text
(
    A
  + B
  + C
) | up[a],
    down[b],
    up[c] >
```

係数・Index・順序は通常表示と共通。alternate・width・precision を葉へ一括転送せず、width を折り返し幅、precision を係数精度や式の切り詰めへ転用しない。端末幅取得・自動折り返しは行わない。

## 4. builtin primitive

各代数 crate の primitive は Display と derive(Debug) を提供する。以下は primitive 単体のラベル。

| 型 | Display |
| --- | --- |
| SpinHalfOperator | `I, Sx, Sy, Sz, Sp, Sm` |
| SpinOneOperator | `I, Sx, Sy, Sz, Sp, Sm, Dxy, Q0, Qxy, Qyz, Qzx, Q1p, Q1m, Q2p, Q2m` |
| FermionOperator | I → `I`、Annihilate → `a`、Create → `a†`、Number → `n`、Mx/My/Mz → `Mx`/`My`/`Mz` |
| SpinHalfState | Up → `up`、Down → `down` |
| SpinOneState | Plus → `+1`、Zero → `0`、Minus → `-1` |
| FermionState | Empty → `0`、Occupied → `1` |

SpinOneState::Zero と FermionState::Empty は零ベクトルではない。`|0>` と空の和の `0` を区別する。規格化や物理的な結合定数は変更しない。

## 5. 実装と資源の扱い

表示処理は [syntaqs-core/src/expr/format.rs](../../crates/syntaqs-core/src/expr/format.rs) に閉じる。局所／複合、状態／演算子をまたぐ借用の作業スタックを使い、括弧・改行・配置先も反復処理する。和・tensor の子は iterator の継続項目で扱い、全子を一括で積まない。葉や係数の複製、式全体の中間 String、評価用 traversal は使わない。出力エラーでは直ちに停止する。

ket にまとめられる tensor の判定は反復的な後順走査で行う。表示呼び出し内だけの `BTreeMap` に、共有された構文節点のアドレスをキーとして可否の両方を記録する。キーを参照外しせず、Index の同一性判定にも使わない。このため I に Eq・Hash・Ord は不要。判定対象の一意な節点数を U、辺数を E とすると、分類は O((U + E) log(U + 1))、記録は O(U)。深い混合 tensor を祖先ごとに再走査する二乗化を避ける。

表示作業スタックと ket の葉走査は深さに比例する領域を使う。共有部分式は各出現箇所へ表示するため、出力は共有節点数より長くなり得る。Index 以外の自動省略はせず、深い複数行表示のインデントも出力長に含まれる。core の `no_std + alloc`・unsafe 禁止を維持し、新しい依存は追加しない。

derive(Debug) は再帰的なので非常に深い木での成功は保証しない。Display および既存の評価・破棄の深さ耐性と区別する。

## 6. 検証

| 観点 | 確認内容 |
| --- | --- |
| 基本書式 | 4式型の全節点、零・I・unit、零・一・負の係数、重複項、二重随伴、括弧、非可換な積・作用の順序 |
| 状態・配置 | 単一・積・入れ子の tensor、局所の和・係数倍・作用への Index 引継ぎ、複雑な因子を展開しないこと |
| 複数行 | 和・ket・括弧のインデント、空・単項、通常表示と係数・Index・順序が一致すること |
| Index | 0・7・8・9文字以上、Unicode、複数回書き込み、precision を無視する Display、単体と Debug の完全な表示 |
| 標準具体型 | Tag の等値性・hash・Ord・Clone・重複配置拒否。Id128 の Copy・16 byte・UUID の表示と生成 |
| primitive | 全 variant のラベルと ket 組み立て、ket の 0 と零状態の区別 |
| 空間・能力 | 未束縛・束縛・spectator・ordinary / graded、非 Clone の葉、表示だけの型、表示能力なしの既存構築、評価規則を呼ばず入力と共有を維持すること |
| 深さ | 小さいスタックで深さ20,000の Display。深い tensor の判定が二乗にならないこと。Debug は通常の深さで構造・空間・完全な Index を確認 |
| 出力失敗 | 実際の `fmt::Error` を伝播し、以降の葉を表示しないこと |

2026-11-01、Rust 1.99.1 / aarch64-apple-darwin、direnv の devShell で確認した。関連5 crate のテスト・doc test と、以下の workspace 検証は成功した。

```sh
direnv exec . cargo test --workspace --all-features --offline --quiet
direnv exec . cargo check --workspace --all-targets --no-default-features --offline
direnv exec . cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
direnv exec . cargo fmt --all -- --check
direnv exec . cargo doc --workspace --all-features --no-deps --offline
direnv exec . cargo run -p examples --no-default-features --bin expression_formatting --offline
```

書式・型制約・エラー・深さの回帰テストは [core の formatting.rs](../../crates/syntaqs-core/tests/formatting.rs)、Tag と Id128 は [basic の indices.rs](../../crates/syntaqs-basic/tests/indices.rs)、primitive 全 variant は各 builtin crate の `tests/formatting.rs` で確認する。深さ20,000の局所／複合の作用・和・係数倍・積・随伴と混合 tensor を128 KiBのスタックで表示・破棄できた。内部テストでは深い単純 tensor の ket 化と共有部分式の分類も確認した。最後に追加した深さ・Debug・分類のテストは対象を絞って再実行した。

no_std 対応はホスト上の確認であり、専用 target でのビルドは未実施（環境にはホスト target のライブラリのみ導入済み）。MSRV での検証、性能ベンチマーク・速度比較も今回の実施範囲には含めない。[今後の課題](../future-work.md#3-未実施の検証と性能)を参照する。
