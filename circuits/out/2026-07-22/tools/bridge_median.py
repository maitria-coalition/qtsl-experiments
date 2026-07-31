# bridge_median.py — circuits bridge receipt: in-process median-of-N
# single-certificate check timing. Usage:
#   python3 bridge_median.py <checker-dir> <fixture.cbor> <N>
import sys, time, statistics, importlib.util

ckdir, fixture, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
sys.path.insert(0, ckdir)
import cbor2  # same dependency the checker itself uses

spec = importlib.util.spec_from_file_location("ck_manifest", ckdir + "/ck_manifest.py")
ck = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ck)

with open(fixture, "rb") as fh:
    m = cbor2.load(fh)

# warmup (JIT-free CPython, but warms caches/allocator)
for _ in range(10):
    ck.check(m)

ts = []
for _ in range(n):
    t0 = time.perf_counter()
    ck.check(m)
    ts.append((time.perf_counter() - t0) * 1000.0)

print(f"fixture={fixture} n={n} median={statistics.median(ts):.4f} ms "
      f"min={min(ts):.4f} mean={statistics.mean(ts):.4f} max={max(ts):.4f}")
