"""Prepare isolated comparators and measure closure term-pair filtering."""

import argparse
import csv
from pathlib import Path
import re

from measure import run, timings


CASES = (
    "heisenberg-2", "heisenberg-4", "heisenberg-8", "heisenberg-12", "heisenberg-16",
    "heisenberg-2-in-64", "fermion-2", "fermion-3", "fermion-4", "fermion-tail-64",
    "pauli-2-mixed", "spin-half-2",
)
PROFILE_CASES = ("heisenberg-2", "heisenberg-16", "heisenberg-2-in-64", "pauli-2-mixed")


def replace_once(source, old, new):
    if source.count(old) != 1:
        raise SystemExit(f"Expected one occurrence of {old!r}")
    return source.replace(old, new)


def prepare(snapshot, variant):
    if variant == "eager":
        path = snapshot / "crates/syntaqs-lie/src/expression.rs"
        source = path.read_text()
        start = source.index("        if let Some(commutator)")
        end = source.index("    fn hs_in", start)
        source = source[:start] + """        self.in_context(context)?
            .lie_bracket_expression(&rhs.in_context(context)?)?
            .collect_terms_in(context)
    }
""" + source[end:]
        path.write_text(source)
        return

    path = snapshot / "crates/syntaqs-core/src/evaluate/operator/commutator.rs"
    source = replace_once(path.read_text(), "use super::", "extern crate std;\n\nuse super::")
    setup = "        let mut terms_at"
    finished = "        let mut sum = Sum::new();"
    if variant == "all-pairs-profile":
        source = replace_once(source, "collections::BTreeMap, ", "")
        source = replace_once(source, "OperatorTerm, ", "")
        start = source.index("        // The index keeps term positions")
        end = source.index(finished, start)
        source = source[:start] + "        let index_ms = 0.0;\n" + source[end:]
        start = source.index("            for &(position, _) in &a.factors")
        end = source.index("            for index in candidates.drain(..)", start)
        source = source[:start] + "            candidates.extend(0..right.terms.len());\n" + source[end:]
        start = source.index("    fn term_is_odd")
        end = source.index("\n}\n", start)
        source = source[:start] + source[end:]
    else:
        source = replace_once(source, setup, "        let start = std::time::Instant::now();\n" + setup)
        source = replace_once(
            source, finished,
            "        let index_ms = start.elapsed().as_secs_f64() * 1e3;\n" + finished,
        )
    source = replace_once(source, finished, finished + "\n        let (mut visited, mut built) = (0, 0);")
    source = replace_once(
        source,
        "                for term in self.multiply_terms(a, &right.terms[index]) {",
        """                visited += 1;
                let terms = self.multiply_terms(a, &right.terms[index]);
                built += terms.len();
                for term in terms {""",
    )
    source = replace_once(
        source,
        "        sum.finish(&self.primitives)",
        """        std::println!(
            "# commutator,{},{},{visited},{built},{index_ms:.6}",
            left.terms.len(), right.terms.len(),
        );
        sum.finish(&self.primitives)""",
    )
    path.write_text(source)


def measure(binaries, output):
    output.mkdir(parents=True, exist_ok=True)
    with (output / "timings.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow(("case", "run", "variant", "D", "n", "closure_ms", "peak_rss_bytes"))
        variants = ("pre", "eager", "post")
        for trial in range(9):
            for index, case in enumerate(CASES):
                start = (trial + index) % len(variants)
                order = variants[start:] + variants[:start]
                if (trial + index) % 2:
                    order = order[::-1]
                expected_dims = None
                for variant in order:
                    stdout, rss = run(binaries / variant, case, "closure")
                    dims = re.search(r"D=(\d+), n=(\d+)", stdout).groups()
                    if expected_dims is not None and dims != expected_dims:
                        raise SystemExit(f"Builds disagree on dimensions for {case}")
                    expected_dims = dims
                    writer.writerow((case, trial + 1, variant, *dims, timings(stdout)["closure"], rss))
            data.flush()
            print(f"commutator timings: completed trial {trial + 1}/9", flush=True)

    with (output / "pairs.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow((
            "case", "run", "variant", "left_terms", "right_terms",
            "pairs_visited", "terms_built", "index_setup_ms",
        ))
        for trial in range(3):
            for index, case in enumerate(PROFILE_CASES):
                order = ("all-pairs-profile", "indexed-profile")
                if (trial + index) % 2:
                    order = order[::-1]
                for variant in order:
                    stdout, _ = run(binaries / variant, case, "closure")
                    rows = [
                        line.split(",")[1:] for line in stdout.splitlines()
                        if line.startswith("# commutator,")
                    ]
                    if not rows:
                        raise SystemExit(f"Missing commutator counts for {variant} on {case}")
                    writer.writerows((case, trial + 1, variant, *row) for row in rows)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    patch = sub.add_parser("prepare")
    patch.add_argument("snapshot", type=Path)
    patch.add_argument("variant", choices=("eager", "all-pairs-profile", "indexed-profile"))
    timing = sub.add_parser("measure")
    timing.add_argument("binaries", type=Path)
    timing.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.snapshot, args.variant)
    else:
        measure(args.binaries, args.output)
