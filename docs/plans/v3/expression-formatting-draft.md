# 式木のカスタムフォーマット実装記録（一時）

更新日: 2026-11-01

**実装・検証済み。** 合意した表示規則・型制約・Index と Tag の契約・実装方法・検証結果は[式木の表示](../../design/expression-formatting.md)へ移した。

- 4式型の Display、`{:#}` の複数行表示、derive(Debug) を追加した。
- 式木内の Index を先頭8文字に制限し、`qmb_basic::Tag` と builtin primitive の Display を追加した。
- 表示は評価せず反復処理する。深さ20,000、Unicode、出力失敗、表示だけの型を検証した。
- 公開 API の契約は `qmb_core::expr` と `qmb_basic::Tag` の rustdoc、利用例は [expression_formatting](../../../examples/src/bin/expression_formatting.rs) を参照する。

専用 no_std target・MSRV での検証は未実施。実施コマンドと確認範囲は[設計文書の検証記録](../../design/expression-formatting.md#6-検証)に記録する。
