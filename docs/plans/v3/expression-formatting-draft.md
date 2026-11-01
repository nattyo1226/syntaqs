# 式木のカスタムフォーマット実装計画（一時）

更新日: 2026-11-01

**合意した仕様を実装するための計画。機能は未実装。** 実装後、採用した契約を設計文書・rustdoc・examples へ移す。

## 1. 対象と基本契約

| 入口 | 表示 |
| --- | --- |
| `{}` | 数式風の表示。式側では改行を挿入しない |
| `{:#}` | 数式風の複数行表示 |
| `{:?}` / `{:#?}` | 基本的に `derive(Debug)` による保持構造の表示 |

- Display の対象は `LocalState`・`LocalOperator`・`State`・`Operator`。必要な表示能力は trait 実装だけに置き、構築・評価の制約へ追加しない。
- 保持している式を借用して表示する。評価・展開・簡約、係数計算、項の集約、因子の並べ替えは行わない。零・一の係数、重複項、二重随伴も残す。
- ordinary / graded とも保存順で表示し、符号計算や parity string の挿入をしない。
- Display は数式本体を表示する。空間・ambient・元の grouping の確認には Debug を使う。未束縛演算子も表示できる。
- 同じ Display になる異なる構造や Index を許容する。表示文字列を同一性判定や保存形式に使わない。
- 今回は標準表示、Tag、builtin primitive の表示を実装する。任意ラベルの callback、行幅指定の自動折り返し、LaTeX・ASCII 専用モードは対象外とする。

## 2. 標準 Index と8文字制限

### 2.1. Id128 と Tag

`qmb-basic::Id128` は現行どおり UUID を保持する。単体の Display・Debug は完全な UUID を表示し、Copy・16 byte・UUID 全体による等値性を維持する。

`qmb-basic` に次の標準 Index を追加する。

```rust
pub struct Tag {
    raw: String,
}
```

- 同一性・Hash・Ord は文字列に従う。同じ文字列の Tag は、別々に構築しても等しい。隠れた UUID・自動採番は持たせない。
- Display は raw を引用符・接頭辞・エスケープなしでそのまま表示する。空文字列や Unicode も補正・正規化しない。
- Clone・Debug・PartialEq・Eq・Hash・PartialOrd・Ord を提供する。Debug は `Tag { raw: "a" }` の形。Copy は提供しない。
- raw は非公開とし、可変の改名 API は初回に含めない。
- constructor は `Tag::new(impl Into<String>)`、参照取得は `as_str()` を基本とし、`From<String>`・`From<&str>` などの補助 API は既存の慣習に沿って整える。
- 同名 Tag を持つ葉を同じ tensor 空間へ独立な因子として追加すると、既存の重複検証で拒否される。名前の選択は利用側が管理する。

`qmb_basic::HilbertSpace` は現行の `HilbertSpace<Id128>` の別名を維持する。Tag は `qmb_core::HilbertSpace<Tag>` で使える。core は標準具体型に依存しない。

### 2.2. 式木内の Index

**式木の Display は、任意の Index の Display の出力を先頭8文字まで表示する。** Id128・Tag・利用者定義型に共通とし、省略記号・衝突検出・桁の自動伸長は加えない。Index 単体の Display と式木の Debug は切り詰めない。

文字数は Rust の `char` 単位とし、UTF-8 の途中で切断しない。書記素クラスタ数やターミナルの表示幅を8列へ揃える処理は行わない。

`core::fmt::Write` の内部 adapter で複数回の書き込みを通算し、8文字以降は受理して破棄する。任意の Display が precision に従うとは限らないため、単に `{:.8}` を渡す実装にはしない。

中間の完全な String や対応表は不要。上限到達を fmt::Error で表さず、実際の出力エラーは伝播させる。Index 自身の Display は最後まで実行され得るため、その型の内部処理時間・割り当てまでは定数に制限できない。ラベル中の改行や区切り文字は解釈・補正せず、利用側が用途に適したラベルを用意する。

Index の表示を tensor ordering から決める処理や、表示用の位置検索は追加しない。

## 3. Display の書式

以下の `A, B, C` は演算子、`psi, phi` は状態を表す説明用ラベル。

| 対象 | 表示 |
| --- | --- |
| 演算子 primitive | primitive 自身の Display |
| 未配置の状態 primitive の式 | `\|up>` |
| 配置済みの状態 primitive | `\|up[a]>` |
| 空の和 | `0` |
| 複合演算子の Identity | `I` |
| 空 tensor 上の状態 Unit | `unit` |
| 係数倍 | `(2+3i) * A` |
| 和 | `A + B + C` |
| 演算子の積 | `A * B` |
| 作用 | `A psi` / `Sx[a] \|up[a]>` |
| 随伴 | `(A)†` |
| 局所演算子の配置 | `Sx[a]` / `(Sx + Sy)[a]` |
| 単純な積状態 | `\|up[a],down[b],up[c]>` |
| 複合的な因子の tensor | `(\|up[a]> + \|down[a]>) ⊗ \|up[b]>` |

### 3.1. 係数・符号・括弧

- 係数は自身の Display を常に括弧内へ出す。実部・虚部の分解、零・一・符号の判定はしない。`A + (-1+0i) * B` を `A - B` へ変換しない。
- `Operator::scalar(c)` は `(c) * I`。局所 identity は保持している primitive の表示に委譲し、表示時に identity を生成しない。
- 優先順位は弱い順に、和、`⊗` 表記の tensor、作用、積／係数倍、随伴・配置・primitive。単一の ket にまとめた積状態は原子的な表記として扱う。
- 和の子が和なら括弧を残す。積は左結合に従い、右側の積は括弧で囲む。
- 係数倍の operand が和・積・係数倍・作用・`⊗` 表記の tensor なら括弧を付ける。積の子が係数倍の場合も括弧で範囲を保つ。
- `⊗` 表記の子が和・係数倍・作用・`⊗` 表記の tensor なら括弧を付ける。
- 作用の演算子が和・積・係数倍なら括弧を付ける。状態が和・係数倍・作用・`⊗` 表記の tensor の場合も同様。まとめた ket や配置済み演算子には追加の括弧を付けない。
- 随伴の対象は常に括弧で囲む。複合的な局所演算子を配置する場合も、全体を括弧で囲んで `[index]` を付ける。
- 任意の primitive の Display はラベルとして使う。出力を構文解析して優先順位を推測しない。

```text
(A + B) * C
A * B * C
A * (B * C)
A + (B + C)
(2+3i) * (A + B)
(2) * ((3) * A)
A * ((2) * B)
(A * B)†
((A)†)†
(A * B) |up[a]>
A (B |up[a]>)
```

### 3.2. 状態の配置と tensor

状態の Index は、和・係数倍・作用の内側でも各状態 primitive の ket 内に付ける。局所状態の Local を読む際に借用の Index を子の表示へ引き継ぎ、表示される和・係数倍・作用に応じて括弧を付ける。

```text
|up[a]> + |down[a]>
(2+3i) * (|up[a]> + |down[a]>)
Sx[a] |up[a]>
(Sx + Sy)[a] |up[a]>
(|up[a]> + |down[a]>) ⊗ |up[b]>
```

局所の Act 内の演算子には、同じ因子の Index を演算子配置の書式で付ける。新たな Local 節点の生成や分配はしない。空の和は配置後も `0` とし、空間は Debug で確認する。

単純な積状態は、Tensor と、局所式が Primitive の Local のみからなる部分木を対象に、`ラベル[Index]` を葉順にカンマで並べて1つの ket にまとめる。入れ子の tensor の grouping は表示上省略し、Debug に委ねる。

和・係数倍・作用・明示的な Unit などを含む部分木全体はまとめず、`⊗` 表記を使う。その子にある単純な積状態はまとめられる。和の展開や係数の移動はしない。

状態 primitive はラベルだけを出し、式側が ket 記号と Index を付ける。`S::Display` の文字列から ket 記号を取り除く処理や追加の必須 trait は導入しない。

### 3.3. 複数行表示

`{:#}` では同じ内容を構造に沿って改行し、1段4スペースでインデントする。

- 和は項ごとに改行し、2項目以降の行頭に `+` を置く。
- 複数因子の ket はカンマの後で改行する。最初の因子は `|` と同じ行、後続因子は継続行、最後の `>` は最後の因子と同じ行に出す。単一因子は1行に保つ。
- 複合的な operand は括弧を保ち、その内側をインデントする。積と作用の改行位置は、単純な部分で過剰に改行しない方針で実装時に調整する。
- Index の8文字制限・係数・因子順序は通常表示と共通とし、alternate 指定を葉の formatter へ一括転送しない。

```text
(
    A
    + B
    + C
) |up[a],
    down[b],
    up[c]>
```

厳密な行幅への折り返しや端末幅の取得は行わない。format の width を折り返し幅へ読み替えず、precision を係数精度や式の切り詰めへ転用しない。

## 4. builtin primitive の表示

各代数 crate に Display を追加し、既存の derive(Debug) を維持する。以下は primitive 単体のラベルであり、状態の ket 記号は式側が付ける。

| 型 | Display |
| --- | --- |
| SpinHalfOperator | `I, Sx, Sy, Sz, Sp, Sm` |
| SpinOneOperator | `I, Sx, Sy, Sz, Sp, Sm, Dxy, Q0, Qxy, Qyz, Qzx, Q1p, Q1m, Q2p, Q2m` |
| FermionOperator | `I, a, a†, n, Mx, My, Mz`（順に I, Annihilate, Create, Number, Mx, My, Mz） |
| SpinHalfState | Up → `up`、Down → `down` |
| SpinOneState | Plus → `+1`、Zero → `0`、Minus → `-1` |
| FermionState | Empty → `0`、Occupied → `1` |

SpinOneState::Zero と FermionState::Empty は零ベクトルではない。`|0>` と空の和の `0` を区別する。規格化や物理的な結合定数は変更しない。

## 5. Debug と型制約

4式型、内部の共有構文・節点・作用保持型、HilbertSpace とその内部節点へ基本的に derive(Debug) を追加し、既存の共通パーツの derive を利用する。内部 enum を公開する必要はない。

管理用フィールド・格納 variant もそのまま表示してよい。primitive・係数・Index は自身の Debug に委譲し、8文字制限は適用しない。Debug 文字列の永続的な互換性は保証しない。

derive による Debug は再帰的に表示するため、非常に深い木での成功は保証しない。Display の反復走査と、既存の評価・破棄の深さ耐性とは区別する。

| 型 | Display の条件 | Debug の条件 |
| --- | --- | --- |
| `LocalState<S, O, C>` | `S, O, C: Display` | `S, O, C: Debug` |
| `LocalOperator<O, C>` | `O, C: Display` | `O, C: Debug` |
| `State<I, S, O, C>` | `I, S, O, C: Display` | `I, S, O, C: Debug` |
| `Operator<I, O, C>` | `I, O, C: Display` | `I, O, C: Debug` |
| `HilbertSpace<I>` | 今回は追加しない | `I: Debug` |

状態は遅延作用を含み得るため O の表示能力も必要。表示のために Clone・Eq・Hash・Ord・数値能力を追加せず、Display がない型への Debug フォールバックも設けない。

## 6. 実装箇所と進め方

| 箇所 | 変更 |
| --- | --- |
| [expr/mod.rs](../../../crates/qmb-core/src/expr/mod.rs)、新設の `expr/format.rs` | 非公開の表示処理と4式型の Display |
| [expr/local.rs](../../../crates/qmb-core/src/expr/local.rs)、[expr/composite.rs](../../../crates/qmb-core/src/expr/composite.rs)、[space.rs](../../../crates/qmb-core/src/space.rs) | Debug の derive。既存の構築・評価・共有は維持 |
| [qmb-basic/src/lib.rs](../../../crates/qmb-basic/src/lib.rs) | Tag の追加。Id128 の既存契約と型別名は維持 |
| 各 builtin crate の `operator.rs`・`state.rs` | 第4節の Display |
| examples、rustdoc、設計文書 | 実装後の使い方・契約・検証結果を反映 |

- core の `no_std + alloc`・unsafe 禁止を維持し、新しい依存を追加しない。評価用の [traversal.rs](../../../crates/qmb-core/src/expr/traversal.rs) は計算・分配を含むため表示に流用しない。
- Display は局所／複合、状態／演算子をまたぐ明示的な作業スタックで処理する。括弧・区切り・改行・インデント・配置先も作業項目で扱い、式の深さに比例する関数再帰を避ける。
- 和・tensor の子は iterator の継続項目で処理し、幅の広い節点の全子を一括で積まない。ket をまとめる判定も反復処理し、祖先ごとの再走査による二乗化を避ける。判定方法と追加メモリは実装時に具体化する。
- 式全体の中間 String や primitive・係数・Index の複製を必須にせず、出力先へ逐次書き込む。出力エラーでは直ちに停止する。
- 共有部分式は各出現箇所へ表示する。出力長は共有節点数より大きくなり得る。表示内容の省略は Index の8文字制限だけとし、式の深さによる自動省略は行わない。
- [Index の設計](../../design/index.md)にある UUID 中心の標準具体型・表示名の契約を、Id128 と Tag を区別する形へ更新する。Tag の同一性は文字列であり、名前管理は利用側が担う。表示が実装済みとなった時点で、関連設計文書・future-work・v3 README の状態を揃える。

## 7. 検証

| 観点 | 確認内容 |
| --- | --- |
| 基本書式 | 4式型の全節点、零・I・unit、係数と負号、二重随伴、入れ子の括弧、非可換な積・作用の順序 |
| 状態・配置 | 単一・積・入れ子の tensor、局所の和・係数倍・作用への Index の引継ぎ、複雑な因子を展開しないこと |
| 複数行 | 和・ket・入れ子の括弧とインデント。通常表示と係数・Index・順序が一致し、空・単項も表示できること |
| Index | 0・7・8・9文字以上、Unicode、複数回の書き込み、precision を無視する Display。単体と Debug は省略されないこと |
| Tag / Id128 | 同名 Tag の等値性・hash・重複配置の拒否・Clone。Id128 の Copy・16 byte・UUID の表示と生成 |
| primitive | 全 variant のラベルと ket 組み立て、ket の 0 と零状態の区別 |
| 空間 | 未束縛・束縛、spectator、ordinary / graded。表示で束縛推定・並べ替え・符号付加をしないこと |
| 能力・共有 | 非 Clone の葉、表示だけの型、表示能力なしの既存構築。評価規則が呼ばれず、入力と共有が維持されること |
| 深さ | 既存テスト同様の小さいスタックで深さ20,000の Display。深い tensor の判定が二乗にならないこと |
| Debug | 通常の深さで derive の構造・空間・完全な Index を確認。深さ20,000での成功は要求しない |
| 出力失敗 | 実際の fmt::Error を伝播し、以降の葉を表示しないこと |

core・basic・builtin 3 crate の関連テストと doc test、workspace の format・Clippy、関連する feature 構成を確認する。ホスト上の確認と専用 no_std target の確認は区別して記録する。

実装・検証はこれから行う。本書中の出力例は仕様であり、実行結果ではない。
