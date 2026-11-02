# v3 公開準備の計画と経過

更新日: 2026-11-02

## 1. 現在の作業と残る項目

現在進行中の v3 公開準備の計画・経過を記録する。提供済みの仕様は `design/`、着手時期を決めていない追加機能は [future-work](../../future-work.md)で管理する。

| 項目 | 状態・次の作業 |
| --- | --- |
| 文書間の責任分離 | 再構成を実施。要件・概要・機能別仕様・将来機能・進行中の計画を整理した。 |
| `docs/` の英語化 | 未着手。`plans/` を含む全ドキュメントを英語に統一する。 |
| README・Cargo.toml の crate 順序 | 未着手。責任に基づく順序を、外部依存と各パッケージの依存一覧にも適用する。 |

確定した方針と個別の修正項目は[ドキュメントの最終調整](documentation.md)に記録する。英語化と依存一覧の並べ替えは、責任分離の再構成とは別の作業として残る。

## 2. 公開準備の経過

### 2.1. Primitive の命名・規格化

2026-11-01 に実装・検証を完了した。現在の primitive と規格化は[局所代数](../../design/local-structure.md)、次元規格化した HS 内積は[局所式](../../design/primitive-expression.md)、物理的な結合定数の補正は[モデル構築](../../design/models.md)に定める。実行した検証は[経過の記録](verification.md)にまとめる。

### 2.2. 式木表示

2026-11-01 に実装・検証を完了した。通常・複数行表示、Debug、Index の8文字制限、Tag と builtin primitive のラベルを提供する。現在の契約は[式木の表示](../../design/expression-formatting.md)、利用例は [expression_formatting](../../../examples/src/bin/expression_formatting.rs)を参照する。

### 2.3. README とライセンス

2026-11-02 に導入案内・crate README・MIT ライセンスを整備し、ローカルリンク・利用例・ライセンスと Cargo メタデータの対応を確認した。現在の案内は[ルート README](../../../README.md)、ライセンス本文は [LICENSE](../../../LICENSE)に置く。著作権表示は `Copyright (c) 2026 Haruki SAITO`、適用範囲はリポジトリ全体である。

利用例の Git 依存は手元のリポジトリへ対応付けて検証しており、改名済みコードの GitHub 反映後の導入確認は未実施である。README の crate 順序は最終調整の対象として残る。

### 2.4. GitHub Actions

2026-11-02 に workflow を整備し、利用者が一時的に `feat/workflow` を push 対象へ追加して、GitHub Actions 上で正常に実行されることを確認した。確認後の push 対象は `main` のみとした。

現在の workflow の構成・実行方法・検証限界は[verification](../../design/verification.md)に定める。専用 no_std target は workflow の対象に含めていない。

### 2.5. 多体関係の列挙

2026-10-31 に実装・受入確認を完了した。現在の Relation・Group・SiteMapper と多体の葉選択は[幾何](../../design/geometry.md)、利用例は[examples](../../../examples/README.md)、当時の確認は[検証の経過](verification.md)を参照する。

### 2.6. 文書間の責任分離

要件から実装詳細・移行履歴を分け、architecture を全体構造の概要に整理した。設計文書は現在の機能別仕様とし、構築入口・正規化・数値規約・モデル構築・検証構成を独立させた。未実装機能の詳細と API 候補を `future-work/`、先行検証の記録を本計画の経過へ移した。

文書・見出しの移動に合わせ、リポジトリ内の Markdown リンクを更新した。2026-11-02 の再編時に38ファイル・340件のローカルリンクとコードフェンスを確認し、リンク切れと未閉鎖のフェンスはなかった。`git diff --check` と、`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` が成功した。実装コード・依存設定の変更はなく、workspace テストは今回再実行していない。
