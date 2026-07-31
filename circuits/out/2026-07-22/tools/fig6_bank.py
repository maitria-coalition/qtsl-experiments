# fig6_bank.py — circuits: per-instance Fig6 data bank (addendum schema).
# Usage: python3 fig6_bank.py <suite-dir> <silicon-tag> <out-dir>
#
# One row per TASK (634 total: 219 liveness + 206 safety + 209 SL).
# t_check_ms = median of 5 in-process check() calls (single core, this box).
# t_produce_ms = the sweep receipts' per-instance producer wall (single-shot,
#   same box, regenerated immediately before this harness ran).
# rational_count / denominator_bits_max are counted by monkeypatching the
#   checker's own dec_frac — i.e. exactly what the replay decodes.
# obligation_count = the checker's own row count for the task (summed over
#   the task's certificates). state_space_size = integer-point count of the
#   manifest's declared state box (product over state_wires of hi-lo+1).
# Chain-replay walls land in a separate TSV (median of 5 per sidecar).
import sys, os, time, statistics, importlib.util, re

W, SILICON, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(OUT, exist_ok=True)
sys.path.insert(0, W)
import cbor2

def load_mod(name):
    spec = importlib.util.spec_from_file_location(name, os.path.join(W, name + ".py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m

ck_manifest = load_mod("ck_manifest")
ck_safety = load_mod("ck_safety")
ck_specchain = load_mod("ck_specchain")

# ------------------------------------------------ receipts producer-ms parse
def parse_table(path, name_col=0, prod_col=None):
    rows = {}
    for line in open(os.path.join(W, path)):
        if not line.startswith("|") or line.startswith("|--") or "instance" in line or "task |" in line:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        try:
            rows[cells[name_col]] = float(cells[prod_col])
        except (ValueError, IndexError):
            continue
    return rows

prod_live = parse_table("SWEEP-RECEIPTS.md", 0, 5)     # per certificate
prod_safe = parse_table("SAFETY-RECEIPTS.md", 0, 6)    # per certificate
prod_sl = parse_table("SL-RECEIPTS.md", 0, 10)         # per task (both certs)

# ------------------------------------------------ instrumented + timed check
def stats_check(mod, m):
    """one instrumented pass (rational stats) + median-of-5 timed passes"""
    counter = {"n": 0, "maxbits": 0}
    orig = mod.dec_frac
    def spy(x):
        f = orig(x)
        counter["n"] += 1
        b = f.denominator.bit_length()
        if b > counter["maxbits"]:
            counter["maxbits"] = b
        return f
    mod.dec_frac = spy
    try:
        r = mod.check(m)
    finally:
        mod.dec_frac = orig
    ts = []
    for _ in range(5):
        t0 = time.perf_counter()
        mod.check(m)
        ts.append((time.perf_counter() - t0) * 1000.0)
    rows = r.get("rows") if isinstance(r, dict) else None
    return rows, counter["n"], counter["maxbits"], statistics.median(ts)

def fixture_facts(fdir, fname):
    path = os.path.join(W, fdir, fname)
    nbytes = os.path.getsize(path)
    with open(path, "rb") as fh:
        m = cbor2.load(fh)
    ss = 1
    for w in m["state_wires"]:
        ss *= (w[2] - w[1] + 1)
    mod = ck_safety if "window" in m else ck_manifest
    rows, nrat, maxbits, med = stats_check(mod, m)
    return nbytes, ss, rows, nrat, maxbits, med

# ------------------------------------------------ class definitions
def group(fdir, splitter):
    tasks = {}
    for f in sorted(os.listdir(os.path.join(W, fdir))):
        if not f.endswith(".cbor"):
            continue
        tasks.setdefault(splitter(f[:-5]), []).append(f)
    return tasks

live = group("fixtures", lambda n: re.sub(r"_p[12]$", "", n))
safe = group("fixtures_safety", lambda n: re.sub(r"_p[12]$", "", n))
sl = group("fixtures_sl", lambda n: re.sub(r"_[wg]$", "", n))
assert (len(live), len(safe), len(sl)) == (219, 206, 209), (len(live), len(safe), len(sl))

CLASSES = [
    ("liveness", live, "fixtures", "closed-form-ranking-v0",
     lambda t, fs: sum(prod_live.get(f[:-5], 0.0) for f in fs)),
    ("safety", safe, "fixtures_safety", "safety-window-v0",
     lambda t, fs: sum(prod_safe.get(f[:-5], 0.0) for f in fs)),
    ("safety+liveness", sl, "fixtures_sl", "window+recurrence-v0",
     lambda t, fs: prod_sl.get(t, 0.0)),
]

hdr = ["instance_id", "section", "lane", "config", "outcome", "t_produce_ms",
       "t_check_ms", "t_total_ms", "timeout_s", "silicon", "cert_bytes",
       "obligation_count", "state_space_size", "retrain_count",
       "denominator_bits_max", "rational_count", "property_class"]
rows_out = []
for cname, tasks, fdir, config, prodfn in CLASSES:
    for task, files in sorted(tasks.items()):
        tb = tr = nrat = 0
        mb = 0
        tchk = 0.0
        ss = None
        for f in files:
            nbytes, s, r, n, b, med = fixture_facts(fdir, f)
            tb += nbytes
            tr += (r or 0)
            nrat += n
            mb = max(mb, b)
            tchk += med
            ss = s if ss is None else max(ss, s)
        tp = prodfn(task, files)
        rows_out.append([task, "circuits", "cpu", config, "solved",
                         f"{tp:.3f}", f"{tchk:.4f}", f"{tp + tchk:.4f}",
                         "18000", SILICON, str(tb), str(tr), str(ss), "0",
                         str(mb), str(nrat), cname])

with open(os.path.join(OUT, "fig6.tsv"), "w") as fh:
    fh.write("\t".join(hdr) + "\n")
    for r in rows_out:
        fh.write("\t".join(r) + "\n")
print(f"fig6.tsv: {len(rows_out)} task rows on {SILICON}")

# ------------------------------------------------ chain walls, separately
chdr = ["instance_id", "property_class", "chain_steps", "chain_pairs",
        "chain_check_ms", "silicon"]
crows = []
for cname, cdir, fdir in [("liveness", "chains", "fixtures"),
                          ("safety", "chains_safety", "fixtures_safety"),
                          ("safety+liveness", "chains_sl", "fixtures_sl")]:
    fixdir = os.path.join(W, fdir)
    for f in sorted(os.listdir(os.path.join(W, cdir))):
        if not f.endswith(".cbor"):
            continue
        with open(os.path.join(W, cdir, f), "rb") as fh:
            sc = cbor2.load(fh)
        ts = []
        for _ in range(5):
            t0 = time.perf_counter()
            r = ck_specchain.check(sc, fixdir)
            ts.append((time.perf_counter() - t0) * 1000.0)
        steps = r.get("steps", "") if isinstance(r, dict) else ""
        pairs = r.get("pairs", "") if isinstance(r, dict) else ""
        crows.append([sc.get("task", f[:-5]), cname, str(steps), str(pairs),
                      f"{statistics.median(ts):.4f}", SILICON])

with open(os.path.join(OUT, "fig6-chains.tsv"), "w") as fh:
    fh.write("\t".join(chdr) + "\n")
    for r in crows:
        fh.write("\t".join(r) + "\n")
print(f"fig6-chains.tsv: {len(crows)} chain rows on {SILICON}")
