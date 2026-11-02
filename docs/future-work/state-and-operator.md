# Reviewing State, Operator, and space management

Updated: 2026-11-02

## 1. Review scope

Revisit ownership, sharing, associations among State, Operator, Structure, and Space, and dependencies between Geometry and spaces. Design and timing are undecided. Current primitive/space/expression boundaries are not fixed as the final design.

## 2. Relating operators and states

This review has not started; timing and implementation scope are undecided.

Consider whether operational quantum theory can organize both as special cases of general operators, and decide how mathematical meaning maps to APIs. Current State is a ket with domain/codomain `ℂ→H`; bound Operator has `H→H`. Local and composite representations each have separate state/operator trees and evaluation, with no type for a general `U→V` map.

Review these questions:

- What mathematical object does a general operator represent, and what kind of state is meant? Do not assume kets, density operators, and quantum operations are one object.
- How do composition, tensor product, adjoint, inner product, trace, zero/identity, and domains/codomains relate?
- How far should mathematical commonality extend into public types, representations, or shared processing? Decide public-type unification separately.
- How do unbound placement expressions relate to maps with definite domains/codomains?

Recheck space/geometry boundaries and scope type/evaluator integration only after responsibilities are clear. Consider this independently of grading storage and APIs. Completion means documenting the mathematical objects, API mapping, shared and distinct parts, and reasons, including a possible decision against unification.
