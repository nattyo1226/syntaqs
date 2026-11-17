# syntaqs-basic

Choose UUID or named identifiers for the Hilbert-space factors in
[syntaqs](../../README.md). An identifier determines which factor an expression
refers to; primitive types determine that factor's local algebra.

| Type | Use when | Identity rule |
| --- | --- | --- |
| `Id128` | Independent factors need generated identifiers. | UUID v4; copying preserves identity. |
| `Tag` | The caller manages meaningful names. | The complete string; equal names identify the same factor. |

Use [version-tagged Git dependencies](../../docs/installation.md) for release
installation and read documentation at the same tag. The paths below describe
the current checkout. This crate uses std.

## Creating identifiers

Both types live in `syntaqs_basic::index` and are also available in the prelude.
For example:

```rust
use syntaqs_basic::index::{Id128, Tag};

fn main() {
    let id = Id128::new();
    let same_factor = id;
    assert_eq!(id, same_factor);

    let site = Tag::new("site-0");
    assert_eq!(site.as_str(), "site-0");
    assert_eq!(site, Tag::from("site-0"));
}
```

Pass either type to `syntaqs_core::space::HilbertSpace::factor`. Core accepts any
suitable equality-comparable index type, so this crate is optional.

`Id128::new()` and `Default` obtain randomness through the uuid crate. `Tag`
callers must choose distinct names for independent factors. Standalone Display
and Debug retain full identifiers; expression Display shows only the first eight
Unicode scalar values, which never affect identity comparisons.

## Integration and references

See the [API overview](src/lib.rs), [identity contract](../../docs/design/index.md),
and [expression formatting](../../docs/design/expression-formatting.md).
