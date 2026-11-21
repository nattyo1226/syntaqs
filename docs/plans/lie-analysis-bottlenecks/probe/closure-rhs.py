"""Instrument RHS preparation and compare paired Lie analysis builds."""

import argparse
import csv
from hashlib import sha256
from pathlib import Path

from measure import CASES, MEASUREMENTS, SITE_CASES, measure_suite, run


RHS_CASES = ("fermion-4", "pauli-2-mixed", "heisenberg-16")


def replace_once(source, old, new):
    if source.count(old) != 1:
        raise SystemExit(f"Expected one occurrence of {old!r}")
    return source.replace(old, new)


def profile(snapshot):
    path = snapshot / "crates/syntaqs-core/src/evaluate/operator/contraction.rs"
    source = replace_once(
        path.read_text(),
        "let mut evaluated = Self::new(lhs, space);",
        'let mut evaluated = crate::hotspot::measure("hs_prepare_left", || Self::new(lhs, space));',
    )
    source = replace_once(
        source,
        ".then(|| evaluated.evaluate(rhs, space));",
        '.then(|| crate::hotspot::measure("hs_prepare_right", || evaluated.evaluate(rhs, space)));',
    )
    if "pub fn prepare_hs_against_in" in source:
        source = replace_once(
            source,
            "basis.push(evaluator.evaluate(&operand.expression, ambient));",
            '''basis.push(crate::hotspot::measure("hs_prepare_left", || {
                evaluator.evaluate(&operand.expression, ambient)
            }));''',
        )
        source = replace_once(
            source,
            "let right = evaluator.evaluate(&target.expression, ambient);",
            '''let right = crate::hotspot::measure("hs_prepare_right", || {
                evaluator.evaluate(&target.expression, ambient)
            });''',
        )
        representation = snapshot / "crates/syntaqs-lie/src/representation.rs"
        representation.write_text(
            replace_once(
                representation.read_text(),
                "let mut overlaps = E::prepare_hs_against(family.operators(), family.context())?;",
                '''let mut overlaps = syntaqs_core::hotspot::phase("rhs_basis", || {
            E::prepare_hs_against(family.operators(), family.context())
        })?;''',
            )
        )
    path.write_text(source)

    projection = snapshot / "crates/syntaqs-lie/src/family/projection.rs"
    source = projection.read_text()
    start = source.index("        let rhs = self\n", source.index("    fn solve_target"))
    end = source.index("        let values = self", start)
    rhs = "".join("    " + line for line in source[start:end].splitlines(keepends=True))
    projection.write_text(
        source[:start] + '''        let rhs = syntaqs_core::hotspot::phase("rhs", || {
''' + rhs + '''            Ok::<_, LieError<E::Real>>(rhs)
        })?;
''' + source[end:]
    )


def compare_invariants(binaries, output, labels):
    with output.open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow(("comparison", "case", "matched_lines", "sha256"))
        for case in dict.fromkeys((*CASES, *SITE_CASES)):
            lines = []
            for variant in ("pre", "post"):
                stdout, _ = run(binaries / f"{variant}-diagnostics", case, "all")
                lines.append([line for line in stdout.splitlines() if line.startswith("# invariant ")])
            if not lines[0] or lines[0] != lines[1]:
                raise SystemExit(f"Invariant diagnostics differ for {case} in {labels}")
            digest = sha256("\n".join(lines[0]).encode()).hexdigest()
            writer.writerow(("/".join(labels), case, len(lines[0]), digest))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("prepare").add_argument("snapshot", type=Path)
    for name in ("measure", "invariants"):
        command = commands.add_parser(name)
        command.add_argument("binaries", type=Path)
        command.add_argument("output", type=Path)
        command.add_argument("--revisions", nargs=2, required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        profile(args.snapshot)
    elif args.command == "measure":
        args.output.mkdir(parents=True, exist_ok=True)
        revisions = dict(zip(("pre", "post"), args.revisions))
        MEASUREMENTS["timings"] = (RHS_CASES, ("closure",), 7)
        MEASUREMENTS["hotspots"] = (RHS_CASES, ("closure",), 5)
        for suite in ("timings", "hotspots"):
            measure_suite(args.binaries, args.output, suite, revisions)
    else:
        compare_invariants(args.binaries, args.output, args.revisions)
