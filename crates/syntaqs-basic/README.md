# syntaqs-basic

Standard factor identifiers for [syntaqs](../../README.md).

## Responsibilities

- Issue UUID v4 identifiers through `Id128::new()` and `Default`.
- Provide caller-chosen names through `Tag`.
- Offer `HilbertSpace` as an alias for `syntaqs_core::HilbertSpace<Id128>`.

`Id128` generation uses the operating system's randomness through the uuid crate. Copying an existing identifier preserves its identity. `Tag` equality uses the complete string, so callers must choose distinct names for independent factors.

## Integration

The generic core accepts any suitable equality-comparable index type; using this crate is optional. Identifiers determine factor identity, while primitive types and their rules determine the local algebra. Space construction and expression evaluation belong to [syntaqs-core](../syntaqs-core/README.md).

This crate uses the standard library. Its index types provide Display and Debug; expression Display truncates index labels to eight Unicode scalar values while identity comparisons retain the complete value.

See the [crate API overview](src/lib.rs), [index design](../../docs/design/index.md), and [expression formatting design](../../docs/design/expression-formatting.md). Installation is covered by the [project README](../../README.md).
