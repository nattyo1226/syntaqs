# Version 0.5.0

The following work is scheduled for merge into `main` with version 0.5.0.

## Module structure and public API

Reorganized modules around their responsibilities and made defining module paths the public API, with a prelude for frequently used items in each library crate. This is a breaking change: old compatibility paths and non-prelude re-exports were removed. The spin-half Neel helper was removed; examples construct the state from expression building blocks.

## Core evaluation performance

Implemented and validated local-operator grouping, state-term collection between
actions, exact-zero contraction stopping, and shared spectator identity expansion
for operator coordinates. Ready for merge; decisions, performance comparisons,
and remaining limits are recorded in the
[permanent task record](../../decisions/core-evaluation-performance.md).
