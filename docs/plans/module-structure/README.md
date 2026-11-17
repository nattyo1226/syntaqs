# Module structure

## Objective and scope

Organize workspace modules and test ownership in coherent batches. This is a plan only; implementation requires a separate request. Public API paths may change when necessary for the work below. Preserve behavior, public capabilities, feature gates and supported environments except for the explicitly agreed removal of the dedicated Neel constructor and `UtilityError` rename.

The complete public-item inventory, final canonical API and prelude design follow module organization as a separate phase. The agreed direction is semantically meaningful public modules with convenience re-exports only in preludes. Do not create preludes, undertake a workspace-wide API migration, remove existing re-exports wholesale, or treat [API design proposals](api-assessment.md) as an implementation checklist in this task. Existing re-exports can remain temporarily for a coherent, compiling intermediate state; adjust visibility and imports only where a scoped move requires it. Record affected paths for the subsequent API phase. The [recommendations](module-api-recommendations.md) are background proposals, not implementation instructions.

## Work included

1. **Module files:** use `name.rs` plus `name/` for named modules. Crate entry points are excluded. Shared integration-test helpers use `tests/common/mod.rs` as an explicit exception. The core coefficient fixture may be shared with unit tests via the agreed `#[path]`.
2. **Tests:** place tests under the module or common parent whose behavior they exercise. Preserve integration-test crate boundaries and move shared fixtures to `tests/common/mod.rs`. Check imports, privacy and test discovery.
3. **Responsibility-based splits:** split `expr::parts` into cohesive children and assess the documented `syntaqs-lie::family` and `decomposition` splits against their actual dependencies before moving code. Do not expose private machinery or split files merely to meet a size target. Details and candidate destinations are in the [inventory](inventory.md); its API tree is historical and does not define the final API.
4. **Agreed model change:** remove the spin-half Neel constructor; build the example's state from expression building blocks. Rename the three physical-system `UtilityError` types to `models::ModelError`, updating affected consumers. Review integration tests individually: use simpler product states when Neel alternation is incidental, preserve a locally constructed Neel state when the test specifically needs its mathematical properties, and remove tests solely for the deleted constructor. See [API assessment](api-assessment.md#neel-removal-and-integration-tests).

Do not implement the proposed broad renames and public API reshaping (`ops`/`expr`, geometry, export, Lie, or root re-exports) as part of this structural pass. API consequences intrinsic to a selected move are handled locally and documented; final naming and exposure are assessed later.

## Execution plan

- [x] Inspect representative module layouts and record candidate moves, ownership, and exceptions.
- [x] Before implementation, check the current worktree and verify that each listed source and destination still matches the inventory; resolve path collisions and path-sensitive references.
- [x] Convert module entry files and integration-test helper locations by crate; verify Cargo test-target discovery and run affected tests. Workspace all-features tests, all-target no-default-features check, formatting, Clippy, and export no-default-features tests passed; Cargo target names and kinds were unchanged.
- [x] Rehome unit tests by responsibility, adjusting imports and privacy; core unit, integration and documentation tests and core Clippy passed. All 54 unit-test function names were preserved.
- [x] Split `expr::parts` and Lie family/decomposition along their responsibility boundaries. Core and Lie tests and doctests, targeted Clippy, and the core no-default-features check passed; existing public paths remain available for the later API phase.
- [ ] Apply the agreed Neel and model-error changes; update affected tests, example, and current documentation, and run affected crate tests and example checks.
- [ ] Review the complete diff for unintended behavior, lost public capabilities, changed test coverage and unrelated changes; run repository-required formatting, lint, tests, doctests and builds as applicable, recording any limits.
- [ ] Record the resulting structure and API changes for the later API inventory and prelude phase. Do not mark the API phase complete as part of this task.

Batches may be reordered where dependencies make it safer, but each must leave a coherent build and receive relevant path and test checks. Planning does not authorize code changes.

## Completion criteria

Named modules follow the file convention with the documented helper exception; tests are owned appropriately; selected responsibility splits are coherent; and the explicitly requested model change is completed. Affected consumers and documentation reflect the resulting behavior, with no unintended loss of public functionality. Required checks pass or their limitations are recorded. Full API and prelude decisions remain deferred.
