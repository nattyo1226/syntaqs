"""Measure paired Lie analysis runs in fresh processes on macOS or Linux."""

import argparse
import csv
from hashlib import sha256
import os
from pathlib import Path
import re
import sys
import tempfile


CASES = (
    "spin-half-1", "spin-half-2", "spin-half-4", "spin-half-2-in-8",
    "pauli-2", "pauli-2-mixed", "spin-one-dipole", "spin-one-cartesian",
    "heisenberg-2", "heisenberg-4", "heisenberg-8", "pauli-z-4",
    "fermion-2", "fermion-3", "fermion-4",
)
STAGES = ("gram", "factorize", "closure", "adjoint", "killing", "commutant", "center", "decompose")
MEMORY_CASES = ("spin-half-2", "pauli-2", "pauli-z-4", "heisenberg-8", "fermion-4")
PROFILE_CASES = ("pauli-2", "pauli-2-mixed", "heisenberg-8", "pauli-z-4", "fermion-3", "fermion-4")
SITE_CASES = (
    "heisenberg-2", "heisenberg-4", "heisenberg-8", "heisenberg-12", "heisenberg-16",
    "heisenberg-2-in-4", "heisenberg-2-in-8", "heisenberg-2-in-12", "heisenberg-2-in-16",
    "heisenberg-2-in-32", "heisenberg-2-in-64",
    "fermion-front-2", "fermion-front-4", "fermion-front-8", "fermion-front-16",
    "fermion-front-32", "fermion-front-64",
    "fermion-tail-4", "fermion-tail-8", "fermion-tail-16", "fermion-tail-32", "fermion-tail-64",
)
DIRECT_CASES = (
    "spin-half-2", "pauli-2", "pauli-2-mixed", "pauli-z-4",
    "heisenberg-8", "heisenberg-16", "heisenberg-2-in-64",
    "fermion-3", "fermion-4", "fermion-tail-64",
)
REVISIONS = {"pre": "3cadca3", "post": "c26ea49"}
MEASUREMENTS = {
    "timings": (CASES, ("all",), 5),
    "memory": (MEMORY_CASES, ("closure", "commutant", "decompose"), 3),
    "site-scaling": (SITE_CASES, ("all",), 3),
    "hotspots": (tuple(dict.fromkeys((*PROFILE_CASES, *SITE_CASES))), ("all",), 3),
}


def run(binary, case, mode):
    env = dict(os.environ, OPENBLAS_NUM_THREADS="1")
    with tempfile.TemporaryFile(mode="w+t") as output:
        pid = os.posix_spawn(
            binary, [str(binary), case, mode], env,
            file_actions=[(os.POSIX_SPAWN_DUP2, output.fileno(), 1)],
        )
        _, status, usage = os.wait4(pid, 0)
        output.seek(0)
        stdout = output.read()
    if status:
        raise SystemExit(f"{binary.name} {case} {mode}: wait status {status}\n{stdout}")
    # macOS reports bytes. Linux reports KiB.
    peak = usage.ru_maxrss * (1024 if sys.platform.startswith("linux") else 1)
    return stdout, peak


def timings(output):
    return dict(line.split(",") for line in output.splitlines() if not line.startswith("#"))


def measure_suite(binaries, directory, suite, revisions):
    cases, modes, trials = MEASUREMENTS[suite]
    profile = suite == "hotspots"
    with (directory / f"{suite}.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        if profile:
            writer.writerow((
                "revision", "case", "run", "stage", "phase", "operation", "calls",
                "elapsed_ms", "tensor_terms", "local_traces",
            ))
        else:
            writer.writerow((
                "revision", "case", "mode", "run", "D", "n",
                *(f"{stage}_ms" for stage in STAGES),
                "attempts", "coordinate_nonzero", "coordinate_tiny", "peak_rss_bytes",
            ))

        for trial in range(1, trials + 1):
            for index, case in enumerate(cases):
                builds = ("pre", "post") if (trial + index) % 2 else ("post", "pre")
                for mode in modes:
                    expected_dims = None
                    for build in builds:
                        binary = binaries / (f"{build}-profile" if profile else build)
                        output, peak = run(binary, case, mode)
                        dims = re.search(r"D=(\d+), n=(\d+)", output).groups()
                        if expected_dims is not None and dims != expected_dims:
                            raise SystemExit(f"Builds disagree on dimensions for {case}")
                        expected_dims = dims
                        if ("# hotspot," in output) != profile:
                            raise SystemExit(f"Unexpected instrumentation status for {binary}")
                        times = timings(output)
                        if profile:
                            prefix = (revisions[build], case, trial)
                            for line in output.splitlines():
                                if line.startswith("# hotspot,"):
                                    writer.writerow((*prefix, *line.split(",")[1:]))
                            for stage, elapsed in times.items():
                                writer.writerow((*prefix, stage, "root", "stage_total", 1, elapsed, 0, 0))
                        else:
                            diagnostics = re.search(
                                r"attempts=(\d+).*coordinate_nonzero=(\d+), coordinate_tiny=(\d+)",
                                output, re.S,
                            )
                            writer.writerow((
                                revisions[build], case, mode, trial, *dims,
                                *(times.get(stage, "") for stage in STAGES),
                                *(diagnostics.groups() if diagnostics else ("", "", "")), peak,
                            ))
            data.flush()
            print(f"{suite}: completed paired trial {trial}/{trials}", flush=True)


def measure_direct(binaries, directory):
    with (directory / "direct-hs.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        stages = ("gram", "closure", "decompose")
        writer.writerow(("case", "run", "build", *(f"{stage}_ms" for stage in stages)))
        for trial in range(1, 6):
            for index, case in enumerate(DIRECT_CASES):
                builds = ("post", "direct") if (trial + index) % 2 else ("direct", "post")
                for build in builds:
                    output, _ = run(binaries / f"{build}-diagnostics", case, "decompose")
                    times = timings(output)
                    writer.writerow((case, trial, build, *(times[stage] for stage in stages)))
            data.flush()
            print(f"direct-hs: completed paired trial {trial}/5", flush=True)


def closure_snapshot(binary, case):
    output, _ = run(binary, case, "closure")
    n = int(re.search(r", n=(\d+)", output)[1])
    constants = []
    checks = []
    for line in output.splitlines():
        if line.startswith("constant,"):
            constants.append(float(line.partition(",")[2]))
        elif line.startswith("check,"):
            _, left, right, measured, scale, threshold = line.split(",")
            checks.append((int(left), int(right), float(measured), float(scale), float(threshold)))
    if len(constants) != n**3 or len(checks) != n**2:
        raise SystemExit(f"Incomplete closure diagnostics from {binary} for {case}")
    return constants, checks


def compare_closure(binaries, directory, comparisons, labels):
    builds = tuple(dict.fromkeys(build for pair in comparisons for build in pair))
    with (directory / "diagnostics.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow((
            "comparison", "case", "constants", "pairs", "max_constant_difference",
            "max_residual_difference", "max_scale_difference", "max_threshold_difference",
        ))
        for case in dict.fromkeys((*CASES, *SITE_CASES)):
            snapshots = {
                build: closure_snapshot(binaries / f"{build}-diagnostics", case)
                for build in builds
            }
            for baseline, candidate in comparisons:
                left, checks_left = snapshots[baseline]
                right, checks_right = snapshots[candidate]
                if len(left) != len(right) or len(checks_left) != len(checks_right):
                    raise SystemExit(f"Builds disagree on Lie dimension for {case}")
                constant_delta = max(abs(a - b) for a, b in zip(left, right))
                check_deltas = [0.0] * 3
                for a, b in zip(checks_left, checks_right):
                    if a[:2] != b[:2]:
                        raise SystemExit(f"Builds disagree on bracket pair order for {case}")
                    for i, (x, y) in enumerate(zip(a[2:], b[2:])):
                        check_deltas[i] = max(check_deltas[i], abs(x - y))
                writer.writerow((
                    f"{labels.get(baseline, baseline)}/{labels.get(candidate, candidate)}",
                    case, len(left), len(checks_left),
                    constant_delta, *check_deltas,
                ))
    print("diagnostics: compared all distinct workloads", flush=True)


def main():
    suites = (*MEASUREMENTS, "direct-hs", "diagnostics")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binaries", type=Path, help="directory created by build.sh")
    parser.add_argument("directory", type=Path, help="new directory for CSV output")
    parser.add_argument("--suite", action="append", choices=suites, help="run one or more named suites")
    parser.add_argument(
        "--revisions", nargs=2, metavar=("BASELINE", "CANDIDATE"),
        help="compare only pre and post, using these source labels",
    )
    args = parser.parse_args()
    if sys.platform != "darwin" and not sys.platform.startswith("linux"):
        parser.error("peak RSS units are defined for macOS and Linux")

    binaries = args.binaries.resolve()
    comparisons = (("pre", "post"),)
    labels = {}
    if args.revisions is None:
        comparisons += (("post", "direct"),)
    else:
        labels = dict(zip(("pre", "post"), args.revisions))
        if args.suite and "direct-hs" in args.suite:
            parser.error("--revisions compares two builds and cannot run the historical direct-hs trial")
        suites = tuple(suite for suite in suites if suite != "direct-hs")
    selected = args.suite or suites

    names = set()
    if "diagnostics" in selected:
        names.update(build for pair in comparisons for build in pair)
    if "direct-hs" in selected:
        names.update(("post", "direct"))
    if names:
        digests = {sha256((binaries / f"{name}-diagnostics").read_bytes()).digest() for name in names}
        if len(digests) != len(names):
            parser.error("each diagnostics build must use a distinct executable")

    args.directory.mkdir(parents=True)
    for suite in selected:
        if suite == "direct-hs":
            measure_direct(binaries, args.directory)
        elif suite == "diagnostics":
            compare_closure(binaries, args.directory, comparisons, labels)
        else:
            measure_suite(binaries, args.directory, suite, labels or REVISIONS)


if __name__ == "__main__":
    main()
