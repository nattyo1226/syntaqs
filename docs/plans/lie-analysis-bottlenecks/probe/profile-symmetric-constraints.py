"""Add symmetric construction and SVD timers to an isolated benchmark snapshot."""

import argparse
from pathlib import Path


def replace_once(source, old, new):
    if source.count(old) != 1:
        raise SystemExit(f"Expected one occurrence of {old!r}")
    return source.replace(old, new)


def instrument(snapshot):
    invariant = snapshot / "crates/syntaqs-lie/src/invariant.rs"
    source = invariant.read_text()
    prefix, separator, body = source.partition("pub(crate) fn symmetric_commutant<R>(")
    if not separator:
        raise SystemExit("Missing symmetric commutant")
    begin = "    let pairs: Vec<_>"
    body = replace_once(body, begin, "    let start = syntaqs_core::hotspot::Instant::now();\n" + begin)
    end = "    let analysis = rank_nullspace(&constraints, options)?;"
    body = replace_once(
        body, end,
        '    syntaqs_core::hotspot::observe("symmetric_constraints_build", start.elapsed(), 1, 0, 0);\n' + end,
    )
    full = "    let full = commutant_constraints(orthonormal_adjoints, n);\n"
    if full in body:
        body = replace_once(
            body, full,
            full + '    syntaqs_core::hotspot::observe("symmetric_full_intermediate", std::time::Duration::ZERO, 1, 0, 0);\n',
        )

    kernels = snapshot / "crates/syntaqs-lie/src/numerical/kernels.rs"
    source = replace_once(
        kernels.read_text(),
        "    let (_, singular_values, vt) = matrix.svd(false, true)?;",
        '''    let (_, singular_values, vt) =
        syntaqs_core::hotspot::measure("constraint_svd", || matrix.svd(false, true))?;''',
    )
    invariant.write_text(prefix + separator + body)
    kernels.write_text(source)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("snapshot", type=Path)
    instrument(parser.parse_args().snapshot)
