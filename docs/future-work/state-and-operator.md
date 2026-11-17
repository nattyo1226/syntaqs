# Reviewing State, Operator, and space management

Updated: 2026-11-17

## 1. Review scope

Review whether general operators offer a useful mathematical account of both
states and operators, and what that implies for ownership and sharing. The
review includes local algebra, spaces, and geometry-to-space assignments.
It has not started; design, timing, and implementation scope are undecided.

## 2. Relating operators and states

Currently, State is a ket with domain/codomain `ℂ→H`, while a bound Operator has
`H→H`. Local and composite representations each have separate state/operator
trees and evaluation, with no general `U→V` map type. Use these current facts as
the starting point for deciding how mathematical commonality should map to APIs.

Review these questions:

- What mathematical object does a general operator represent, and what kind of state is meant? Do not assume kets, density operators, and quantum operations are one object.
- How do composition, tensor product, adjoint, inner product, trace, zero/identity, and domains/codomains relate?
- How far should mathematical commonality extend into public types, representations, or shared processing? Decide public-type unification separately.
- How do unbound placement expressions relate to maps with definite domains/codomains?

Recheck space/geometry boundaries and scope type/evaluator integration only after responsibilities are clear. Consider this independently of grading storage and APIs. Completion means documenting the mathematical objects, API mapping, shared and distinct parts, and reasons, including a possible decision against unification.
