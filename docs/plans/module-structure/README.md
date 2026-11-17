# Module structure and public API

## Current phase

The module-file, test-ownership and semantic-refactoring passes are complete. The public API review is complete; implementation has not started. The user accepts breaking changes and does not require compatibility with old paths.

- [API assessment](api-assessment.md): agreed public paths, visibility, preludes and rationale.
- [API implementation plan](api-implementation.md): implementation units, migration and verification.

Planning does not authorize implementation. Preserve behavior, feature gates and supported environments except for explicitly agreed changes.

## Structural conventions

Named modules use `name.rs` and, when needed, a `name/` directory. Crate entry points are excluded. Shared integration-test helpers use `tests/common/mod.rs` so Cargo does not discover them as separate test targets. Core unit tests may use `#[path = "../tests/common/mod.rs"]` to share the coefficient fixture with integration tests. These are the agreed exceptions; preserve test ownership and discovery during API migration.

## Progress and completion

- [x] Complete module-file, test-ownership and semantic structure changes.
- [x] Review public API boundaries and select preludes.
- [ ] Implement the [API plan](api-implementation.md) in verified units.
- [ ] Complete the plan's final cross-workspace checks, review and documentation updates.

The work is complete when the agreed API and preludes are implemented, consumers and documentation use the new paths, intended visibility and behavior are verified, and required checks pass or their limitations are recorded.
