# Documentation guidelines alignment

The audit phase assessed all 41 existing Markdown documents and identified six
confirmed issues and seven editorial or policy-alignment candidates.
Conventions-based corrections are complete. The separate writing-focused
revision has not started.

- [Baseline findings and proposed editing sequence](audit.md)
- [Baseline assessment of every document](coverage.md)
- [Conventions corrections and verification](conventions-results.md)
- [Baseline mechanical checks](checks.json), [current checks](conventions-checks.json),
  and [reproduction helper](check_docs.py)

## Objective and scope

Prepare documentation edits based on both `conventions.md` and `writing.md`
from the `doc-maintain` and `doc-write` skills. The conventions govern document
responsibilities, placement, and ownership; the writing guidelines govern
reader-oriented structure, clarity, and technical precision.

The completed audit records findings against both references. The current editing
phase addresses C01–C06 and the conventions-related candidates E01, E02, E03, and
E05. It covers current API descriptions, information ownership, plan requirements,
environment instructions, and the future-work overview. Broader information-flow
and wording changes remain for the later writing-focused task.

The audit covers the root README, `docs/`, and the crate, example, test, and
support READMEs tracked in Git. Inspect implementation and configuration only
where needed to verify a documentation claim. Rustdoc comments and external
websites are outside this audit's systematic coverage.

## Stages and progress

1. Establish the applicable conventions and writing guidelines, and inventory
   the documents — complete.
2. Read each document and assess ownership, current state, links, and writing —
   complete.
3. Verify significant claims against local sources and check audit links —
   complete.
4. Record the audit findings, coverage, and limitations — complete.
5. Recheck implementation and correct the conventions findings — complete.
6. Run the corrected Quick start, validate links and ownership, review the diff,
   and record the conventions phase's results — complete.

## Completion criteria

- Every document in scope has a recorded assessment.
- Findings identify the applicable rule, concrete evidence, reader impact, and
  a proposed editing direction.
- Verified issues, editorial judgments, and unresolved questions are clearly
  distinguished.
- Local file and heading links and Markdown code fences have been checked,
  with the method and limitations recorded.
- C01–C06 and E01/E02/E03/E05 have recorded resolutions grounded in the current
  implementation and shared conventions.
- The corrected Quick start runs against local crates, and affected document
  links and heading anchors pass checks.
- Permanent specifications retain their mathematical contracts and do not link
  to this temporary directory. Production code and dependencies are unchanged.

Retain this user-requested record for the subsequent writing-focused work. The
contents are temporary working material, not authoritative project specifications.
