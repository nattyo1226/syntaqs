# Installing syntaqs

Use a version tag for release installation. This is the recommended way to
select a release and keep all syntaqs crates on the same revision.

## Basic dependencies

Add the crates your application uses to `Cargo.toml`, with the same tag on each
syntaqs dependency. For example:

```toml
[dependencies]
syntaqs-core = { git = "https://github.com/nattyo1226/syntaqs", tag = "v0.5.0" }
syntaqs-spin-half = { git = "https://github.com/nattyo1226/syntaqs", tag = "v0.5.0" }

num-complex = "0.4.6"
```

The workspace uses Rust edition 2024 and requires Rust 1.85 or newer.
Publishing to crates.io is planned; current installation uses Git.

## Match documentation to the version

For version 0.5.0, use the
[documentation and examples at v0.5.0](https://github.com/nattyo1226/syntaqs/tree/v0.5.0).
Module paths can change between releases; keep dependencies, documentation, and
examples on the same tag.

For unreleased work, select a reviewed commit with `rev`. Cargo's
[Git dependency reference](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choice-of-commit)
explains tag, commit, and branch selection.

## Optional crates and features

Normalization, numerical export, and Lie analysis are separate crates. Add the
entries you need to the same dependency table:

```toml
syntaqs-normalize = { git = "https://github.com/nattyo1226/syntaqs", tag = "v0.5.0" }
syntaqs-export = { git = "https://github.com/nattyo1226/syntaqs", tag = "v0.5.0", features = ["ndarray", "sprs"] }
syntaqs-lie = { git = "https://github.com/nattyo1226/syntaqs", tag = "v0.5.0" }
```

- [Normalize](../crates/syntaqs-normalize/README.md) adds square-root norms,
  state normalization, and normalized expectations.
- [Export](../crates/syntaqs-export/README.md) always supports Vec output and has
  no default features. Enable `ndarray` and `sprs` independently for their
  container APIs, and declare direct dependencies on container crates whose
  types you use.
- [Lie analysis](../crates/syntaqs-lie/README.md) requires an application-selected
  BLAS/LAPACK provider. Follow its [native-linkage setup](../crates/syntaqs-lie/README.md#native-linkage)
  when adding the crate.

See [examples](../examples/README.md) for runnable workflows and
[verification](design/verification.md) for repository development requirements
and checks.
