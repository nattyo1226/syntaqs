# Documentation guidelines alignment

The audit phase assessed all 41 existing Markdown documents and identified six
confirmed issues and seven editorial or policy-alignment candidates.
The conventions and writing phases are complete. The writing phase revised the
42 project documents, reviewed the existing task records, and verified the
workspace/crate entry points and tagged-release Quick start.

- [Baseline findings and proposed editing sequence](audit.md)
- [Baseline assessment of every document](coverage.md)
- [Conventions corrections and verification](conventions-results.md)
- [Writing revisions and document coverage](writing-results.md)
- [Minimal root README follow-up](writing-results.md#root-readme-follow-up), with
  [updated checks](root-readme-checks.json)
- Check snapshots: [baseline](checks.json), [conventions](conventions-checks.json),
  and [writing](writing-checks.json); [reproduction helper](check_docs.py)

## Objective and scope

Prepare documentation edits based on both `conventions.md` and `writing.md`
from the `doc-maintain` and `doc-write` skills. The conventions govern document
responsibilities, placement, and ownership; the writing guidelines govern
reader-oriented structure, clarity, and technical precision.

The completed audit records findings against both references. The conventions
phase addressed C01–C06 and E01/E02/E03/E05. The current phase applies writing
guidelines throughout the current documentation, including E04, E06, and E07:
reader-oriented entry points, definition-before-use, coherent paragraphs,
appropriate presentation, and concise references to canonical contracts.

Installation guidance will recommend version-tagged Git dependencies. Verify
the examples against the selected tag and current checkout, keeping differences
in release and development APIs explicit. Preserve the facts and results of
historical audits, decisions, and measurements.

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
7. Review tag compatibility and revise entry-point READMEs — complete.
8. Revise all current designs, requirements, architecture, proposals, decisions,
   and active plans for information flow and precision — complete.
9. Verify examples, links, preserved contracts, and per-document coverage;
   record the writing phase's results — complete.

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
- Version tags are the recommended release-installation method, with consistent
  tag selection across crates and links to release-matched documentation.
- Every Markdown document has a recorded writing assessment; revisions preserve
  technical conditions, historical evidence, and proposal status.
- New and changed executable README examples pass, and the root example runs
  with both the selected release tag and the current checkout.

These records support the user-requested documentation work. They are temporary
working material, not authoritative project specifications.
