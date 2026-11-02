# Storage, restoration, and index reassignment

Updated: 2026-11-02

These contracts concern unimplemented storage, restoration, and independent copies. See [current index specifications](../design/index.md) for issuance and sharing.

## 1. Storage, restoration, and multiple runtimes

Distinguish three operations:

| Operation | Index handling |
| --- | --- |
| Restore a stored object | Retain original indices and references, restoring a description of the same factors. |
| Create an independent system from stored data | Issue fresh indices and consistently reassign space/state/operator references. |
| Combine objects from multiple runtimes | Match factors by index and validate local structures and spaces. |

### 1.1. Restoring the same factors

Restoration must not issue new indices. Repeated restoration of the same data produces the same indices. Separate memory objects describe the same factors; shared ownership or restoration to the same memory object is not required.

Adding two restored copies as independent factors in one space/state tensor product must fail for duplicate basic indices. Multiple references to the same factor are allowed.

### 1.2. Creating an independent system

Unlike identity-preserving restoration, explicitly map old indices to fresh ones. Every reference to one old index receives the same new index, and different old indices receive different new indices. Reassign all space/state/operator references consistently while preserving local structure, factor order, and tensor structure.

### 1.3. Combining runtimes

Equal indices denote the same factor even across runtimes. Index equality alone does not imply space compatibility: apply [index/expression consistency checks](../design/index.md#6-local-algebra-correspondence-and-consistency). S/O/C and operation bounds reject incompatible expression types. No global index-to-structure registry is introduced.

Do not match different indices automatically merely because dimensions or physical meanings agree. Use explicit reassignment under the independent-system contract when independence is intended. Do not silently change indices to avoid duplicates during composition.

Processing internal computational bonds must not implicitly modify syntaqs indices. Tensor-network export needs a separate mapping between physical factors and computational legs.

Storage formats, public APIs, and index reassignment are unimplemented and should be considered independently of parallel evaluation.
