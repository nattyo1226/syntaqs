# 検証の構成と実行方法

更新日: 2026-11-02

## 1. 検証の責任

各機能の数学的契約と境界条件は対応する `design/` の文書に定める。本書は検証の配置・共通の実行方法・環境上の限界を記述する。過去の実行結果・当時の toolchain・公開準備の経過は[v3 の検証記録](../plans/v3/verification.md)で管理する。

本番ライブラリのテストは各 crate の `tests/` と単体テストに置く。共有の三準位・graded 二状態代数、非正規直交基底、独立参照計算と core・normalize・export の横断テストは [syntaqs-tests](../../tests/README.md)、モデル利用の縦断テストは [examples](../../examples/README.md)に置く。本番 crate はテストパッケージに依存しない。

## 2. 共通の確認方法

リポジトリ直下から実行する。開発ツールが PATH にない場合は `direnv exec .` を付ける。

```sh
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo check --workspace --all-targets --no-default-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

任意 backend の独立検証には `cargo test -p syntaqs-export --no-default-features` と `cargo test -p syntaqs-tests --no-default-features` を使い、それぞれ backend なし・`--features ndarray`・`--features sprs` を確認する。両 backend は workspace の全 feature 構成で確認する。workspace 一括実行では他パッケージからの feature 統合があるため、単独検証も区別する。

独立な小規模行列・Fock 空間の符号規則・Gram 行列を参照に、代数計算・基底展開・密疎出力を両精度で比較する。実装中の evaluator や export で参照値そのものを生成しない。近似比較・厳密零・相殺残差は[数値規約](numerics.md)に従う。型制約と誤った型の合成は capability テスト・compile_fail doctest でも確認する。

## 3. CI と検証限界

[CI workflow](../../.github/workflows/ci.yml) は `main` への push・pull request・手動実行を契機に Linux の64 bit 環境で動作する。Rust 1.85.0 と stable それぞれで、workspace の既定・全 feature・既定 feature 無効のテストを実行し、doctest・compile_fail も含める。

Stable では export・共有テストの backend 別検証、format・Clippy、警告をエラーとする API ドキュメント生成を行う。依存キャッシュと同じ PR・ブランチの古い実行のキャンセルを設定する。

`no_std + alloc` を宣言する core・geometry・spin-half・fermion の専用 target での実ビルドは未実施である。ソース・依存 feature とホスト上の確認を、専用 target での成功とみなさない。共有の Arc はポインタ幅の atomic 操作を持つ target を前提とする。

性能ベンチマーク・速度比較・追加の割り当て最適化も未実施である。共有・疎出力で密中間領域を作らないこと・深い木の反復処理に関するテストを、一般的な速度やメモリ量の保証と混同しない。今後の検証・最適化は[future-work](../future-work.md#3-未実施の検証と性能)に記録する。
