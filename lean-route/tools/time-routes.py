#!/usr/bin/env python3
"""time-routes.py — per-certificate end-to-end timing receipts for the
two production checking routes on the lean-route ladder, in the same
receipt discipline as out/results.json (host identity embedded in every
row; measurements are replaced, never blended).

Routes:

  qtslab   the certified checker binary: container on stdin, verdict on
           stdout ("s QTSLAB ACCEPT ..." / "VERDICT UNSUPPORTED ..." /
           "VERDICT UNREADABLE ...", always exit 0).  Containers:
           out/qtslab/ (the content-identical content-resolved mints;
           tools/mint-qtslab-vintage.py).
  glc      the fast-path engine checker (glc-check from the pinned qtsl
           checkout): container path as argv, ACCEPT line on stdout.
           Containers: out/glc/ (the staged-chain lane; GLC-CERT.md).

Usage:
  time-routes.py --route qtslab --bin <qtslab> --containers <dir> \
      --out <receipts.json> [--budget 600] [--reps 3] [--note "..."]

Protocol: serial, one process per pass, end-to-end wall clock (spawn to
exit) — the same "end-to-end" the Lean-route receipts time.  --pin runs
every checked process on one named CPU (the one-Arm-core protocol of
the certified-checker replay census; also removes the big.LITTLE core
lottery on asymmetric hosts).  Pass 1
always runs; passes 2..reps run only after an accept (refusals and
budget-cuts are recorded once).  Every row carries the pass's 1-minute
load average; the box is shared, and the section's cross-machine note
prices what timing spread cannot change.
"""

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path


def host_identity() -> dict:
    cpu = ""
    try:
        out = subprocess.run(["lscpu"], capture_output=True, text=True).stdout
        names = [l.split(":", 1)[1].strip() for l in out.splitlines()
                 if l.startswith("Model name")]
        cpu = " + ".join(dict.fromkeys(names))  # big.LITTLE: both, deduped
    except OSError:
        pass
    if not cpu:
        cpu = platform.processor() or platform.machine()
    mem_gib = 0.0
    try:
        for line in Path("/proc/meminfo").read_text().splitlines():
            if line.startswith("MemTotal:"):
                mem_gib = int(line.split()[1]) / 2**20
                break
    except OSError:
        pass
    return {
        "host": platform.node(),
        "cpu": cpu,
        "threads": os.cpu_count() or 0,
        "mem_gib": round(mem_gib, 1),
        "arch": platform.machine(),
    }


def classify_qtslab(stdout: str) -> tuple[str, str]:
    line = stdout.splitlines()[0].strip() if stdout.strip() else ""
    if line.startswith("s QTSLAB ACCEPT"):
        return "accept", line[:80]
    if line.startswith("VERDICT UNSUPPORTED"):
        return "unsupported", line
    if line.startswith("VERDICT"):
        return "refuse", line
    return "error", line or "<no output>"


def classify_glc(stdout: str, rc: int) -> tuple[str, str]:
    line = stdout.splitlines()[0].strip() if stdout.strip() else ""
    if rc == 0 and "ACCEPT" in line:
        return "accept", line[:120]
    if line.startswith("REFUSE") or "REFUSE" in line:
        return "refuse", line[:160]
    return "error", (line or f"<exit {rc}>")[:160]


def run_one(route: str, binary: Path, container: Path, budget: float,
            scratch: Path, pin: int | None):
    argv = [str(binary)]
    if route == "glc":
        argv.append(str(container))
    load1 = os.getloadavg()[0]
    pre = (lambda: os.sched_setaffinity(0, {pin})) if pin is not None else None
    with open(container, "rb") if route == "qtslab" else open(os.devnull, "rb") as fin, \
         open(scratch, "w+b") as fout:
        t0 = time.perf_counter()
        proc = subprocess.Popen(argv, stdin=fin, stdout=fout,
                                stderr=subprocess.DEVNULL, preexec_fn=pre)
        deadline = t0 + budget
        timed_out = False
        while True:
            wpid, status, rusage = os.wait4(proc.pid, os.WNOHANG)
            if wpid == proc.pid:
                break
            if time.perf_counter() > deadline:
                proc.kill()
                _, status, rusage = os.wait4(proc.pid, 0)
                timed_out = True
                break
            time.sleep(0.001 if time.perf_counter() - t0 < 1 else 0.05)
        t1 = time.perf_counter()
        proc.returncode = os.waitstatus_to_exitcode(status)  # keep Popen quiet
    stdout = scratch.read_text(errors="replace")
    if timed_out:
        return {"outcome": "timeout", "t_e2e_ms": None, "verdict": None,
                "maxrss_kb": rusage.ru_maxrss, "load1": round(load1, 1)}
    kind, verdict = (classify_qtslab(stdout) if route == "qtslab"
                     else classify_glc(stdout, proc.returncode))
    return {
        "outcome": kind,
        "t_e2e_ms": round((t1 - t0) * 1000, 3),
        "verdict": verdict,
        "maxrss_kb": rusage.ru_maxrss,
        "load1": round(load1, 1),
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--route", required=True, choices=["qtslab", "glc"])
    ap.add_argument("--bin", required=True, type=Path)
    ap.add_argument("--containers", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--budget", type=float, default=600.0)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--pin", type=int, default=None,
                    help="pin the checked process to this CPU (one-core protocol)")
    ap.add_argument("--note", default="")
    a = ap.parse_args()

    binary = a.bin.resolve()
    bin_bytes = binary.read_bytes()
    bin_id = {
        "basename": binary.name,
        "md5": hashlib.md5(bin_bytes).hexdigest(),
        "sha256": hashlib.sha256(bin_bytes).hexdigest(),
        "bytes": len(bin_bytes),
    }
    host = host_identity()

    rows = []
    containers = sorted(a.containers.glob("*.glc"), key=lambda p: p.stat().st_size)
    if not containers:
        print(f"no .glc containers under {a.containers}", file=sys.stderr)
        return 1
    scratch = Path(f"/tmp/.time-routes-{os.getpid()}.out")
    for c in containers:
        name = c.stem
        size = c.stat().st_size
        for p in range(1, a.reps + 1):
            r = run_one(a.route, binary, c, a.budget, scratch, a.pin)
            rows.append({
                "name": name, "container_bytes": size, "route": a.route,
                "pass": p, "budget_s": a.budget, **r, "pinned_cpu": a.pin, "host": host,
                "binary": bin_id, "note": a.note,
            })
            print(f"{name} pass {p}: {r['outcome']}"
                  f" {r['t_e2e_ms'] and str(r['t_e2e_ms']) + ' ms' or ''}",
                  flush=True)
            if r["outcome"] != "accept":
                break
    scratch.unlink(missing_ok=True)
    a.out.write_text(json.dumps(rows, indent=1) + "\n")
    print(f"wrote {len(rows)} rows -> {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
