# Module structure

## Objective and scope

Evaluate and plan a consistent Rust module layout that makes ownership and related functionality easier to find. This plan covers source and test module organization across the workspace. The user has accepted public path changes; preserve observable behavior unless a change is separately requested. This plan does not itself authorize implementation.

## Assessment

1. **Use `name.rs` with `name/` for children.** Apply this to all named modules, including integration-test support modules: it makes the module entry point identifiable by name and eliminates mixed layouts. Rust supports both forms, so this is a consistency choice, not a correctness requirement. Inventory `mod.rs` files first, and check `#[path]`, `include!`, generated paths and file references before moving them. Crate entry points such as `lib.rs` and `main.rs` are excluded. Check for filename collisions before each move.
2. **Place unit tests under the module they test.** Recommend this when a test has a clear owner: for example, `expr::composite_tests` and `expr::group_tests` are currently siblings of their corresponding modules. Nesting changes relative imports and privacy context, so inspect each test and its helpers rather than moving mechanically. Choose the owner by the responsibility being tested, not by whether a matching module name exists. Put tests of one module under its `tests`; put cross-module or parent-level tests under the nearest common owner’s `tests`, using topic files such as `tests/state.rs` when useful. Consider integration tests for behavior observed through the public API; they retain their separate crate boundary. Do not create a production module solely to match a test name. Assess `evaluate::state_tests` and `stopping_tests` individually: `evaluate::tests::stopping` is a candidate for cross-cutting stopping behavior, and mixed tests can be split by responsibility.
3. **Split coherent responsibilities into child modules.** Recommend this as a design criterion, not a fixed line limit or a requirement that every item have its own file. In `expr::parts`, `linear_combination` is a child while `ScaleExpr`, `SumExpr`, `ProductExpr`, `ActExpr`, and `AdjointExpr` are defined directly. Assess their responsibilities and dependencies before choosing groupings. Extra indirection or tiny modules without a distinct responsibility can make navigation worse. Keep a parent as a small facade where that reflects its role, and public path compatibility is not required. Choose visibility and re-exports based on the resulting API, and update affected imports and documentation; check rustdoc and downstream usage as well as compilation.

## Stages and progress

- [x] Inspect representative layouts and assess the proposed rules. The examples above are observations, not a complete inventory.
- [ ] Inventory all module entry points, test ownership, oversized or mixed-responsibility modules, and path-sensitive references. Record exceptions and proposed boundaries.
- [ ] Determine concrete ownership and boundaries from responsibilities and dependencies, and record consequential public path changes.
- [ ] If separately authorized, migrate in coherent batches, adjust affected paths, and run relevant tests and path checks for each batch.
- [ ] Review the final structure and diff, run workspace-required checks and relevant tests, and document any limitations.

## Completion criteria

The inventory is resolved, named module entry points follow the chosen convention, tests live under their appropriate owner, and module boundaries reflect cohesive responsibilities. Public paths may change; observable behavior remains intact unless a change is separately approved. Applicable workspace checks and tests pass, or any failures and verification gaps are recorded.
