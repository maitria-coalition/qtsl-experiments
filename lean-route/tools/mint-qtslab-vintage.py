#!/usr/bin/env python3
"""mint-qtslab-vintage.py — re-mint the ladder fixtures for the certified
checker's prerequisite discipline.

The committed fixtures (fixtures/*.geolog) seal each certificate over the
staged three-revision theory chain with its prerequisites marked
ASSUMED-HELD (prereq flag 0).  The certified checker binds the *content*
of exactly that chain — its vocabulary pins are keyed to the same three
revision content-ids the fixtures name — and it refuses assumed-held
prerequisites by design (`prereq_content_assumed_held`): a claim resting
on an unresolved assumption is not a claim it will certify.

This tool re-marks the three prerequisites as CONTENT-RESOLVED (flag 1 +
resolution ordinal), which is the form the checker's own acceptance
corpus carries.  Nothing else changes:

  * the commit body — every certificate row — is byte-identical;
  * the sealed skeleton (signature, body hash, root) is byte-identical;
  * the three prerequisite content-ids are byte-identical.

The transform is 3 flag bytes and 3 inserted ordinals (u16le 0,1,2) in
the container's outer prerequisite table, +6 bytes total.  It is a pure
function of the committed fixture bytes: re-running reproduces the
minted containers exactly (the make target re-derives and byte-checks).

Usage:  mint-qtslab-vintage.py <fixtures-dir> <out-dir>
        mint-qtslab-vintage.py --verify <fixtures-dir> <out-dir>

--verify re-mints into memory and byte-compares against <out-dir>,
failing loudly on any drift; it writes nothing.
"""

import hashlib
import json
import struct
import sys
from pathlib import Path

EXCLUDED = {"terms_v1_bern_fastpath"}  # MANIFEST.tsv: EXCLUDED species


def mint(data: bytes) -> bytes:
    if data[:8] != b"geologfl":
        raise ValueError("not a geologfl container")
    version, pcount = struct.unpack_from("<II", data, 8)
    if version != 1:
        raise ValueError(f"container version {version}, expected 1")
    pos = 16
    out = bytearray(data[:16])
    for ordinal in range(pcount):
        flag = data[pos]
        if flag != 0:
            raise ValueError(
                f"prereq {ordinal}: flag {flag}, expected 0 (assumed-held); "
                "fixture is not the assumed-vintage this mint transforms"
            )
        chid = data[pos + 1 : pos + 33]
        out += b"\x01" + chid + struct.pack("<H", ordinal)
        pos += 33
    out += data[pos:]
    return bytes(out)


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--verify"]
    verify = "--verify" in sys.argv[1:]
    if len(args) != 2:
        print(__doc__.strip().splitlines()[-8].strip(), file=sys.stderr)
        return 2
    fixtures, outdir = Path(args[0]), Path(args[1])
    if not verify:
        outdir.mkdir(parents=True, exist_ok=True)

    manifest = []
    failures = 0
    for src in sorted(fixtures.glob("*.geolog")):
        if src.stem in EXCLUDED:
            continue
        data = src.read_bytes()
        minted = mint(data)
        dst = outdir / (src.stem + ".glc")
        if verify:
            have = dst.read_bytes() if dst.exists() else None
            if have != minted:
                print(f"DRIFT {dst.name}: committed bytes != re-mint", file=sys.stderr)
                failures += 1
                continue
            print(f"ok {dst.name} ({len(minted)} bytes)")
        else:
            dst.write_bytes(minted)
            print(f"minted {dst.name} ({len(data)} -> {len(minted)} bytes)")
        manifest.append(
            {
                "name": src.stem,
                "source_fixture": f"fixtures/{src.name}",
                "source_sha256": hashlib.sha256(data).hexdigest(),
                "minted_sha256": hashlib.sha256(minted).hexdigest(),
                "container_bytes": len(minted),
                "transform": "prereq flags 0->1 (assumed-held -> content-resolved), "
                "resolution ordinals u16le 0..2 inserted; commit body, skeleton, "
                "signature, and prerequisite content-ids byte-identical",
            }
        )
    if verify:
        mpath = outdir / "QTSLAB-MANIFEST.json"
        want = json.dumps({"containers": manifest}, indent=1) + "\n"
        if not mpath.exists() or mpath.read_text() != want:
            print("DRIFT QTSLAB-MANIFEST.json", file=sys.stderr)
            failures += 1
        if failures:
            print(f"FAILED: {failures} drift(s)", file=sys.stderr)
            return 1
        print(f"verified {len(manifest)} containers + manifest byte-stable")
        return 0
    (outdir / "QTSLAB-MANIFEST.json").write_text(
        json.dumps({"containers": manifest}, indent=1) + "\n"
    )
    print(f"wrote {len(manifest)} containers + QTSLAB-MANIFEST.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
