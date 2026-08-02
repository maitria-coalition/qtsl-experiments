#!/usr/bin/env python3
"""Preregistered analysis for the sec1 order/interleave experiment.

Reads the four arm receipts from gb10-order-interleave/, applies exactly
the analysis declared in PREREG.md (medians of wall_s over 10 measured
rows per bench; arm D discards its first round as the preregistered
warmup), and writes ANALYSIS.json beside the receipts. Exits nonzero on
an E1 (validity) failure; timing verdicts are printed, never gated on.

Orchestration/analysis only — computationally insignificant by the house
bar (checker-compare.py precedent).
"""
import json
import statistics
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
OI = HERE / "out/glc-baked/sec1/gb10-order-interleave"
BENCHES = ["faulty_marbles", "faulty_regions", "faulty_unreliable"]
ARMS = ["order-mru", "order-rum", "order-umr", "interleave-rr"]
BAND = 0.03  # the preregistered ±3% stability band


def load_arm(label: str):
    path = OI / label / "campaign.json"
    if not path.is_file():
        return None
    receipt = json.loads(path.read_text())
    rows = receipt["rows"]
    if label == "interleave-rr":
        # Preregistered: the first round (first 3 rows, one per bench in
        # roster order) is the warmup; analysis uses rows 3..33.
        expected = [b for _ in range(11) for b in BENCHES]
        assert [r["bench"] for r in rows] == expected, f"{label}: roster order mismatch"
        assert all(not r["warmup"] for r in rows), f"{label}: unexpected runner warmup flags"
        measured = rows[3:]
    else:
        assert len(rows) == 33, f"{label}: expected 33 rows, got {len(rows)}"
        measured = [r for r in rows if not r["warmup"]]
    assert len(measured) == 30, f"{label}: expected 30 analyzed rows"
    return receipt, rows, measured


def main() -> int:
    gate_ok = True
    summary = {"band": BAND, "arms": {}, "medians": {}, "verdicts": {}}
    medians: dict[str, dict[str, float]] = {}
    for label in ARMS:
        loaded = load_arm(label)
        if loaded is None:
            print(f"{label}: MISSING (arm not yet run)")
            continue
        receipt, rows, measured = loaded
        # E1: timing-independent validity, every row (warmups included).
        bad = [
            r for r in rows
            if not r.get("byte_identical")
            or not r.get("geolog", {}).get("accept")
            or r.get("geolog", {}).get("refusals")
            or r.get("geolog", {}).get("violations")
        ]
        if bad:
            print(f"{label}: E1 FAILURE — {len(bad)} rows not byte-identical/geolog-clean")
            gate_ok = False
        lc = receipt["load_context"]
        gate = lc["gate_max_load_1min"]
        transients = [
            i for i, r in enumerate(rows)
            if r["loadavg_before_1min"] > gate or r["loadavg_after_1min"] > gate
        ]
        med = {}
        for bench in BENCHES:
            walls = [r["wall_s"] for r in measured if r["bench"] == bench]
            assert len(walls) == 10, f"{label}/{bench}: expected 10 measured rows"
            med[bench] = statistics.median(walls)
        medians[label] = med
        order = list(dict.fromkeys(r["bench"] for r in rows))
        summary["arms"][label] = {
            "roster_order": order,
            "complete_roster": receipt["complete_roster"],
            "loadavg_start": lc["loadavg_start"],
            "loadavg_max_1min_observed": lc["loadavg_max_1min_observed"],
            "nice": lc["nice"],
            "cpus_allowed_list": lc["cpus_allowed_list"],
            "rows_flagged_load_transient": transients,
            "mtk_revision": receipt["mtk_revision"],
            "binary_sha256": receipt["binary_sha256"],
        }
        summary["medians"][label] = med
        print(f"{label}: medians " + "  ".join(f"{b}={med[b]:.6f}" for b in BENCHES)
              + (f"  [load transients at rows {transients}]" if transients else ""))

    order_arms = [a for a in ("order-mru", "order-rum", "order-umr") if a in medians]
    if len(order_arms) >= 2:
        for bench in BENCHES:
            vals = {a: medians[a][bench] for a in order_arms}
            spread = max(vals.values()) / min(vals.values()) - 1.0
            summary["verdicts"][f"order-spread/{bench}"] = {
                "per_arm": vals, "relative_spread": spread,
                "within_band": spread <= BAND,
            }
            print(f"order spread {bench}: {spread*100:.2f}% "
                  f"({'within' if spread <= BAND else 'EXCEEDS'} the {BAND*100:.0f}% band)")
    if "interleave-rr" in medians and "order-mru" in medians:
        for bench in BENCHES:
            a, d = medians["order-mru"][bench], medians["interleave-rr"][bench]
            delta = d / a - 1.0
            summary["verdicts"][f"interleave-delta/{bench}"] = {
                "blocked": a, "interleaved": d, "relative_delta": delta,
                "within_band": abs(delta) <= BAND,
            }
            print(f"interleave delta {bench}: {delta*+100:+.2f}% vs blocked "
                  f"({'within' if abs(delta) <= BAND else 'EXCEEDS'} the {BAND*100:.0f}% band)")

    (OI / "ANALYSIS.json").write_text(json.dumps(summary, indent=1) + "\n")
    print(f"wrote {OI / 'ANALYSIS.json'}")
    return 0 if gate_ok else 1


if __name__ == "__main__":
    sys.exit(main())
