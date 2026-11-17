# blas-provider

This unpublished package selects the system OpenBLAS used by workspace tests
and runnable examples. It is shared development support under `support/`, separate
from the public library crates and test suites.

It selects system OpenBLAS through `blas-src`. Environments that run linked
workspace tests and examples must provide OpenBLAS, including LAPACK, and
pkg-config. Binaries and tests retain the native link with
`extern crate blas_provider as _;`.

Downstream applications select their own provider through the
[Lie native-linkage setup](../../crates/syntaqs-lie/README.md#native-linkage).
Production libraries retain provider-neutral dependencies. See the
[verification guide](../../docs/design/verification.md#2-common-checks) for workspace commands.
