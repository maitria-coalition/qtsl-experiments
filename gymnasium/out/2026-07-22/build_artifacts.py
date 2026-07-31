#!/usr/bin/env python3
"""build_artifacts.py — gymnasium re-run (2026-07-22): emit fig6.tsv
(per-instance rows, Fig6 addendum schema) + ledger-data.json (the delta
ledger's numeric core) from the receipts under receipts/. Deterministic; re-run after
any receipt refresh. No table in the chapter is written here (gen_tables.ts
owns those)."""
import json, os, re, glob, subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.normpath(os.path.join(HERE, ".."))          # gymnasium/out
REPO = os.path.normpath(os.path.join(OUT, "../.."))

def load(p): return json.load(open(os.path.join(HERE, "receipts", p)))

SIL = {
    "gb10":     "gb10/single core pinned, nice",
    "epyc":     "epyc/single core pinned",
    "rtx5090":  "rtx5090",
}

def produce_map(rec):
    """cert rel-path -> (min_ms, cert_bytes) from a producer receipt tree."""
    m = {}
    def walk(d):
        if isinstance(d, dict):
            if "cert" in d and isinstance(d.get("produce"), dict) and "min_ms" in d["produce"]:
                m[d["cert"]] = (d["produce"]["min_ms"], d.get("cert_bytes"))
            for v in d.values(): walk(v)
    walk(rec["fl_rows"]); walk(rec["tt_rows"]); walk(rec.get("hitting_companion"))
    return m

def ns_map(rec):
    m = {}
    for rows in (rec["fl_rows"], rec["tt_rows"]):
        for env, d in rows.items():
            if isinstance(d, dict) and "ns" in d: m[env] = d["ns"]
    return m

def checker_map(*files):
    m = {}
    for f in files:
        m.update(load(f)["certs"])
    return m

def box2d_rows(fname):
    out = {}
    certs = ["ll/ll_u4_min_a100.cbor", "ll/ll_u4_fixture_n20k.cbor",
             "llc/llc_u4_min_a100.cbor", "llc/llc_u4_fixture_n20k.cbor"]
    lines = [l for l in open(os.path.join(HERE, "receipts", fname)) if l.startswith("ACCEPT")]
    for cert, l in zip(certs, lines):
        d = dict(kv.split("=", 1) for kv in l.split() if "=" in kv)
        out[cert] = {"rows": int(d["rows"]), "verify_median_ns": int(d["verify_median_ns"])}
    return out

def fsize(cert): return os.path.getsize(os.path.join(OUT, cert))

rows = []
def emit(instance, lane, config, silicon, t_prod, t_chk, cert_bytes, obl, sss):
    t_tot = round(t_prod + t_chk, 4) if (t_prod is not None and t_chk is not None) else ""
    rows.append([instance, "gymnasium", lane, config, "solved",
                 "" if t_prod is None else t_prod, "" if t_chk is None else t_chk,
                 t_tot, 0, silicon, cert_bytes, obl, sss if sss is not None else "", 0])

for box, prodf, chkf in [
    ("gb10", "gb10/producer_receipts_rust.1core.json",
             ("gb10/checker_receipts.tips-gb10.json", "gb10/checker_receipts_tt.tips-gb10.json")),
    ("epyc", "epyc/producer_receipts_rust.1core.json",
             ("epyc/checker_receipts.tips.json", "epyc/checker_receipts_tt.tips.json")),
]:
    prec = load(prodf); pm = produce_map(prec); nsm = ns_map(prec)
    cm = checker_map(*chkf)
    for cert, c in sorted(cm.items()):
        p = pm.get(cert, (None, None))
        env = cert.split("/")[0]
        t_chk = round(c["verify_median_ns"] / 1e6, 4)
        emit(cert, "cpu", os.path.basename(cert).replace(".cbor", ""), SIL[box],
             p[0], t_chk, p[1] or fsize(cert), c["rows"], nsm.get(env))
    # blackjack low-k rows (python-emitted; check re-derived via sweep-lanes)
    for l in open(os.path.join(HERE, "receipts", f"{box}/sweep-lanes.txt")):
        m = re.match(r"\S+ (blackjack/rwdyadic_k(?:16|20)_T22) auto :: ACCEPT (.*)", l)
        if m:
            cert = m.group(1) + ".cbor"
            d = dict(kv.split("=", 1) for kv in m.group(2).split() if "=" in kv)
            emit(cert, "cpu", os.path.basename(cert).replace(".cbor", ""), SIL[box],
                 None, round(int(d["verify_median_ns"]) / 1e6, 4), fsize(cert),
                 int(d["rows"]), nsm.get("blackjack"))
    # box2d (replay-only: producer = the committed rollout ledgers, not re-run)
    b2 = box2d_rows(f"{box}/box2d.tips.txt" if box == "epyc" else f"{box}/box2d.tips-gb10.txt")
    for cert, c in b2.items():
        emit(cert, "cpu", os.path.basename(cert).replace(".cbor", ""), SIL[box],
             None, round(c["verify_median_ns"] / 1e6, 4), fsize(cert), c["rows"], None)

# GPU lane (producer only; check not co-located — t_total blank by design)
gp = json.load(open(os.path.join(OUT, "gpu_bench_receipts.json")))["lanes"]
prec = load("gb10/producer_receipts_rust.1core.json"); nsm = ns_map(prec)
for key, l in sorted(gp.items()):
    env, var = key.split("/")
    match = glob.glob(os.path.join(OUT, env, var + "_T*.cbor"))
    cert = os.path.relpath(match[0], OUT) if match else key
    obl = None
    emit(cert, "gpu", var, SIL["rtx5090"], round(l["gpu_min_ms"], 4), None,
         fsize(cert) if match else "", "", nsm.get(env))

hdr = ["instance_id", "section", "lane", "config", "outcome", "t_produce_ms",
       "t_check_ms", "t_total_ms", "timeout_s", "silicon", "cert_bytes",
       "obligation_count", "state_space_size", "retrain_count"]
with open(os.path.join(HERE, "fig6.tsv"), "w") as f:
    f.write("\t".join(hdr) + "\n")
    for r in rows:
        f.write("\t".join(str(x) for x in r) + "\n")
print(f"fig6.tsv: {len(rows)} rows")

# ledger numeric core
def tot_min(rec):
    s = 0
    def walk(d):
        nonlocal s
        if isinstance(d, dict):
            if isinstance(d.get("produce"), dict) and "min_ms" in d["produce"]:
                s += d["produce"]["min_ms"]
            for v in d.values(): walk(v)
    walk(rec["fl_rows"]); walk(rec["tt_rows"]); walk(rec.get("hitting_companion"))
    return round(s, 1)

led = {}
for box in ("gb10", "epyc"):
    p1 = load(f"{box}/producer_receipts_rust.1core.json")
    p4 = load(f"{box}/producer_receipts_rust.4core.json")
    pc = load(f"{box}/producer_receipts_rust.cold4.json")
    led[box] = {
        "catalogue_sum_min_1core_ms": tot_min(p1),
        "catalogue_sum_min_4core_ms": tot_min(p4),
        "cold_end_to_end_4core_s": pc["total_wall_s"],
        "byte_compare_mismatched": pc["byte_compare"]["mismatched"],
        "taxi_rainy_fickle_optimum_1c_ms": p1["tt_rows"]["taxi-rainy-fickle"]["optimum"]["produce"]["min_ms"],
        "taxi_rainy_fickle_optimum_4c_ms": p4["tt_rows"]["taxi-rainy-fickle"]["optimum"]["produce"]["min_ms"],
        "fl8x8_exact_T200_1c_ms": p1["fl_rows"]["frozenlake-8x8"]["exact"]["200"]["produce"]["min_ms"],
        "blackjack_optimum_1c_ms": p1["tt_rows"]["blackjack"]["optimum"]["produce"]["min_ms"],
    }
    sweep = {}
    for l in open(os.path.join(HERE, "receipts", f"{box}/sweep-lanes.txt")):
        m = re.match(r"\S+ (\S+) (ref|auto) :: ACCEPT (.*)", l)
        if m:
            d = dict(kv.split("=", 1) for kv in m.group(3).split() if "=" in kv)
            sweep.setdefault(m.group(1), {})[m.group(2)] = {
                "lane": d["lane"], "median_us": round(int(d["verify_median_ns"]) / 1e3, 1)}
    led[box]["sweep_ref_vs_auto"] = sweep
json.dump(led, open(os.path.join(HERE, "ledger-data.json"), "w"), indent=1, sort_keys=True)
print("ledger-data.json written")
