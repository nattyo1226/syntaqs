# Verification structure and commands

Updated: 2026-11-02

## 1. Verification responsibilities

Feature designs define mathematical contracts and boundary cases. This document describes test placement, common commands, and environment limitations. Historical results, toolchains, and release progress belong in the [v3 verification record](../plans/v3/verification.md).

Production-library tests live in each crate's `tests/` and unit tests. [syntaqs-tests](../../tests/README.md) holds shared three-level and graded two-state algebras, non-orthonormal bases, independent references, and core/normalize/export integration tests. [Examples](../../examples/README.md) include end-to-end model tests. Production crates do not depend on test packages.

## 2. Common checks

Run from the repository root. Prefix commands with `direnv exec .` when development tools are not on PATH.

```sh
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo check --workspace --all-targets --no-default-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

For independent optional-backend checks, use `cargo test -p syntaqs-export --no-default-features` and `cargo test -p syntaqs-tests --no-default-features` with no backend, `--features ndarray`, and `--features sprs`. Workspace all-features covers both together. Keep separate package checks because workspace runs may unify features from other packages.

Compare algebraic evaluation, expansion, and dense/sparse output at both precisions against independent small matrices, Fock-space sign rules, and Gram matrices. Do not construct reference values using the evaluator/export under test. Follow [numerics](numerics.md) for approximate comparison, exact zero, and cancellation residuals. Capability tests and compile_fail doctests also check bounds and incompatible type combinations.

## 3. CI and verification limits

The [CI workflow](../../.github/workflows/ci.yml) runs on pushes to `main`, pull requests, and manual dispatch on 64-bit Linux. Rust 1.85.0 and stable each test the workspace with default, all, and no default features, including doctests and compile_fail cases.

Stable also runs backend-specific export/shared tests, format, Clippy, and API documentation with warnings denied. Dependency caching and cancellation of older runs for the same PR/branch are configured.

Dedicated no_std target builds for core, geometry, spin-half, and fermion remain unverified. Source/dependency-feature inspection and host builds do not establish dedicated-target success. Shared Arc requires pointer-width atomic operations on the target.

Benchmarks, speed comparisons, and additional allocation optimization are also unverified. Tests for sharing, sparse output without dense intermediates, and iterative deep-tree processing are not general speed/memory guarantees. Track further verification and optimization in [future work](../future-work.md#3-outstanding-verification-and-performance).
