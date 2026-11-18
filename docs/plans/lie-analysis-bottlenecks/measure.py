"""Collect stage timings and process peak RSS on macOS or Linux."""

import argparse
import csv
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
STAGES = "gram factorize closure adjoint killing commutant center decompose".split()
MEMORY_CASES = ("spin-half-2", "pauli-2", "pauli-z-4", "heisenberg-8", "fermion-4")
PROFILE_CASES = ("pauli-2", "pauli-2-mixed", "heisenberg-8", "pauli-z-4", "fermion-3", "fermion-4")
PROFILE_STAGES = {"closure", "commutant", "decompose"}
SITE_CASES = (
    "heisenberg-2", "heisenberg-4", "heisenberg-8", "heisenberg-12", "heisenberg-16",
    "heisenberg-2-in-4", "heisenberg-2-in-8", "heisenberg-2-in-12", "heisenberg-2-in-16",
    "heisenberg-2-in-32", "heisenberg-2-in-64",
    "fermion-front-2", "fermion-front-4", "fermion-front-8", "fermion-front-16",
    "fermion-front-32", "fermion-front-64",
    "fermion-tail-4", "fermion-tail-8", "fermion-tail-16", "fermion-tail-32", "fermion-tail-64",
)
SITE_PROFILE_METRICS = (
    ("rhs_ms", "closure", "rhs", "phase_total", "ms"),
    ("rhs_calls", "closure", "rhs", "hs_contract", "calls"),
    ("rhs_intrinsic_terms", "closure", "rhs", "collect_reduce", "terms"),
    ("rhs_collect_left_ms", "closure", "rhs", "hs_collect_left", "ms"),
    ("rhs_collect_right_ms", "closure", "rhs", "hs_collect_right", "ms"),
    ("rhs_collect_reduce_ms", "closure", "rhs", "collect_reduce", "ms"),
    ("rhs_merge_ms", "closure", "rhs", "collect_merge", "ms"),
    ("rhs_rebuild_ms", "closure", "rhs", "collect_rebuild", "ms"),
    ("rhs_contract_ms", "closure", "rhs", "hs_contract", "ms"),
    ("residual_norm_ms", "closure", "residual_norm", "phase_total", "ms"),
    ("closure_trace_terms", "closure", None, "trace_walk", "terms"),
    ("closure_graded_lower_ms", "closure", None, "trace_graded_lower", "ms"),
    ("closure_parity_insertions", "closure", None, "trace_parity_insertions", "calls"),
    ("decompose_center_ms", "decompose", "center_split", "phase_total", "ms"),
    ("component_check_ms", "decompose", "component_orthonormality", "phase_total", "ms"),
    ("component_collect_left_ms", "decompose", "component_orthonormality", "hs_collect_left", "ms"),
    ("component_collect_right_ms", "decompose", "component_orthonormality", "hs_collect_right", "ms"),
    ("component_trace_local_ms", "decompose", "component_orthonormality", "trace_local_reduce", "ms"),
    ("component_trace_terms", "decompose", "component_orthonormality", "trace_walk", "terms"),
    ("component_graded_enumerate_ms", "decompose", "component_orthonormality", "trace_graded_enumerate", "ms"),
    ("component_graded_lower_ms", "decompose", "component_orthonormality", "trace_graded_lower", "ms"),
    ("component_parity_insertions", "decompose", "component_orthonormality", "trace_parity_insertions", "calls"),
)


def run(binary, case, mode="all"):
    env = dict(os.environ, OPENBLAS_NUM_THREADS="1")
    with tempfile.TemporaryFile(mode="w+t") as output:
        pid = os.posix_spawn(
            binary, [binary, case, mode], env,
            file_actions=[(os.POSIX_SPAWN_DUP2, output.fileno(), 1)],
        )
        _, status, usage = os.wait4(pid, 0)
        output.seek(0)
        stdout = output.read()
    if status:
        raise SystemExit(f"{case} {mode}: process exited with wait status {status}\n{stdout}")
    # macOS reports bytes. Linux reports KiB.
    peak = usage.ru_maxrss * (1024 if sys.platform.startswith("linux") else 1)
    return stdout, peak


def measure(binary, directory):
    with (directory / "timings.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow([
            "case", "run", "D", "n", *[f"{stage}_ms" for stage in STAGES],
            "attempts", "coordinate_nonzero", "coordinate_tiny", "peak_rss_bytes",
        ])
        for trial in range(1, 6):
            for case in CASES:
                output, peak = run(binary, case)
                dims = re.search(r"D=(\d+), n=(\d+)", output).groups()
                attempts = re.search(r"attempts=(\d+)", output)[1]
                coords = re.search(r"coordinate_nonzero=(\d+), coordinate_tiny=(\d+)", output).groups()
                times = dict(line.split(",") for line in output.splitlines() if not line.startswith("#"))
                writer.writerow([case, trial, *dims, *[times[s] for s in STAGES], attempts, *coords, peak])
            data.flush()
            print(f"Completed timing trial {trial}", flush=True)

    with (directory / "memory.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow(["case", "mode", "run", "peak_rss_bytes"])
        for case in MEMORY_CASES:
            for mode in ("closure", "commutant", "decompose"):
                for trial in range(1, 4):
                    _, peak = run(binary, case, mode)
                    writer.writerow([case, mode, trial, peak])
            data.flush()
            print(f"Completed RSS measurements for {case}", flush=True)


def profile(binary, directory):
    with (directory / "profile.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow(["case", "run", "stage", "operation", "calls", "elapsed_ms"])
        for trial in range(1, 4):
            for case in PROFILE_CASES:
                output, _ = run(binary, case)
                if "# profile," not in output:
                    raise SystemExit("Profiling requires the instrumented binary from profile.patch")
                for line in output.splitlines():
                    if line.startswith("# profile,"):
                        _, stage, operation, calls, elapsed_ms = line.split(",")
                    elif not line.startswith("#"):
                        stage, elapsed_ms = line.split(",")
                        operation, calls = "stage_total", 1
                    else:
                        continue
                    if stage in PROFILE_STAGES:
                        writer.writerow([case, trial, stage, operation, calls, elapsed_ms])
            data.flush()
            print(f"Completed profiling trial {trial}", flush=True)


def measure_hotspots(binary, directory, baseline):
    with (directory / "hotspots.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow([
            "case", "run", "build", "stage", "phase", "operation", "calls",
            "elapsed_ms", "tensor_terms", "local_traces",
        ])
        for trial in range(1, 4):
            for case in PROFILE_CASES:
                for build, executable in (("baseline", baseline), ("instrumented", binary)):
                    output, _ = run(executable, case)
                    if ("# hotspot," in output) != (build == "instrumented"):
                        raise SystemExit(f"Unexpected hotspot output from {build} executable: {executable}")
                    for line in output.splitlines():
                        if line.startswith("# hotspot,"):
                            _, stage, phase, operation, calls, elapsed_ms, terms, traces = line.split(",")
                        elif not line.startswith("#"):
                            stage, elapsed_ms = line.split(",")
                            phase, operation, calls, terms, traces = "root", "stage_total", 1, 0, 0
                        else:
                            continue
                        if stage in PROFILE_STAGES:
                            writer.writerow([case, trial, build, stage, phase, operation, calls,
                                             elapsed_ms, terms, traces])
                data.flush()
            print(f"Completed hotspot trial {trial}", flush=True)


def site_profile(output):
    samples = []
    for line in output.splitlines():
        if line.startswith("# hotspot,"):
            _, stage, phase, operation, calls, elapsed_ms, terms, _ = line.split(",")
            samples.append((stage, phase, operation, {
                "calls": int(calls), "ms": float(elapsed_ms), "terms": int(terms),
            }))
    if not samples:
        raise SystemExit("Site scaling requires the instrumented binary from hotspots.patch")
    values = []
    for _, stage, phase, operation, field in SITE_PROFILE_METRICS:
        matches = [sample[field] for row_stage, row_phase, row_operation, sample in samples
                   if stage == row_stage and operation == row_operation
                   and (phase is None or phase == row_phase)]
        if not matches and operation not in {"trace_graded_enumerate", "trace_graded_lower", "trace_parity_insertions"}:
            raise SystemExit(f"Missing site profile metric {stage}, {phase}, {operation}")
        value = sum(matches)
        values.append(f"{value:.6f}" if field == "ms" else value)
    return values


def measure_site_scaling(binary, directory, baseline):
    stages = ("gram", "factorize", "closure", "commutant", "center", "decompose")
    with (directory / "site-scaling.csv").open("w", newline="") as data:
        writer = csv.writer(data, lineterminator="\n")
        writer.writerow([
            "case", "run", "sites", "D", "n", *[f"{stage}_ms" for stage in stages],
            "peak_rss_bytes", "profile_closure_ms", "profile_decompose_ms",
            *[name for name, *_ in SITE_PROFILE_METRICS],
        ])
        for trial in range(1, 4):
            for case in SITE_CASES:
                reference, peak = run(baseline, case)
                if "# hotspot," in reference:
                    raise SystemExit("Site scaling baseline must be the uninstrumented binary")
                output, _ = run(binary, case)
                dims = re.search(r"D=(\d+), n=(\d+)", reference).groups()
                if dims != re.search(r"D=(\d+), n=(\d+)", output).groups():
                    raise SystemExit(f"Benchmark builds disagree on dimensions for {case}")
                times = dict(line.split(",") for line in reference.splitlines() if not line.startswith("#"))
                profile_times = dict(line.split(",") for line in output.splitlines() if not line.startswith("#"))
                writer.writerow([
                    case, trial, case.rsplit("-", 1)[1], *dims, *[times[stage] for stage in stages],
                    peak, profile_times["closure"], profile_times["decompose"], *site_profile(output),
                ])
                data.flush()
            print(f"Completed site-scaling trial {trial}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("directory", type=Path, help="output directory for CSV files")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--profile", action="store_true")
    mode.add_argument("--hotspots", type=Path, metavar="BASELINE", help="measure alongside the baseline executable")
    mode.add_argument("--site-scaling", type=Path, metavar="BASELINE", help="measure fixed-dimension workloads across site counts")
    args = parser.parse_args()
    if sys.platform != "darwin" and not sys.platform.startswith("linux"):
        parser.error("peak RSS units are defined for macOS and Linux")
    args.directory.mkdir(parents=True, exist_ok=True)
    binary = str(args.binary.resolve())
    if args.site_scaling is not None:
        measure_site_scaling(binary, args.directory, str(args.site_scaling.resolve()))
    elif args.hotspots is not None:
        measure_hotspots(binary, args.directory, str(args.hotspots.resolve()))
    elif args.profile:
        profile(binary, args.directory)
    else:
        measure(binary, args.directory)
