# qmb-rs workspace tests

`qmb-tests` は、共有するテスト用代数と独立した数値参照計算、およびそれらを使う公開 API・横断テストを持つ workspace パッケージである。`examples` と同様に version・edition・rust-version を workspace から継承し、`publish = false` とする。

`src/graded.rs` は parity を持つ二状態代数、`src/three_level.rs` は非正規直交基底も扱う三状態代数を提供する。`tests/` の各テストはこのライブラリを import し、core・normalize・export の公開 API を検証する。ファイル名の `core_`・`normalize_`・`export_` は主な検証対象を表す。ライブラリ crate の内部実装を検証するテストや固有の fixture は、各 crate に残す。

本番用の crate は `qmb-tests` に依存しない。共有 fixture を使うテストをこのパッケージ内にまとめることで、他の crate の `tests/` を相対パスで読み込まず、テスト支援ライブラリの配布も不要にする。各ライブラリ単体の `cargo test` と、workspace 全体の検証を区別する。

リポジトリ直下から実行する。

```sh
# core・normalize・Vec export のテスト
cargo test -p qmb-tests

# ndarray・sprs を含む全 backend
cargo test -p qmb-tests --all-features

# backend を個別に検証
cargo test -p qmb-tests --no-default-features --features ndarray
cargo test -p qmb-tests --no-default-features --features sprs

# 各 crate・examples・横断テストをまとめて検証
cargo test --workspace --all-features
```

`qmb-tests` の既定 feature は空である。`ndarray`・`sprs` は、それぞれ `qmb-export` の対応 feature を有効にする。単独の backend 構成は `-p qmb-tests` で検証する。workspace 全体では、他パッケージの依存設定から backend が有効になる場合がある。
