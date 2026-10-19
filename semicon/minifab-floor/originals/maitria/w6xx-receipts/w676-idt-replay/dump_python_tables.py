#!/usr/bin/env python3
"""W676 idt-replay leg: canonical-table dump from the IDT leg's OWN
constructors (minifab_idt.py family/wcfamily), for the differential
against the mtk Rust port (carved_script.rs dump_canonical_tables).

Usage: dump_python_tables.py <path-to-minifab_idt.py>
Prints the same line format as the Rust side; also re-runs the raw
validator on every table (so the dump is validator-clean by
construction) and asserts tick contiguity (both families construct
contiguous tick blocks; the canonical format encodes them as ranges).
"""

import hashlib
import importlib.util
import sys

path = sys.argv[1]
sha = hashlib.sha256(open(path, "rb").read()).hexdigest()
print(f"# instrument {path} sha256={sha}", file=sys.stderr)

spec = importlib.util.spec_from_file_location("minifab_idt", path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

T = {1: 225, 2: 30, 3: 55, 4: 50, 5: 255}


def dump(label, sched, n):
    mk = mod.validate(sched, n)  # raw-semantics validator, loud on any violation
    print(f"# {label} N={n} validator_makespan={mk}", file=sys.stderr)
    for l in range(n):
        print(f"{label} N={n} lot={l} rel={sched['rel'][l]}")
        for k in (1, 2, 3, 4, 5):
            s, f = sched["s"][(l, k)], sched["f"][(l, k)]
            tks = sched["ticks"][(l, k)]
            assert tks == list(range(tks[0], tks[0] + T[k])), \
                f"non-contiguous ticks lot {l} step {k}"
            print(f"{label} N={n} lot={l} k={k} s={s} f={f} "
                  f"ticks={tks[0]}..{tks[-1]}")


for n in [1, 2, 3, 4, 5, 6, 8, 12, 30, 84]:
    dump("pistar", mod.family(n), n)
for n in [3, 4, 5, 8, 84]:
    dump("wcfamily", mod.wcfamily(n), n)
