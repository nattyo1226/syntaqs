# blas-provider

Internal BLAS/LAPACK provider shared by workspace development, runnable examples,
and verification. This package is not published and is kept outside the public
library crates and test suites.

It selects system OpenBLAS through `blas-src`. Environments that run linked
workspace tests and examples must provide OpenBLAS, including LAPACK, and
pkg-config. Import `extern crate blas_provider as _;`
in binaries or tests that need the native linkage. Production libraries retain
provider-neutral dependencies so applications can select their own provider.
