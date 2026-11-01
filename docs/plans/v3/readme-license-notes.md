# README・ライセンス整備の作業メモ（一時）

作成日: 2026-11-01
更新日: 2026-11-02

[README・ライセンス整備計画](README.md#プロジェクトルートの-readme-とライセンスファイル)に反映するため、会話で決めた事項を一時的に記録する。

## 名前と発音

- 新しいプロジェクト名は **syreqa**。`qmb` は仮の名前として扱う。
- 由来は **Sy**mbolic **R**epresentation and **E**valuation of **Q**uantum **A**lgebra。
- 二酸化珪素を表す英語の **silica** と同じ発音とし、日本語の読みは「シリカ」とする。
- README には発音記号 **/ˈsɪl.ɪ.kə/** を掲載する。
- 発音記号の参照先: [Cambridge Dictionary — silica](https://dictionary.cambridge.org/pronunciation/english/silica)。英米ともに /ˈsɪl.ɪ.kə/ と記載されている。

README での表記案:

> syreqa /ˈsɪl.ɪ.kə/ — Symbolic Representation and Evaluation of Quantum Algebra

同名ソフトウェア・crate の予備検索では重複を確認していないが、crates.io での登録可否は未確認。crate 名・ディレクトリ名・リポジトリ名などへの名称変更はまだ実施していない。

## ライセンス

- **MIT** を単独で採用する。
- 著作権表示は **Copyright (c) 2026 Haruki SAITO** とする。著作権者は本名で表記する。
- 適用範囲は **リポジトリ全体**とする。`crates/` 内に限定せず、自作のコード・利用例・テスト・関連文書を含める。
- 研究・商用を問わず利用できる方針とする。
- ライセンスファイル、README の案内、Cargo のライセンス指定を整合させる。

ルートに LICENSE を配置し、全体への適用を案内する。MIT 本文はソフトウェアと関連文書を対象とする。参照: [MIT 原文](https://opensource.org/license/mit)。

## README の言語と導入方法

- プロジェクトと各 crate の README は **英語**とする。
- 現時点の導入方法は **Git 依存**として案内する。
- 将来的には **crates.io への公開を目指す**。今回の方針確定は公開操作の依頼ではない。

## 作業状況

既存の利用者向け README である `examples/README.md` と `tests/README.md` を英訳した。各ライブラリ crate 内には現時点で README は存在しない。設計・計画文書は既存の日本語を維持している。

ルートの README・ライセンスファイルの作成、各ライブラリ crate の README の作成、名称変更、既存コード・Cargo メタデータの変更は未実施。
