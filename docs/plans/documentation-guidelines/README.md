# Documentation guidelines alignment

The audit phase is complete: all 41 existing Markdown documents were assessed.
There are six confirmed issues and seven editorial or policy-alignment candidates.

- [Findings and proposed editing sequence](audit.md)
- [Assessment of every document](coverage.md)
- [Mechanical check results](checks.json) and [reproduction helper](check_docs.py)

## Objective and scope

Prepare documentation edits based on both `conventions.md` and `writing.md`
from the `doc-maintain` and `doc-write` skills. The conventions govern document
responsibilities, placement, and ownership; the writing guidelines govern
reader-oriented structure, clarity, and technical precision.

The completed audit records findings against both references. Follow-up editing
will be requested separately; this preparation changes only the working records.

The audit covers the root README, `docs/`, and the crate, example, test, and
support READMEs tracked in Git. Inspect implementation and configuration only
where needed to verify a documentation claim. Rustdoc comments and external
websites are outside this audit's systematic coverage.

## Stages and progress

1. Establish the applicable conventions and writing guidelines, and inventory
   the documents — complete.
2. Read each document and assess ownership, current state, links, and writing —
   complete.
3. Verify significant claims against local sources and check local links —
   complete.
4. Record findings, document coverage and limitations, and inspect the final
   changes — complete.

## Completion criteria

- Every document in scope has a recorded assessment.
- Findings identify the applicable rule, concrete evidence, reader impact, and
  a proposed editing direction.
- Verified issues, editorial judgments, and unresolved questions are clearly
  distinguished.
- Local file and heading links and Markdown code fences have been checked,
  with the method and limitations recorded.
- Only this task directory has been added or changed. Existing documentation
  remains unchanged; implementing proposed edits requires a separate request.

Retain this user-requested record for the subsequent editing work. Follow-up edits
have not started. The contents are temporary working material, not authoritative
project specifications.
