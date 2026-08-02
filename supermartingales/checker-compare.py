#!/usr/bin/env python3
"""Checker-protocol comparability cells for the Section 1 timing table.

The timing table's cells were measured under three distinct protocols
(in-process check inside checked production; one external checker
process per repeat over one shared container; one external checker
process over a whole batch of per-obligation containers). This driver
measures the deltas between those protocols directly, so that public
claims can compare cells with the protocol difference as a measured
number rather than a caveat.

All child processes run under `nice -n 19` (the campaign convention).
Every wall is a monotonic clock around child spawn-to-exit — "process
wall" semantics, deliberately including process startup, so that the
per-invocation and amortized cells differ only in how many containers
one process consumes.

Subcommands (receipts land in the directory this file names in OUT):

  external-shared     per-invocation (1 file/process) and amortized
                      (same file passed k times to one process) cells
                      over the shared incumbent containers
  external-perrow     the same checker binary over the pre-migration
                      per-obligation container batches (one process
                      per batch), plus a single-file cell on the
                      smallest batch member
  retain              two full-roster checked-production campaigns,
                      identical but for --retain-containers
  cold                fresh-process single-repeat productions of one
                      benchmark (whole process wall vs receipt row wall)
  batch-amortized     the checker's own timing decomposition
                      (--timings): per-process derivation measured
                      directly (not inferred from a k-slope) beside
                      per-item marginal walls, homogeneous (one
                      container x10/process) and whole-roster (all
                      three containers, one process) cells

Orchestration only: the measured work happens in the pinned binaries.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent  # repository root
OUT = ROOT / "supermartingales/out/glc-baked/sec1/gb10-checker-compare"
INCUMBENTS = ROOT / "supermartingales/out/glc-baked/sec1/incumbents"
PERROW = ROOT / "supermartingales/out/glc/sec1"
BENCHMARKS = ["faulty_marbles", "faulty_regions", "faulty_unreliable"]
SCHEMA = "maitria.supermart.checker-compare.v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_rev(path: Path) -> str:
    return subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()


def environment() -> dict:
    return {
        "loadavg": list(os.getloadavg()),
        "nproc": os.cpu_count(),
        "nice": 19,
        "utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }


def run_timed(argv: list[str]) -> tuple[float, subprocess.CompletedProcess]:
    """Monotonic wall around child spawn-to-exit, niced."""
    cmd = ["nice", "-n", "19", *argv]
    t0 = time.monotonic()
    proc = subprocess.run(cmd, capture_output=True, text=True)
    wall = time.monotonic() - t0
    return wall, proc


def checker_accepts(proc: subprocess.CompletedProcess, expect_lines: int) -> bool:
    lines = [l for l in proc.stdout.splitlines() if l.strip()]
    return (
        proc.returncode == 0
        and len(lines) == expect_lines
        and all(" ACCEPT " in l for l in lines)
    )


def base_receipt(args, protocol: str) -> dict:
    return {
        "receipt_version": 1,
        "schema": SCHEMA,
        "section": "sec1",
        "machine": args.machine,
        "protocol": protocol,
        "wall_semantics": "monotonic clock around child process spawn-to-exit (includes process startup)",
        "checker_binary": {
            "path": args.glc_check.name,
            "sha256": sha256(args.glc_check),
        },
        "qtsl_revision": args.qtsl_rev,
        "qtsl_experiments_revision": git_rev(ROOT),
        "environment_start": environment(),
    }


def write_receipt(path: Path, receipt: dict) -> None:
    receipt["environment_end"] = environment()
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(receipt, indent=1, sort_keys=False) + "\n")
    tmp.replace(path)
    print(f"wrote {path}")


def cell_external_shared(args) -> None:
    receipt = base_receipt(
        args,
        "per benchmark: 10 measured repeats at 1 file/invocation, then 10 measured repeats "
        "with the same shared container passed 10 times to one invocation; no warmup "
        "(matching the geolog-shared receipt's convention); serial, one child at a time",
    )
    receipt["cells"] = []
    for bench in BENCHMARKS:
        container = INCUMBENTS / bench / f"{bench}.glc"
        digest = sha256(container)
        for k in (1, 10):
            rows = []
            for repeat in range(10):
                wall, proc = run_timed([str(args.glc_check), *([str(container)] * k)])
                ok = checker_accepts(proc, expect_lines=k)
                rows.append({"repeat": repeat, "wall_s": wall, "accept": ok})
                if not ok:
                    print(proc.stdout, proc.stderr, file=sys.stderr)
                    sys.exit(f"{bench} k={k} repeat={repeat}: checker did not accept")
            receipt["cells"].append({
                "cell": "external-shared",
                "bench": bench,
                "container_sha256": digest,
                "files_per_invocation": k,
                "rows": rows,
                "median_wall_s": statistics.median(r["wall_s"] for r in rows),
            })
    write_receipt(OUT / "external-shared.json", receipt)


def cell_external_perrow(args) -> None:
    receipt = base_receipt(
        args,
        "per benchmark: 10 measured repeats of one checker invocation over the whole "
        "pre-migration per-obligation container batch (the batch the legacy group-wall "
        "receipts timed), same (current) checker binary; plus 10 repeats at 1 file/invocation "
        "on the batch's first container; no warmup; serial",
    )
    receipt["cells"] = []
    for bench in BENCHMARKS:
        batch = sorted((PERROW / bench / "rows").glob("*.glc"))
        if not batch:
            sys.exit(f"{bench}: no per-obligation containers found under {PERROW}")
        roster = [{"file": p.name, "sha256": sha256(p)} for p in batch]
        rows = []
        for repeat in range(10):
            wall, proc = run_timed([str(args.glc_check), *map(str, batch)])
            ok = checker_accepts(proc, expect_lines=len(batch))
            rows.append({"repeat": repeat, "wall_s": wall, "accept": ok})
            if not ok:
                print(proc.stdout, proc.stderr, file=sys.stderr)
                sys.exit(f"{bench} batch repeat={repeat}: checker did not accept")
        receipt["cells"].append({
            "cell": "external-perrow-batch",
            "bench": bench,
            "n_containers": len(batch),
            "containers": roster,
            "rows": rows,
            "median_wall_s": statistics.median(r["wall_s"] for r in rows),
        })
        single = batch[0]
        rows = []
        for repeat in range(10):
            wall, proc = run_timed([str(args.glc_check), str(single)])
            ok = checker_accepts(proc, expect_lines=1)
            rows.append({"repeat": repeat, "wall_s": wall, "accept": ok})
            if not ok:
                sys.exit(f"{bench} single repeat={repeat}: checker did not accept")
        receipt["cells"].append({
            "cell": "external-perrow-single",
            "bench": bench,
            "container": {"file": single.name, "sha256": sha256(single)},
            "rows": rows,
            "median_wall_s": statistics.median(r["wall_s"] for r in rows),
        })
    write_receipt(OUT / "external-perrow-bridge.json", receipt)


def run_campaign(args, retain: Path | None, out_json: Path) -> tuple[float, dict]:
    argv = [
        str(args.runner), "retime",
        "--experiments", str(ROOT),
        "--machine", args.machine,
        "--out", str(out_json),
        "--compare", str(INCUMBENTS),
        "--lane", "host",
        "--warmups", "1",
        "--repeats", "10",
    ]
    if retain is not None:
        argv += ["--retain-containers", str(retain)]
    wall, proc = run_timed(argv)
    if proc.returncode != 0:
        print(proc.stdout[-4000:], proc.stderr[-4000:], file=sys.stderr)
        sys.exit(f"campaign failed (retain={retain is not None})")
    return wall, json.loads(out_json.read_text())


def summarize_rows(campaign: dict) -> dict:
    rows = [r for r in campaign["rows"] if not r["warmup"]]
    return {
        "binary_sha256": campaign["binary_sha256"],
        "mtk_revision": campaign["mtk_revision"],
        "rows": len(campaign["rows"]),
        "measured_rows": len(rows),
        "all_byte_identical": all(r["byte_identical"] for r in campaign["rows"]),
        "sum_row_wall_s": sum(r["wall_s"] for r in campaign["rows"]),
        "sum_measured_row_wall_s": sum(r["wall_s"] for r in rows),
        "median_wall_s_by_bench": {
            b: statistics.median(r["wall_s"] for r in rows if r["bench"] == b)
            for b in BENCHMARKS
        },
        "median_check_s_by_bench": {
            b: statistics.median(r["check_s"] for r in rows if r["bench"] == b)
            for b in BENCHMARKS
        },
    }


def cell_retain(args) -> None:
    receipt = base_receipt(
        args,
        "two full-roster checked-production campaigns (1 flagged warmup + 10 measured "
        "repeats per benchmark), identical but for --retain-containers; arm order "
        "retain-first then no-retain; outer wall is the monotonic process wall of the "
        "whole campaign invocation; retained copies land on the same filesystem as the "
        "repository and are deleted after measurement (byte-identical to the committed "
        "incumbents, re-verified by the runner's own gates on every row)",
    )
    receipt["runner_binary"] = {"path": args.runner.name, "sha256": sha256(args.runner)}
    scratch = OUT / "retain-scratch"
    if scratch.exists():
        shutil.rmtree(scratch)
    with tempfile.TemporaryDirectory(dir=OUT) as td:
        wall_r, campaign_r = run_campaign(args, scratch, Path(td) / "retain.json")
        wall_n, campaign_n = run_campaign(args, None, Path(td) / "noretain.json")
    retained = sorted(p.relative_to(scratch).as_posix() for p in scratch.rglob("*.glc"))
    retained_bytes = sum((scratch / p).stat().st_size for p in retained)
    shutil.rmtree(scratch)
    receipt["arms"] = {
        "retain": {"outer_wall_s": wall_r, "retained_files": len(retained),
                    "retained_bytes": retained_bytes, **summarize_rows(campaign_r)},
        "no_retain": {"outer_wall_s": wall_n, **summarize_rows(campaign_n)},
    }
    receipt["outer_wall_delta_s"] = wall_r - wall_n
    write_receipt(OUT / "retain-vs-noretain.json", receipt)


def cell_cold(args) -> None:
    bench = "faulty_marbles"
    receipt = base_receipt(
        args,
        f"10 fresh-process single-repeat productions of {bench} (--warmups 0 --repeats 1): "
        "whole process wall (monotonic, spawn-to-exit) beside the receipt's one row wall; "
        "the cold-process-per-repeat calibration anchor",
    )
    receipt["runner_binary"] = {"path": args.runner.name, "sha256": sha256(args.runner)}
    if args.max_load_1min is not None:
        receipt["load_gate_1min"] = args.max_load_1min
    rows = []
    with tempfile.TemporaryDirectory(dir=OUT) as td:
        for repeat in range(10):
            out_json = Path(td) / f"cold-{repeat}.json"
            gate = ([] if args.max_load_1min is None
                    else ["--max-load-1min", str(args.max_load_1min)])
            wall, proc = run_timed([
                str(args.runner), "retime",
                "--experiments", str(ROOT),
                "--machine", args.machine,
                "--out", str(out_json),
                "--compare", str(INCUMBENTS),
                "--lane", "host",
                "--warmups", "0",
                "--repeats", "1",
                "--bench", bench,
                *gate,
            ])
            if proc.returncode != 0:
                print(proc.stdout[-4000:], proc.stderr[-4000:], file=sys.stderr)
                sys.exit(f"cold repeat={repeat} failed (exit {proc.returncode}; "
                         "3 = quiet gate refusal, no receipt written)")
            campaign = json.loads(out_json.read_text())
            [row] = campaign["rows"]
            row_out = {
                "repeat": repeat,
                "process_wall_s": wall,
                "row_wall_s": row["wall_s"],
                "produce_s": row["produce_s"],
                "check_s": row["check_s"],
                "seal_s": row["seal_s"],
                "byte_identical": row["byte_identical"],
            }
            lc = campaign.get("load_context")
            if lc is not None:
                row_out["loadavg_start_1min"] = lc["loadavg_start"][0]
            rows.append(row_out)
    receipt["bench"] = bench
    receipt["rows"] = rows
    receipt["median_process_wall_s"] = statistics.median(r["process_wall_s"] for r in rows)
    receipt["median_row_wall_s"] = statistics.median(r["row_wall_s"] for r in rows)
    write_receipt(OUT / "cold-anchor.json", receipt)


def read_timings(path: Path, expect_files: int) -> dict:
    """Parse one glc-check --timings receipt; enforce its shape."""
    t = json.loads(path.read_text())
    assert t["schema"] == "maitria.glc-check.timings.v1", t["schema"]
    assert len(t["files"]) == expect_files, (len(t["files"]), expect_files)
    assert all(f["verdict"] == "ACCEPT" for f in t["files"])
    assert len(t["kit_derivations"]) == 1, "one closure revision aboard"
    return t


def cell_batch_amortized(args) -> None:
    receipt = base_receipt(
        args,
        "checker self-timing (--timings, monotonic in-process clock): per benchmark, "
        "10 process-repeats of one invocation with the shared container passed 10 times "
        "(homogeneous cell), then 10 process-repeats of one invocation over all three "
        "shared containers (roster cell); every batch verdict line asserted byte-identical "
        "to the single-invocation reference line (conformance in the field: cost may "
        "change, verdicts may not); per-file walls exclude the first-contact baked "
        "vocabulary derivation, which is reported separately per process; no warmup; "
        "serial, one child at a time",
    )
    receipt["checker_timings_schema"] = "maitria.glc-check.timings.v1"
    containers = {b: INCUMBENTS / b / f"{b}.glc" for b in BENCHMARKS}
    digests = {b: sha256(p) for b, p in containers.items()}

    # Single-invocation reference verdict lines (the conformance oracle).
    refline = {}
    for bench, cpath in containers.items():
        wall, proc = run_timed([str(args.glc_check), str(cpath)])
        ok = checker_accepts(proc, expect_lines=1)
        if not ok:
            print(proc.stdout, proc.stderr, file=sys.stderr)
            sys.exit(f"{bench}: single-invocation reference did not accept")
        refline[bench] = proc.stdout

    receipt["cells"] = []
    with tempfile.TemporaryDirectory() as td:
        tf = Path(td) / "timings.json"

        for bench, cpath in containers.items():
            k = 10
            rows = []
            for repeat in range(10):
                wall, proc = run_timed([
                    str(args.glc_check), "--timings", str(tf), *([str(cpath)] * k),
                ])
                if proc.returncode != 0 or proc.stdout != refline[bench] * k:
                    print(proc.stdout, proc.stderr, file=sys.stderr)
                    sys.exit(f"{bench} homogeneous repeat={repeat}: batch verdicts "
                             "diverged from the single-invocation reference")
                t = read_timings(tf, expect_files=k)
                item_walls = [f["wall_s"] for f in t["files"]]
                rows.append({
                    "repeat": repeat,
                    "process_wall_s": wall,
                    "startup_s": t["startup_s"],
                    "derivation_s": t["kit_derivations"][0]["wall_s"],
                    "derivation_pin": t["kit_derivations"][0]["pin"],
                    "item_walls_s": item_walls,
                    "median_item_wall_s": statistics.median(item_walls),
                    "checker_total_wall_s": t["total_wall_s"],
                })
            pooled = [w for r in rows for w in r["item_walls_s"]]
            receipt["cells"].append({
                "cell": "batch-amortized-homogeneous",
                "bench": bench,
                "container_sha256": digests[bench],
                "files_per_invocation": k,
                "rows": rows,
                "median_derivation_s": statistics.median(r["derivation_s"] for r in rows),
                "median_startup_s": statistics.median(r["startup_s"] for r in rows),
                "median_item_wall_s_pooled": statistics.median(pooled),
                "median_process_wall_s": statistics.median(r["process_wall_s"] for r in rows),
            })

        roster = [str(containers[b]) for b in BENCHMARKS]
        ref_concat = "".join(refline[b] for b in BENCHMARKS)
        rows = []
        for repeat in range(10):
            wall, proc = run_timed([str(args.glc_check), "--timings", str(tf), *roster])
            if proc.returncode != 0 or proc.stdout != ref_concat:
                print(proc.stdout, proc.stderr, file=sys.stderr)
                sys.exit(f"roster repeat={repeat}: batch verdicts diverged from the "
                         "single-invocation references")
            t = read_timings(tf, expect_files=len(BENCHMARKS))
            rows.append({
                "repeat": repeat,
                "process_wall_s": wall,
                "startup_s": t["startup_s"],
                "derivation_s": t["kit_derivations"][0]["wall_s"],
                "derivation_pin": t["kit_derivations"][0]["pin"],
                "item_wall_s": {
                    b: f["wall_s"] for b, f in zip(BENCHMARKS, t["files"])
                },
                "checker_total_wall_s": t["total_wall_s"],
            })
        receipt["cells"].append({
            "cell": "batch-amortized-roster",
            "benchmarks": BENCHMARKS,
            "container_sha256": digests,
            "rows": rows,
            "median_derivation_s": statistics.median(r["derivation_s"] for r in rows),
            "median_startup_s": statistics.median(r["startup_s"] for r in rows),
            "median_item_wall_s": {
                b: statistics.median(r["item_wall_s"][b] for r in rows)
                for b in BENCHMARKS
            },
            "median_process_wall_s": statistics.median(r["process_wall_s"] for r in rows),
        })
    write_receipt(OUT / "batch-amortized.json", receipt)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cell", choices=["external-shared", "external-perrow", "retain", "cold",
                                         "batch-amortized"])
    parser.add_argument("--machine", default="gb10")
    parser.add_argument("--glc-check", type=Path, default=None,
                        help="path to a built glc-check binary (hashed into every receipt)")
    parser.add_argument("--runner", type=Path, default=None,
                        help="supermart-sec1 binary (retain/cold cells)")
    parser.add_argument("--max-load-1min", type=float, default=None,
                        help="quiet-conditions gate passed through to each fresh runner "
                             "process (cold cell): the runner refuses (exit 3, no receipt) "
                             "when the 1-minute load average exceeds this at spawn")
    parser.add_argument("--out-dir", type=Path, default=None,
                        help="receipt directory (default: the gb10 cell family)")
    parser.add_argument("--qtsl-rev", default=None,
                        help="qtsl source revision the checker binary was built from "
                             "(recorded in the receipt)")
    args = parser.parse_args()
    if args.out_dir is not None:
        globals()["OUT"] = args.out_dir
    if args.glc_check is None:
        sys.exit("--glc-check is required (path to a built glc-check binary)")
    if args.cell in ("retain", "cold") and args.runner is None:
        sys.exit("--runner is required for the retain and cold cells")
    {
        "external-shared": cell_external_shared,
        "external-perrow": cell_external_perrow,
        "retain": cell_retain,
        "cold": cell_cold,
        "batch-amortized": cell_batch_amortized,
    }[args.cell](args)


if __name__ == "__main__":
    main()
