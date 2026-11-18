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


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("directory", type=Path, help="output directory for CSV files")
    parser.add_argument("--profile", action="store_true")
    args = parser.parse_args()
    if sys.platform != "darwin" and not sys.platform.startswith("linux"):
        parser.error("peak RSS units are defined for macOS and Linux")
    args.directory.mkdir(parents=True, exist_ok=True)
    collector = profile if args.profile else measure
    collector(str(args.binary.resolve()), args.directory)
